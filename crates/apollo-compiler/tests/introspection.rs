use apollo_compiler::ast::FieldDefinition;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::introspection;
use apollo_compiler::name;
use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::resolvers::Execution;
use apollo_compiler::resolvers::FieldError;
use apollo_compiler::resolvers::ObjectValue;
use apollo_compiler::resolvers::ResolveInfo;
use apollo_compiler::resolvers::ResolvedValue;
use apollo_compiler::response::JsonMap;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::ty;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;
use expect_test::expect_file;

#[test]
fn test() {
    let schema = r#"
        "The schema"
        schema {
            query: TheQuery
        }

        """
        Root query type
        """
        type TheQuery implements I {
            id: ID!
            ints: [[Int!]]! @deprecated(reason: "…")
            url(arg: In = { b: 4, a: 2 }): Url
            union: U @deprecated(reason: null)
        }

        interface I {
            id: ID!
        }

        input In {
            a: Int! @deprecated(reason: null)
            b: Int @deprecated
        }

        scalar Url @specifiedBy(url: "https://url.spec.whatwg.org/")

        union U = TheQuery | T

        type T {
            enum: E @deprecated
        }

        enum E { 
            NEW
            OLD @deprecated
        }
    "#;
    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();

    let introspect = |query, variables: JsonMap| {
        let document =
            ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
        let operation = document.operations.get(None).unwrap();
        let variables = coerce_variable_values(&schema, operation, &variables).unwrap();
        let response = introspection::partial_execute(
            &schema,
            &schema.implementers_map(),
            &document,
            operation,
            &variables,
        )
        .unwrap();
        serde_json::to_string_pretty(&response).unwrap()
    };

    let query = r#"
        query WithVarible($verbose: Boolean!) {
            I: __type(name: "I") {
                possibleTypes {
                    name
                    fields @skip(if: $verbose) {
                        name
                    }
                    verboseFields: fields(includeDeprecated: true) @include(if: $verbose) {
                        name
                        deprecationReason
                    }
                }
            }
            Url: __type(name: "Url") @include(if: $verbose) {
                specifiedByURL
            }
        }
    "#;
    let expected = expect!([r#"
        {
          "data": {
            "I": {
              "possibleTypes": [
                {
                  "name": "TheQuery",
                  "fields": [
                    {
                      "name": "id"
                    },
                    {
                      "name": "url"
                    }
                  ]
                }
              ]
            }
          }
        }"#]);
    let variables = [("verbose".into(), false.into())].into_iter().collect();
    let response = introspect(query, variables);
    expected.assert_eq(&response);

    let variables = [("verbose".into(), true.into())].into_iter().collect();
    let response = introspect(query, variables);
    let expected = expect!([r#"
        {
          "data": {
            "I": {
              "possibleTypes": [
                {
                  "name": "TheQuery",
                  "verboseFields": [
                    {
                      "name": "id",
                      "deprecationReason": null
                    },
                    {
                      "name": "ints",
                      "deprecationReason": "…"
                    },
                    {
                      "name": "url",
                      "deprecationReason": null
                    },
                    {
                      "name": "union",
                      "deprecationReason": null
                    }
                  ]
                }
              ]
            },
            "Url": {
              "specifiedByURL": "https://url.spec.whatwg.org/"
            }
          }
        }"#]);
    expected.assert_eq(&response);

    let response = introspect(
        include_str!("../test_data/introspection/introspect_full_schema.graphql"),
        Default::default(),
    );
    expect_file!("../test_data/introspection/response_full.json").assert_eq(&response);
}

#[test]
fn built_in_scalars() {
    // Initially a `Schema` contains all built-in types
    let schema = Schema::new();
    assert!(schema.types.contains_key("ID"));
    assert!(schema.types.contains_key("Int"));
    assert!(schema.types.contains_key("Float"));
    assert!(schema.types.contains_key("String"));
    assert!(schema.types.contains_key("Boolean"));

    // Same when parsing
    let input = r"
      type Query { some: Thing }
      scalar Thing
    ";
    let schema = Schema::parse(input, "").unwrap();
    assert!(schema.types.contains_key("ID"));
    assert!(schema.types.contains_key("Int"));
    assert!(schema.types.contains_key("Float"));
    assert!(schema.types.contains_key("String"));
    assert!(schema.types.contains_key("Boolean"));

    // https://spec.graphql.org/draft/#sec-Scalars.Built-in-Scalars
    // > When returning the set of types from the `__Schema` introspection type,
    // > all referenced built-in scalars must be included.
    // > If a built-in scalar type is not referenced anywhere in a schema
    // > (there is no field, argument, or input field of that type) then it must not be included.
    //
    // We reflect this behavior in the Rust API for `Valid<Schema>`:
    // validation removes unused definitions
    let valid_schema = schema.validate().unwrap();
    assert!(!valid_schema.types.contains_key("ID"));
    assert!(!valid_schema.types.contains_key("Int"));
    assert!(!valid_schema.types.contains_key("Float"));
    // String and Boolean are still used in built-in directives and schema-introspection types
    assert!(valid_schema.types.contains_key("String"));
    assert!(valid_schema.types.contains_key("Boolean"));

    // The `Valid<_>` wrapper makes its contents immutable, but it can be unwraped
    let mut mutable_again = valid_schema.into_inner();
    let ExtendedType::Object(query) = &mut mutable_again.types["Query"] else {
        panic!("expected object")
    };
    query.make_mut().fields.insert(
        name!(sensor),
        FieldDefinition {
            description: None,
            name: name!(sensor),
            arguments: vec![InputValueDefinition {
                description: None,
                name: name!(sensorId),
                ty: ty!(ID).into(),
                default_value: None,
                directives: Default::default(),
            }
            .into()],
            ty: ty!(Float),
            directives: Default::default(),
        }
        .into(),
    );
    let valid_after_mutation = mutable_again.validate().unwrap();

    // Validation also adds/restores definitions as needed:
    assert!(valid_after_mutation.types.contains_key("ID"));
    assert!(!valid_after_mutation.types.contains_key("Int"));
    assert!(valid_after_mutation.types.contains_key("Float"));
    assert!(valid_after_mutation.types.contains_key("String"));
    assert!(valid_after_mutation.types.contains_key("Boolean"));
}

/// Both introspection and other concrete fields with custom resolvers
#[test]
fn mixed() {
    let sdl = r#"
      type Query {
        f: Int 
      }
    "#;
    let query = r#"
      {
        f
        Query: __type(name: "Query") {
          fields {
            name
          }
        }
      }
    "#;

    struct InitialValue;

    impl ObjectValue for InitialValue {
        fn type_name(&self) -> &str {
            "Query"
        }

        fn resolve_field<'a>(
            &'a self,
            info: &ResolveInfo<'a>,
        ) -> Result<ResolvedValue<'a>, FieldError> {
            match info.field_name() {
                "f" => Ok(ResolvedValue::leaf(42)),
                _ => Err(self.unknown_field_error(info)),
            }
        }
    }

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

    // Default config disables schema introspection
    let response = Execution::new(&schema, &document)
        .execute_sync(&InitialValue)
        .unwrap();
    let response = serde_json::to_string_pretty(&response).unwrap();
    expect_test::expect![[r#"
        {
          "errors": [
            {
              "message": "resolver error: schema introspection is disabled",
              "locations": [
                {
                  "line": 4,
                  "column": 16
                }
              ],
              "path": [
                "Query"
              ]
            }
          ],
          "data": {
            "f": 42,
            "Query": null
          }
        }"#]]
    .assert_eq(&response);

    // But it can be enabled
    let response = Execution::new(&schema, &document)
        .enable_schema_introspection(true)
        .execute_sync(&InitialValue)
        .unwrap();
    let response = serde_json::to_string_pretty(&response).unwrap();
    expect_test::expect![[r#"
        {
          "data": {
            "f": 42,
            "Query": {
              "fields": [
                {
                  "name": "f"
                }
              ]
            }
          }
        }"#]]
    .assert_eq(&response);
}

#[test]
fn test_graphql_float_variable_coercion() {
    // Small schema with a Float in the input object
    let sdl = r#"
      type Car { id: ID! kilometers: Float! }
      input CarInput { kilometers: Float! }
      type Query { getCarById(id: ID!): Car }
      type Mutation { insertACar(car: CarInput!): Car!
      }
    "#;

    let parsed_schema = Schema::parse_and_validate(sdl, "sdl").unwrap();

    let executable_mutation = ExecutableDocument::parse_and_validate(
        &parsed_schema,
        "mutation MyCarInsertMutation($car: CarInput!){ insertACar(car:$car) { id kilometers } }",
        "MyCarInsertMutation",
    )
    .unwrap();

    let operation = executable_mutation
        .operations
        .get(Some("MyCarInsertMutation"))
        .unwrap();

    let kilometers_value = 3000;

    // Provide an integer for a Float field
    let input_variables = serde_json_bytes::json!({ "car": { "kilometers": kilometers_value } });
    let map = match input_variables {
        serde_json_bytes::Value::Object(m) => m,
        _ => unreachable!(),
    };

    // Coerce and validate.
    let coerced = coerce_variable_values(&parsed_schema, operation, &map).unwrap();
    let vars_for_exec = coerced.into_inner();

    // ---- Assertions ----
    let car = vars_for_exec
        .get("car")
        .and_then(|value| value.as_object())
        .expect("coerced `car` object");
    assert_eq!(
        car.get("kilometers").unwrap(),
        kilometers_value,
        "kilometers should be present and a valid amount."
    );
}
