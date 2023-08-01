use apollo_parser::mir;
use apollo_parser::Parser;
use triomphe::Arc;

fn main() {
    let input = r#"
        query {
            a(arg: 4)
            b @remove
            ... {
                c @remove
                d
            }
            ... Frag
            ... {
                h @remove
            }
            i {
                j @remove
            }
        }
        fragment Frag on Something {
            e
            f
            g @remove
        }
    "#;
    let ast = Parser::new(input).parse();
    for error in ast.errors() {
        println!("{}", error)
    }
    let mut mir = ast.into_mir();
    filter_document(&mut mir);
    assert_eq!(
        mir.serialize().no_indent().to_string(),
        "{ a(arg: 4) ... { d } ...Frag } fragment Frag on Something { e f }"
    );
}

fn filter_document(document: &mut mir::Document) {
    for def in &mut document.definitions {
        match &**def {
            mir::Definition::OperationDefinition(_) => {}
            mir::Definition::FragmentDefinition(_) => {}
            _ => continue,
        };
        match Arc::make_mut(def) {
            mir::Definition::OperationDefinition(op) => {
                assert!(
                    filter_selection_set(&mut op.selection_set),
                    "operation was emptied"
                )
            }
            mir::Definition::FragmentDefinition(frag) => {
                // Left as an exercise to the reader:
                // remove corresponding fragment spreads when a fragment becomes empty.
                // May require a topological sort for spreads in other fragment definitions.
                assert!(filter_selection_set(&mut frag.selection_set));
            }
            _ => {}
        }
    }
}

/// Returns wether the parent should be retained.
fn filter_selection_set(selection_set: &mut Vec<Arc<mir::Selection>>) -> bool {
    if selection_set.is_empty() {
        return true;
    }
    selection_set.retain_mut(|selection| match Arc::make_mut(selection) {
        mir::Selection::Field(field) => {
            if field.directive_by_name("remove").is_none() {
                filter_selection_set(&mut field.selection_set)
            } else {
                false
            }
        }
        mir::Selection::InlineFragment(inline_fragment) => {
            filter_selection_set(&mut inline_fragment.selection_set)
        }
        mir::Selection::FragmentSpread(_) => true,
    });
    !selection_set.is_empty()
}
