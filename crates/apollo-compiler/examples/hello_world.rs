use apollo_compiler::executable;
use apollo_compiler::ApolloCompiler;
use apollo_compiler::Node;
use apollo_compiler::ReprDatabase;
use std::fs;
use std::path::Path;

fn compile_query() -> Option<Node<executable::Fragment>> {
    let file = Path::new("crates/apollo-compiler/examples/query_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");

    let mut compiler = ApolloCompiler::new();
    let document_id = compiler.add_document(&src, file);
    // let errors = ctx.validate();

    let document = compiler.db.executable_document(document_id);
    let operation_names: Vec<_> = document
        .named_operations
        .keys()
        .map(|n| n.as_str())
        .collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());
    let fragments: Vec<_> = document
        .fragments
        .keys()
        .map(|name| name.as_str())
        .collect();
    assert_eq!(["vipCustomer"], fragments.as_slice());

    let operation_variables: Vec<&str> = document
        .named_operations
        .get("ExampleQuery")?
        .variables
        .iter()
        .map(|var| var.name.as_str())
        .collect();

    assert_eq!(operation_variables, ["definedVariable"]);
    document.fragments.get("vipCustomer").cloned()
}

fn main() -> Result<(), ()> {
    match compile_query() {
        Some(_fragment) => Ok(()),
        None => Err(()),
    }
}
