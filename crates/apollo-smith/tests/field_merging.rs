//! Fields selected under the same response name are merged, so generated
//! selections must follow the field selection merging rules, including
//! across fragment spreads and the subselections of same-named fields.

use apollo_compiler::ast::Document as AstDocument;
use common::operation_for_schema;

mod common;

// An implementer's fragment selects `f` with a fixed argument, while the
// generator picks its own arguments for `I.f`. Selecting both in one
// interface selection set would conflict, since one parent isn't an object.
#[test]
fn fragments_on_implementers_merge_with_interface_fields() {
    let schema = "schema { query: Query }
        type Query { i: I, j: I }
        interface I { f(x: Int): Int, g: Int }
        type A implements I { f(x: Int): Int, g: Int }
        fragment FA on A { f(x: 1) g }";
    for seed in 1..=300u64 {
        let Some(op) = operation_for_schema(schema, seed) else {
            continue;
        };
        let full = format!("{schema}\n{op}");
        let ast = AstDocument::parse(&full, "smith.graphql").expect("smith output must parse");
        let Err(errors) = ast.to_mixed_validate() else {
            continue;
        };
        // FA is only used when this operation happens to spread it
        let errors: Vec<String> = errors
            .iter()
            .map(|e| e.error.to_string())
            .filter(|m| !m.contains("`FA`"))
            .collect();
        assert!(errors.is_empty(), "seed={seed}\n{full}\n{errors:#?}");
    }
}
