use apollo_compiler::{
    hir::{Directive, Field},
    ApolloCompiler, FileId, HirDatabase,
};

fn get_directives_used_in_query(compiler: &ApolloCompiler, query_id: &FileId) -> Vec<Directive> {
    // seed the stack with top-level fields
    let mut stack: Vec<Field> =
        compiler
            .db
            .operations(*query_id)
            .iter()
            .fold(vec![], |mut acc, operation_definition| {
                acc.extend(operation_definition.selection_set().fields().to_vec());
                acc
            });

    let mut directives = vec![];

    // depth first search for nested fields with directives
    while !stack.is_empty() {
        if let Some(field) = stack.pop() {
            if let Some(field_definition) = &field.field_definition(&compiler.db) {
                directives.extend(field_definition.directives().to_vec());
            }
            stack.extend(field.selection_set().fields().to_vec());
        }
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

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema_src, "not-used-here.graphql");

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
    let query_id0 = compiler.add_executable(query_src0, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&compiler, &query_id0);
    assert_eq!(directives.len(), 4);

    let query_src1 = r#"query {
          directivesQuery {
            test
          }
        }
        "#;
    let query_id1 = compiler.add_executable(query_src1, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&compiler, &query_id1);
    assert_eq!(directives.len(), 2);

    let query_src2 = r#"query {
          noDirectivesQuery {
            test
          }
        }
        "#;
    let query_id2 = compiler.add_executable(query_src2, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&compiler, &query_id2);
    assert_eq!(directives.len(), 0);
}
