use crate::ast;
use crate::collections::HashMap;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;
use crate::validation::SourceSpan;
use crate::Node;

pub(crate) fn validate_arguments(
    diagnostics: &mut DiagnosticList,
    arguments: &[Node<ast::Argument>],
) {
    let mut seen = HashMap::<_, Option<SourceSpan>>::default();

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
