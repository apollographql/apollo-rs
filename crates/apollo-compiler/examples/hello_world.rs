use std::{fs, path::Path};

use apollo_compiler::{hir, ApolloCompiler, HirDatabase};

fn compile_query() -> Option<hir::FragmentDefinition> {
    let file = Path::new("crates/apollo-compiler/examples/query_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");

    let ctx = ApolloCompiler::new(&src);
    // let errors = ctx.validate();

    let operations = ctx.db.operations();
    let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());
    let frags = ctx.db.fragments();
    let fragments: Vec<_> = frags.iter().map(|frag| frag.name()).collect();
    assert_eq!(["vipCustomer"], fragments.as_slice());

    let operation_variables: Vec<&str> = operations
        .iter()
        .find(|op| op.name() == Some("ExampleQuery"))?
        .variables()
        .iter()
        .map(|var| var.name())
        .collect();

    assert_eq!(operation_variables, ["definedVariable"]);
    ctx.db
        .fragments()
        .iter()
        .find(|op| op.name() == "vipCustomer")
        .cloned()
}

fn main() -> Result<(), ()> {
    match compile_query() {
        Some(_fragment) => Ok(()),
        None => Err(()),
    }
}
