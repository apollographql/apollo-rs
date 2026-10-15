//! Fields are sometimes aliased, including to response names already used
//! in the same selection set, and the result must still satisfy the field
//! selection merging rules.

use apollo_compiler::ast::Document as AstDocument;
use apollo_compiler::ast::Selection;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

fn bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut x = seed.wrapping_mul(0x9E3779B97F4A7C15) | 1;
    (0..len)
        .map(|_| {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            x as u8
        })
        .collect()
}

fn count_aliases(selections: &[Selection]) -> usize {
    selections
        .iter()
        .map(|s| match s {
            Selection::Field(f) => {
                f.alias.is_some() as usize + f.selection_set.as_deref().map_or(0, count_aliases)
            }
            Selection::InlineFragment(i) => count_aliases(&i.selection_set),
            Selection::FragmentSpread(_) => 0,
        })
        .sum()
}

#[test]
fn aliased_selections_stay_mergeable() {
    let schema = "schema { query: Query }
        type Query { a: Int, b: Int, c(x: Int): String, d: String, o: O, u: U }
        type O { a: Int, b: String, o: O }
        type P { a: String, b: String }
        union U = O | P";
    let mut aliases = 0;
    for seed in 1..=200u64 {
        let data = bytes(seed, 512);
        let mut u = Unstructured::new(&data);
        let doc = apollo_parser::Parser::new(schema)
            .parse()
            .document()
            .try_into()
            .expect("schema converts to a smith document");
        let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
        let Some(op) = builder
            .operation_definition()
            .expect("operation generation")
        else {
            continue;
        };
        let op: String = op.into();
        let full = format!("{schema}\n{op}");
        let ast = AstDocument::parse(&full, "smith.graphql").expect("smith output must parse");
        if let Err(errors) = ast.to_mixed_validate() {
            panic!("seed={seed}\n{full}\n{errors}");
        }
        for def in &ast.definitions {
            if let apollo_compiler::ast::Definition::OperationDefinition(op) = def {
                aliases += count_aliases(&op.selection_set);
            }
        }
    }
    assert!(aliases > 0, "no field was aliased");
}

// Counts selection sets that select field `c` more than once with different
// arguments, which is only valid under different aliases.
fn differing_c_arguments(selections: &[Selection]) -> usize {
    let args: Vec<String> = selections
        .iter()
        .filter_map(|s| match s {
            Selection::Field(f) if f.name == "c" => Some(
                f.arguments
                    .iter()
                    .map(|a| format!("{}: {}", a.name, a.value.serialize().no_indent()))
                    .collect::<Vec<_>>()
                    .join(", "),
            ),
            _ => None,
        })
        .collect();
    let here = args.iter().any(|a| a != &args[0]) as usize;
    let nested: usize = selections
        .iter()
        .map(|s| match s {
            Selection::Field(f) => f.selection_set.as_deref().map_or(0, differing_c_arguments),
            Selection::InlineFragment(i) => differing_c_arguments(&i.selection_set),
            Selection::FragmentSpread(_) => 0,
        })
        .sum();
    here + nested
}

#[test]
fn aliases_select_a_field_with_different_arguments() {
    let schema = "schema { query: Query }
        type Query { c(x: Int): String, d: String }";
    let mut differing = 0;
    for seed in 1..=300u64 {
        let data = bytes(seed, 512);
        let mut u = Unstructured::new(&data);
        let doc = apollo_parser::Parser::new(schema)
            .parse()
            .document()
            .try_into()
            .expect("schema converts to a smith document");
        let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
        let Some(op) = builder
            .operation_definition()
            .expect("operation generation")
        else {
            continue;
        };
        let op: String = op.into();
        let full = format!("{schema}\n{op}");
        let ast = AstDocument::parse(&full, "smith.graphql").expect("smith output must parse");
        if let Err(errors) = ast.to_mixed_validate() {
            panic!("seed={seed}\n{full}\n{errors}");
        }
        for def in &ast.definitions {
            if let apollo_compiler::ast::Definition::OperationDefinition(op) = def {
                differing += differing_c_arguments(&op.selection_set);
            }
        }
    }
    assert!(
        differing > 0,
        "no selection set selected `c` with different arguments"
    );
}
