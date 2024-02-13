//! This example outlines using apollo-parser with [annotate-snippets], the error
//! printing library used by the Rust compiler.
//!
//! This allows for a lot of control over how you would like your error output
//! to look before your print them all out.
//!
//! [annotate-snippets]: https://docs.rs/annotate-snippets/0.10.0/annotate_snippets/

use annotate_snippets::Annotation;
use annotate_snippets::AnnotationType;
use annotate_snippets::Renderer;
use annotate_snippets::Slice;
use annotate_snippets::Snippet;
use annotate_snippets::SourceAnnotation;
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
        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(err.message()),
                id: None,
                annotation_type: AnnotationType::Error,
            }),
            footer: vec![],
            slices: vec![Slice {
                source: &src,
                line_start: 0,
                origin: Some(file_name),
                fold: false,
                annotations: vec![SourceAnnotation {
                    label: err.message(),
                    annotation_type: AnnotationType::Error,
                    range: (err.index(), err.index() + err.data().len()), // (start, end) of error token
                }],
            }],
        };

        let renderer = Renderer::styled();
        println!("{}\n\n", renderer.render(snippet));
    }

    cst.document()
}

fn main() {
    parse_schema();
}
