use std::collections::HashMap;

use crate::{
    diagnostics::{ApolloDiagnostic, Label, DiagnosticData},
    hir::{self, DirectiveLocation},
    validation::ValidationDatabase,
};

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    args: Vec<hir::Argument>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::Argument> = HashMap::new();

    for arg in &args {
        let name = arg.name();
        if let Some(prev_arg) = seen.get(name) {
            let original_definition = prev_arg.loc();
            let redefined_definition = arg.loc();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    redefined_definition.into(),
                    DiagnosticData::UniqueArgument {
                        name: name.into(),
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
            seen.insert(name, arg);
        }
    }

    diagnostics
}

pub fn validate_arguments_definition(
    db: &dyn ValidationDatabase,
    args_def: hir::ArgumentsDefinition,
    dir_loc: DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    db.validate_input_values(args_def.input_values, dir_loc)
}
