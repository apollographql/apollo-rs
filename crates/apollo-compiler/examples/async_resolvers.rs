use apollo_compiler::resolvers::AsyncObjectValue;
use apollo_compiler::resolvers::AsyncResolvedValue;
use apollo_compiler::resolvers::Execution;
use apollo_compiler::resolvers::FieldError;
use apollo_compiler::resolvers::ResolveInfo;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use futures::future::BoxFuture;

async fn async_resolvers_example() {
    let sdl = "
      type Query {
        field1: String
        field2: [Int]
      }
    ";

    struct Query;

    impl AsyncObjectValue for Query {
        fn type_name(&self) -> &str {
            "Query"
        }

        fn resolve_field<'a>(
            &'a self,
            info: &'a ResolveInfo<'a>,
        ) -> BoxFuture<'a, Result<AsyncResolvedValue<'a>, FieldError>> {
            Box::pin(async move {
                match info.field_name() {
                    "field1" => Ok(AsyncResolvedValue::leaf(self.resolve_field1().await)),
                    "field2" => Ok(AsyncResolvedValue::list(self.resolve_field2().await)),
                    _ => Err(self.unknown_field_error(info)),
                }
            })
        }
    }

    impl Query {
        async fn resolve_field1(&self) -> String {
            // totally doing asynchronous I/O here
            "string".into()
        }

        async fn resolve_field2(&self) -> [AsyncResolvedValue<'_>; 4] {
            // very await
            [7, 42, 0, 0].map(AsyncResolvedValue::leaf)
        }
    }

    let query = "
        query($skp: Boolean!) {
            field1 @skip(if: $skp)
            field2
        }
    ";
    let variables_values = &serde_json_bytes::json!({
        "skp": false,
    });

    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let document = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
    let response = Execution::new(&schema, &document)
        .raw_variable_values(variables_values.as_object().unwrap())
        .execute_async(&Query)
        .await
        .unwrap();
    let response = serde_json::to_string_pretty(&response).unwrap();
    expect_test::expect![[r#"
        {
          "data": {
            "field1": "string",
            "field2": [
              7,
              42,
              0,
              0
            ]
          }
        }"#]]
    .assert_eq(&response);
}

fn main() {
    futures::executor::block_on(async_resolvers_example())
}
