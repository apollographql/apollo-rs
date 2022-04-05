use std::{fs, path::Path, sync::Arc};

use apollo_compiler::{values, ApolloCompiler};

fn compile_query() -> Option<Arc<values::FragmentDefinition>> {
    let file = Path::new("crates/apollo-compiler/examples/query_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");

    let ctx = ApolloCompiler::new(&src);
    // let errors = ctx.validate();

    let operation_names: Vec<String> = ctx.operations().iter().filter_map(|op| op.name()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());
    let fragments: Vec<String> = ctx.fragments().iter().map(|frag| frag.name()).collect();
    assert_eq!(["vipCustomer"], fragments.as_slice());

    let operation_variables: Vec<String> = ctx
        .operations()
        .find("ExampleQuery")?
        .variables()?
        .iter()
        .map(|var| var.name())
        .collect();

    assert_eq!(["definedVariable"], operation_variables.as_slice());
    ctx.fragments().find("vipCustomer")
}

fn main() -> Result<(), ()> {
    match compile_query() {
        Some(_fragment) => Ok(()),
        None => Err(()),
    }
}
