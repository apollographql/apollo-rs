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
    dbg!(mir);
}

fn filter_document(document: &mut mir::Document) {
    for def in &mut document.definitions {
        match &**def {
            mir::Definition::OperationDefinition(_) => {}
            mir::Definition::FragmentDefinition(_) => {}
            _ => continue,
        };
        match Arc::make_mut(def) {
            mir::Definition::OperationDefinition(op) => filter_selection_set(&mut op.selection_set),
            mir::Definition::FragmentDefinition(frag) => {
                filter_selection_set(&mut frag.selection_set)
            }
            _ => {}
        }
    }
}

fn filter_selection_set(selection_set: &mut Vec<Arc<mir::Selection>>) {
    selection_set.retain_mut(|selection| match Arc::make_mut(selection) {
        mir::Selection::Field(field) => {
            let retain = !field.directives.iter().any(|dir| dir.name == "remove");
            if retain {
                filter_selection_set(&mut field.selection_set);
            }
            retain
        }
        mir::Selection::InlineFragment(inline_fragment) => {
            filter_selection_set(&mut inline_fragment.selection_set);
            true
        }
        mir::Selection::FragmentSpread(_) => true,
    })
}
