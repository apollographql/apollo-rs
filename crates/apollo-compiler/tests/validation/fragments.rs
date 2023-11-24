#[test]
fn long_fragment_chains_do_not_overflow_stack() {
    // Build a query that applies thousands of fragments
    // Validating it would take a lot of recursion and blow the stack
    let mut query = r#"
      query Introspection{
        __schema {
           types {
            ...typeFragment1
          }
        }
      }
    "#
    .to_string();

    let fragments: usize = 1_000;
    for i in 1..fragments {
        query.push_str(&format!(
            "
          fragment typeFragment{i} on __Type {{
            ofType {{
              ...typeFragment{}
            }}
          }}",
            i + 1
        ));
    }
    query.push_str(&format!(
        "
          fragment typeFragment{fragments} on __Type {{
            ofType {{
              name
            }}
          }}"
    ));

    let (schema, executable) = apollo_compiler::parse_mixed(
        format!(
            "type Query {{ a: Int }}
            {query}"
        ),
        "overflow.graphql",
    );

    let errors = executable
        .validate(&schema)
        .expect_err("must have recursion errors");

    let expected = expect_test::expect![[r#"
        Error: too much recursion
        Error: `typeFragment1` fragment cannot reference itself
            ╭─[overflow.graphql:11:11]
            │
         11 │           fragment typeFragment1 on __Type {
            │           ───────────┬──────────  
            │                      ╰──────────── recursive fragment definition
        ────╯
    "#]];
    expected.assert_eq(&errors.to_string_no_color());
}
