use std::collections::HashMap;

use crate::diagnostics::{ApolloDiagnostic, DiagnosticData};
use crate::validation::{NodeLocation, ValidationDatabase};
use crate::{ast, Node};

pub(crate) fn validate_arguments(
    db: &dyn ValidationDatabase,
    arguments: &[Node<ast::Argument>],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashMap::<_, Option<NodeLocation>>::new();

    for argument in arguments {
        let name = &argument.name;
        if let Some(&original_definition) = seen.get(name) {
            let redefined_definition = argument.location();
            diagnostics.push(ApolloDiagnostic::new(
                redefined_definition,
                DiagnosticData::UniqueArgument {
                    name: name.to_string(),
                    original_definition,
                    redefined_definition,
                },
            ));
        } else {
            seen.insert(name, argument.location());
        }
    }

    diagnostics
}
