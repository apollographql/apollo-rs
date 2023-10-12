//! This example collects all directives declared on the fields that are queried by an operation.

use apollo_compiler::executable::Directive;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Node;
use apollo_compiler::Schema;

fn get_directives_used_in_query(doc: &ExecutableDocument) -> Vec<&Node<Directive>> {
    // seed the stack with top-level fields
    let mut stack: Vec<_> = doc
        .all_operations()
        .flat_map(|op| op.definition().selection_set.fields())
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

    let schema = Schema::parse(schema_src, "not-used-here.graphql");

    let query_src0 = r#"query {
          directivesQuery {
            test {
              test {
                test
              }
            }
          }
        }
        "#;
    let query0 = ExecutableDocument::parse(&schema, query_src0, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&query0);
    assert_eq!(directives.len(), 4);

    let query_src1 = r#"query {
          directivesQuery {
            test
          }
        }
        "#;
    let query1 = ExecutableDocument::parse(&schema, query_src1, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&query1);
    assert_eq!(directives.len(), 2);

    let query_src2 = r#"query {
          noDirectivesQuery {
            test
          }
        }
        "#;
    let query2 = ExecutableDocument::parse(&schema, query_src2, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&query2);
    assert_eq!(directives.len(), 0);
}
