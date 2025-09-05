//! This example outlines using apollo-parser with [annotate-snippets], the error
//! printing library used by the Rust compiler.
//!
//! This allows for a lot of control over how you would like your error output
//! to look before your print them all out.
//!
//! [annotate-snippets]: https://docs.rs/annotate-snippets/0.12.0/annotate_snippets/

use annotate_snippets::AnnotationKind;
use annotate_snippets::Level;
use annotate_snippets::Renderer;
use annotate_snippets::Snippet;
use apollo_parser::cst;
use apollo_parser::Parser;
use std::fs;
use std::path::Path;

fn parse_schema() -> cst::Document {
    let file = Path::new("crates/apollo-parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    // this is a nice to have for errors for displaying error origin.
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
        let snippet = Level::ERROR.primary_title(err.message()).element(
            Snippet::source(&src)
                .line_start(0)
                .path(file_name)
                .fold(true)
                .annotation(
                    AnnotationKind::Primary
                        .span(err.index()..err.index() + err.data().len())
                        .label(err.message()),
                ),
        );

        let renderer = Renderer::styled();
        println!("{}\n\n", renderer.render(&[snippet]));
    }

    cst.document()
}

fn main() {
    parse_schema();
}
