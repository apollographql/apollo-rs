use apollo_compiler::execution::coerce_variable_values;
use apollo_compiler::execution::JsonMap;
use apollo_compiler::execution::Response;
use apollo_compiler::execution::SchemaIntrospection;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;
use expect_test::expect_file;

#[test]
fn test() {
    let schema = r#"
        type Query implements I {
            id: ID!
            int: Int! @deprecated(reason: "…")
            url: Url
        }

        interface I {
            id: ID!
        }

        scalar Url @specifiedBy(url: "https://url.spec.whatwg.org/")
    "#;
    let schema = Schema::parse_and_validate(schema, "schema.graphql").unwrap();

    let introspect = |query, variables: JsonMap| {
        let document =
            ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
        let operation = document.get_operation(None).unwrap();
        let variables = coerce_variable_values(&schema, operation, &variables).unwrap();
        let response = SchemaIntrospection::execute_with(
            &schema,
            &document,
            operation,
            &variables,
            |non_introspection_document| Response {
                errors: Default::default(),
                data: apollo_compiler::execution::ResponseData::Object(Default::default()),
                extensions: [(
                    "NON_INTROSPECTION".into(),
                    non_introspection_document
                        .serialize()
                        .no_indent()
                        .to_string()
                        .into(),
                )]
                .into_iter()
                .collect(),
            },
        );
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
                  "name": "Query",
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
                  "name": "Query",
                  "verboseFields": [
                    {
                      "name": "id",
                      "deprecationReason": null
                    },
                    {
                      "name": "int",
                      "deprecationReason": "…"
                    },
                    {
                      "name": "url",
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
