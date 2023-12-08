use std::collections::HashMap;

use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::NodeLocation;
use crate::{ast, Node};

pub(crate) fn validate_arguments(arguments: &[Node<ast::Argument>]) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();
    let mut seen = HashMap::<_, Option<NodeLocation>>::new();

    for argument in arguments {
        let name = &argument.name;
        if let Some(&original_definition) = seen.get(name) {
            let redefined_definition = argument.location();
            diagnostics.push(ValidationError::new(
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
