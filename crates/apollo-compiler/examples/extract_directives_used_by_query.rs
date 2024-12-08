//! This example collects all directives declared on the fields that are queried by an operation.

use apollo_compiler::executable::Directive;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Node;
use apollo_compiler::Schema;

fn get_directives_used_in_query(doc: &ExecutableDocument) -> Vec<&Node<Directive>> {
    // seed the stack with top-level fields
    let mut stack: Vec<_> = doc
        .operations
        .iter()
        .flat_map(|op| op.selection_set.fields())
        .chain(
            // in case the query has fragments
            doc.fragments
                .iter()
                .flat_map(|(_, fragment)| fragment.selection_set.fields()),
        )
        .collect();

    let mut directives = vec![];

    // depth first search for nested fields with directives
    while let Some(field) = stack.pop() {
        directives.extend(field.definition.directives.iter());
        stack.extend(field.selection_set.fields());
    }

    directives
}

fn main() {
    let schema_src = r#"
          directive @testDirective0(testArg: Boolean!) on FIELD_DEFINITION
          directive @testDirective1(testArg: Boolean!) on FIELD_DEFINITION
          directive @testDirective2(testArg: Boolean!) on FIELD_DEFINITION
          directive @testDirective3(testArg: Boolean!) on FIELD_DEFINITION

          type GrandChildTest {
            test: Boolean @testDirective3(testArg: true)
          }

          type ChildTest {
            test: GrandChildTest! @testDirective2(testArg: true)
          }

          type Test {
            test: ChildTest! @testDirective1(testArg: true)
          }

          type NoDirectivesType {
            test: Boolean
          }

          type Query {
            directivesQuery: Test! @testDirective0(testArg: true)
            noDirectivesQuery: NoDirectivesType!
          }
        "#;

    let schema = Schema::parse_and_validate(schema_src, "not-used-here.graphql").unwrap();

    let query_src = r#"query {
          directivesQuery {
            test {
              test {
                test
              }
            }
          }
        }
        "#;
    let query = ExecutableDocument::parse_and_validate(&schema, query_src, "not-used-here.graphql")
        .unwrap();

    let directives = get_directives_used_in_query(&query);
    assert_eq!(directives.len(), 4);

    let query_src = r#"query {
          directivesQuery {
            test
          }
        }
        "#;
    let query = ExecutableDocument::parse(&schema, query_src, "not-used-here.graphql").unwrap();

    let directives = get_directives_used_in_query(&query);
    assert_eq!(directives.len(), 2);

    let query_src = r#"query {
          noDirectivesQuery {
            test
          }
        }
        "#;
    let query = ExecutableDocument::parse(&schema, query_src, "not-used-here.graphql").unwrap();

    let directives = get_directives_used_in_query(&query);
    assert_eq!(directives.len(), 0);

    let query_src = r#"query {
          directivesQuery {
            ... testFragment
          }
        }
        fragment testFragment on Test {
          test
        }
        "#;
    let query = ExecutableDocument::parse(&schema, query_src, "not-used-here.graphql").unwrap();

    let directives = get_directives_used_in_query(&query);
    assert_eq!(directives.len(), 2);
}
