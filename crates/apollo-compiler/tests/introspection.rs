use apollo_compiler::ast::FieldDefinition;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::introspection;
use apollo_compiler::name;
use apollo_compiler::request::coerce_variable_values;
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
