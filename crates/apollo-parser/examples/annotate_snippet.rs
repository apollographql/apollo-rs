use std::{fs, path::Path};

use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use apollo_parser::{ast, Parser};

fn parse_schema() -> ast::Document {
    let file = Path::new("crates/apollo-parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    let file_name = file
        .file_name()
        .expect("Could not get file name.")
        .to_str()
        .expect("Could not get &str from file name.");
    let parser = Parser::new(&src);
    let ast = parser.parse();

    for err in ast.errors().into_iter() {
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
                    range: (err.index(), err.index() + err.data().len()),
                }],
            }],
            opt: FormatOptions {
                color: true,
                ..Default::default()
            },
        };

        let dl = DisplayList::from(snippet);
        println!("{}\n\n", dl);
    }

    ast.document()
}

fn main() {
    parse_schema();
}
