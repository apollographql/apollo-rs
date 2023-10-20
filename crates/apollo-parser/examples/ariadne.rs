/// This example describes how to use `apollo-parser` with
/// [`ariadne`](https://docs.rs/ariadne/0.3.0/ariadne) diagnostic library.
use std::{fs, path::Path};

use apollo_parser::{cst, Parser};
use ariadne::{Label, Report, ReportKind, Source};

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
        let start = err.index();
        let end = start + err.data().len();
        Report::build(ReportKind::Error, file_name, start)
            .with_message(err.message())
            .with_label(Label::new((file_name, start..end)).with_message(err.message()))
            .finish()
            .eprint((file_name, Source::from(&src)))
            .unwrap();
    }

    cst.document()
}

fn main() {
    parse_schema();
}
