fn build_fragment_chain(size: usize) -> String {
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

    for i in 1..size {
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
          fragment typeFragment{size} on __Type {{
            ofType {{
              name
            }}
          }}"
    ));

    query
}

fn build_directive_chain(size: usize) -> String {
    let mut schema = r#"
      type Query {
        field: Int! @directive(arg: true)
      }

      directive @directive(arg: Boolean @argDir1) on FIELD_DEFINITION
    "#
    .to_string();

    for i in 1..size {
        schema.push_str(&format!(
            "
            directive @argDir{i}(arg: Boolean @argDir{}) on ARGUMENT_DEFINITION
            ",
            i + 1
        ));
    }
    schema.push_str(&format!(
        "
        directive @argDir{size}(arg: Boolean) on ARGUMENT_DEFINITION
          "
    ));

    schema
}

fn build_input_object_chain(size: usize) -> String {
    let mut schema = r#"
      type Query {
        field(arg: VeryVeryDeep): Boolean
      }

      input VeryVeryDeep {
        nest: VeryVeryDeep1!
      }
    "#
    .to_string();

    for i in 1..size {
        schema.push_str(&format!(
            "
            input VeryVeryDeep{i} {{ nest: VeryVeryDeep{}! }}
            ",
            i + 1
        ));
    }
    schema.push_str(&format!(
        "
        input VeryVeryDeep{size} {{ final: Boolean }}
          "
    ));

    schema
}

#[test]
fn long_fragment_chains_do_not_overflow_stack() {
    // Build a query that applies 1K fragments
    // Validating it would take a lot of recursion and blow the stack
    let query = build_fragment_chain(1_000);

    let errors = apollo_compiler::parse_mixed_validate(
        format!(
            "type Query {{ a: Int }}
            {query}"
        ),
        "overflow.graphql",
    )
    .expect_err("must have recursion errors");

    let expected = expect_test::expect![[r#"
        Error: too much recursion
        Error: `typeFragment1` contains too much nesting
            ╭─[overflow.graphql:11:11]
            │
         11 │           fragment typeFragment1 on __Type {
            │           ───────────┬──────────  
            │                      ╰──────────── references a very long chain of fragments in its definition
        ────╯
    "#]];
    expected.assert_eq(&errors.to_string());
}

#[test]
fn not_long_enough_fragment_chain_applies_correctly() {
    // Stay just under the recursion limit
    let query = build_fragment_chain(99);

    let _ = apollo_compiler::parse_mixed_validate(
        format!(
            "type Query {{ a: Int }}
            {query}"
        ),
        "no_overflow.graphql",
    )
    .expect("must not have recursion errors");
}

#[test]
fn long_directive_chains_do_not_overflow_stack() {
    // Build a schema that defines hundreds of directives that all use each other in their
    // argument list
    // Validating it would take a lot of recursion and a lot of time
    let schema = build_directive_chain(500);

    let partial = apollo_compiler::Schema::parse_and_validate(schema, "directives.graphql")
        .expect_err("must have recursion errors");

    assert_eq!(partial.errors.len(), 469);
}

#[test]
fn not_long_enough_directive_chain_applies_correctly() {
    // Stay just under the recursion limit
    let schema = build_directive_chain(31);

    let _schema = apollo_compiler::Schema::parse_and_validate(schema, "directives.graphql")
        .expect("must not have recursion errors");
}

#[test]
fn long_input_object_chains_do_not_overflow_stack() {
    // Build a very deeply nested input object
    // Validating it would take a lot of recursion and a lot of time
    let schema = build_input_object_chain(500);

    let partial = apollo_compiler::Schema::parse_and_validate(schema, "input_objects.graphql")
        .expect_err("must have recursion errors");

    // The final 199 input objects do not cause recursion errors because the chain is less than 200
    // directives deep.
    assert_eq!(partial.errors.len(), 469);
}

#[test]
fn not_long_enough_input_object_chain_applies_correctly() {
    // Stay just under the recursion limit
    let schema = build_input_object_chain(31);

    let _schema = apollo_compiler::Schema::parse_and_validate(schema, "input_objects.graphql")
        .expect("must not have recursion errors");
}
