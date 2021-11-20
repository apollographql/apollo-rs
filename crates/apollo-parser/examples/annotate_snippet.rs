use annotate_snippets::{
    display_list::{DisplayList, FormatOptions},
    snippet::{Annotation, AnnotationType, Slice, Snippet, SourceAnnotation},
};
use apollo_parser::Parser;

fn this_fails() {
    let src = r#"
    type Field {
        field: * Cat
    }
    "#;
    let parser = Parser::new(src);
    let ast = parser.parse();

    for err in ast.errors().into_iter().rev() {
        let snippet = Snippet {
            title: Some(Annotation {
                label: Some(err.message()),
                id: None,
                annotation_type: AnnotationType::Error,
            }),
            footer: vec![],
            slices: vec![Slice {
                source: src,
                line_start: 0,
                origin: None,
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
}

fn main() {
    this_fails();
}
