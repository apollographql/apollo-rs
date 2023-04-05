use apollo_compiler::ApolloCompiler;
use std::io::Read;

fn main() {
    let (source, filename) = match std::env::args().nth(1).as_deref() {
        Some("-") | None => {
            let mut source = String::new();
            std::io::stdin().read_to_string(&mut source).unwrap();
            (source, "input.graphql".to_string())
        }
        Some(filename) => (
            std::fs::read_to_string(filename).unwrap(),
            filename.to_string(),
        ),
    };

    let mut compiler = ApolloCompiler::new();
    compiler.add_document(&source, &filename);

    let diagnostics = compiler.validate();

    for diagnostic in diagnostics {
        println!("{diagnostic}");
    }
}
