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
          directive @testDirective(testArg: Boolean!) on FIELD_DEFINITION
          type GrandChildTest {
            test: bool @testDirective(testArg: true)
          }
          type ChildTest {
            test: GrandChildTest! @testDirective(testArg: true)
          }
          type Test {
            test: ChildTest! @testDirective(testArg: true)
          }
          type Query {
            testOperation: Test! @testDirective(testArg: true)
          }
        "#;

    let mut compiler = ApolloCompiler::new();
    compiler.add_type_system(schema_src, "not-used-here.graphql");

    let query_src = r#"query {
          testOperation {
            test {
              test {
                test
              }
            }
          }
        }
        "#;
    let query_id = compiler.add_executable(query_src, "not-used-here.graphql");

    let directives = get_directives_used_in_query(&compiler, &query_id);

    assert_eq!(directives.len(), 4);

    // checkout the tests below as well :)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_finds_directives_on_query() {
        let schema_src = r#"
          directive @testDirective(testArg: Boolean!) on FIELD_DEFINITION
          type Test {
            test: Boolean
          }
          type Query {
            testOperation: Test! @testDirective(testArg: true)
          }
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(&schema_src, "not-used-here.graphql");

        let query_src = r#"query {
          testOperation {
            test
          }
        }
        "#;
        let query_id = compiler.add_executable(&query_src, "not-used-here.graphql");

        let directives = get_directives_used_in_query(&compiler, &query_id);

        assert_eq!(directives.len(), 1);
    }

    #[test]
    fn it_finds_nested_directives() {
        let schema_src = r#"
          directive @testDirective(testArg: Boolean!) on FIELD_DEFINITION
          type GrandChildTest {
            test: bool @testDirective(testArg: true)
          }
          type ChildTest {
            test: GrandChildTest! @testDirective(testArg: true)
          }
          type Test {
            test: ChildTest! @testDirective(testArg: true)
          }
          type Query {
            testOperation: Test! @testDirective(testArg: true)
          }
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(&schema_src, "not-used-here.graphql");

        let query_src = r#"query {
          testOperation {
            test {
              test {
                test
              }
            }
          }
        }
        "#;
        let query_id = compiler.add_executable(&query_src, "not-used-here.graphql");

        let directives = get_directives_used_in_query(&compiler, &query_id);

        assert_eq!(directives.len(), 4);
    }

    #[test]
    fn it_only_directives_used_by_query() {
        let schema_src = r#"
          directive @testDirective(testArg: Boolean!) on FIELD_DEFINITION
          type GrandChildTest {
            test: bool @testDirective(testArg: true)
          }
          type ChildTest {
            test: GrandChildTest! @testDirective(testArg: true)
          }
          type Test {
            test: ChildTest! @testDirective(testArg: true)
          }
          type Query {
            testOperation: Test! @testDirective(testArg: true)
          }
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(&schema_src, "not-used-here.graphql");

        // This query will only hit 2 directives as subfields are not being queried
        let query_src = r#"query {
          testOperation {
            test
          }
        }
        "#;
        let query_id = compiler.add_executable(&query_src, "not-used-here.graphql");

        let directives = get_directives_used_in_query(&compiler, &query_id);

        assert_eq!(directives.len(), 2);
    }
}
