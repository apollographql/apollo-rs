/// This example outlines using apollo-parser with
/// [annotate_snippet](https://docs.rs/annotate-snippets/0.9.1/annotate_snippets/)
/// used by rustlang.
///
/// This allows for a lot of control over how you would like your error output
/// to look before your print them all out.
use std::{fs, path::Path};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use apollo_parser::{ast, Parser};

fn parse_schema() -> ast::Document {
    let file = Path::new("crates/apollo-parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    // this is a nice to have for errors for displaying error origin.
    let file_name = file
        .file_name()
        .expect("Could not get file name.")
        .to_str()
        .expect("Could not get &str from file name.");
    let parser = Parser::new(&src);
    let ast = parser.parse();

    // each err comes with the two pieces of data you need for diagnostics:
    // - message (err.message())
    // - index (err.index())
    for err in ast.errors() {
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
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };

        let dl = DisplayList::from(snippet);
        println!("{dl}\n\n");
    }

    ast.document()
}

fn main() {
    parse_schema();
}
