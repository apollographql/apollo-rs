use crate::ast;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;
use crate::validation::NodeLocation;
use crate::Node;
use std::collections::HashMap;

pub(crate) fn validate_arguments(
    diagnostics: &mut DiagnosticList,
    arguments: &[Node<ast::Argument>],
) {
    let mut seen = HashMap::<_, Option<NodeLocation>>::new();

    for argument in arguments {
        let name = &argument.name;
        if let Some(&original_definition) = seen.get(name) {
            let redefined_definition = argument.location();
            diagnostics.push(
                redefined_definition,
                DiagnosticData::UniqueArgument {
                    name: name.clone(),
                    original_definition,
                    redefined_definition,
                },
            );
        } else {
            seen.insert(name, argument.location());
        }
    }
}
