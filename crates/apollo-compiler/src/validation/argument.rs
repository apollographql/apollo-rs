use std::collections::HashSet;

use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    validation::ValidationDatabase,
    Node,
};
use apollo_parser::cst::{self, CstNode};

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    arguments: &[Node<ast::Argument>],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashSet<ast::Name> = HashSet::new();

    let argument_location = |name: &ast::Name| {
        super::lookup_cst_location(
            db.upcast(),
            name.location().unwrap(),
            |cst: cst::Argument| Some(cst.syntax().text_range()),
        )
    };

    for argument in arguments {
        let name = &argument.name;
        if let Some(original) = seen.get(name) {
            let original_definition = argument_location(original).unwrap();
            let redefined_definition = argument_location(name).unwrap();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    redefined_definition.into(),
                    DiagnosticData::UniqueArgument {
                        name: name.to_string(),
                        original_definition: original_definition.into(),
                        redefined_definition: redefined_definition.into(),
                    },
                )
                .labels([
                    Label::new(
                        original_definition,
                        format!("previously provided `{name}` here"),
                    ),
                    Label::new(
                        redefined_definition,
                        format!("`{name}` provided again here"),
                    ),
                ])
                .help(format!("`{name}` argument must only be provided once.")),
            );
        } else {
            seen.insert(name.clone());
        }
    }

    diagnostics
}
