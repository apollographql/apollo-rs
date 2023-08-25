/// This example describes how to use `apollo-parser` with
/// [`miette`](https://docs.rs/miette/3.2.0/miette) diagnostic library.
///
///
use std::{fs, path::Path};

use apollo_parser::{cst, Parser};
use miette::{Diagnostic, NamedSource, Report, SourceSpan};
use thiserror::Error;

// If your application is using a bunch of other thiserror errors,
// `ApolloParserError` can live within that enum and be responsible for just
// `apollo-parser` errors. It should work really nicely together!
#[derive(Error, Debug, Diagnostic)]
#[error("{}", self.ty)]
#[diagnostic(code("apollo-parser parsing error."))]
struct ApolloParserError {
    ty: String,
    #[source_code]
    src: NamedSource,
    #[label("{}", self.ty)]
    span: SourceSpan,
}

fn parse_schema() -> cst::Document {
    let file = Path::new("crates/apollo-parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    // This is really useful for display the src path within the diagnostic.
    let file_name = file
        .file_name()
        .expect("Could not get file name.")
        .to_str()
        .expect("Could not get &str from file name.");

    let parser = Parser::new(&src);
    let cst = parser.parse();

    // each err comes with the two pieces of data you need for diagnostics:
    // - message (err.message())
    // - index (err.index())
    for err in cst.errors() {
        // We need to create a report and print that individually, as the error
        // slice can have many errors.
        let err = Report::new(ApolloParserError {
            src: NamedSource::new(file_name, src.clone()),
            span: (err.index(), err.data().len()).into(), // (offset, length of error token)
            ty: err.message().into(),
        });
        println!("{err:?}");
    }

    cst.document()
}

fn main() {
    parse_schema();
}
