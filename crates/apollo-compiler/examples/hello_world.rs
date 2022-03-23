use std::{fs, path::Path, sync::Arc};

use apollo_compiler::{values, ApolloCompiler};

fn compile_query() -> Option<Arc<values::FragmentDefinition>> {
    // let file = Path::new("crates/apollo-compiler/examples/query_with_errors.graphql");
    // let src = fs::read_to_string(file).expect("Could not read schema file.");
    // // This is really useful for display the src path within the diagnostic.
    // let file_name = file
    //     .file_name()
    //     .expect("Could not get file name.")
    //     .to_str()
    //     .expect("Could not get &str from file name.");

    // let ctx = ApolloCompiler::new(&src);
    // let errors = ctx.validate();

    // let operation_names: Vec<String> = ctx.operations().iter().filter_map(|op| op.name()).collect();
    // assert_eq!(["ExampleQuery"], operation_names.as_slice());
    // let fragments: Vec<String> = ctx.fragments().iter().map(|frag| frag.name()).collect();
    // assert_eq!(["vipCustomer"], fragments.as_slice());

    // let operation_variables = ctx.operations().find("ExampleQuery")?.variables()?;
    // // let operation_variables = ctx.operations().find("ExampleQuery").variables().find("definedVariable").ty();
    // ctx.fragments().find("vipCustomer")?
    None
}

fn main() {}
