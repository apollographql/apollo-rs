use std::{fs, path::Path};

use apollo_compiler::{values, ApolloCompiler};

fn compile_query() -> Option<values::FragmentDefinition> {
    let file = Path::new("crates/apollo-compiler/examples/query_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");

    let ctx = ApolloCompiler::new(&src);
    // let errors = ctx.validate();

    let operations = ctx.operations();
    let operation_names: Vec<_> = operations.iter().filter_map(|op| op.name()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());
    let frags = ctx.fragments();
    let fragments: Vec<_> = frags.iter().map(|frag| frag.name()).collect();
    assert_eq!(["vipCustomer"], fragments.as_slice());

    let operation_variables: Vec<String> = ctx
        .operations()
        .find("ExampleQuery")?
        .variables()
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
