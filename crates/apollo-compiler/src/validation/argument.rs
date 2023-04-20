use std::collections::HashMap;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, DirectiveLocation},
    validation::ValidationDatabase,
};

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    args: Vec<hir::Argument>,
    parent_op: Option<hir::Name>,
    field_definition: hir::FieldDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::Argument> = HashMap::new();

    let op = db.find_operation(
        args.loc().file_id(),
        parent_op.as_ref().map(|n| n.src.clone()),
    );

    let defined_vars = if let Some(op) = op {
        let op = op.clone();
        op.variables()
    } else {
        Vec::new().as_slice()
    };

    for arg in &args {
        let name = arg.name();

        if let hir::Value::Variable(var) = arg.value() {
            // get the variable definition type as it's defined on an executable definition
            let var_ty = defined_vars.iter().find_map(|v| {
                if v.name() == var.name() {
                    Some(v.ty())
                } else {
                    None
                }
            });
            // get the argument type as it's defined in the type system
        }

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

        let exists = field_definition
            .arguments()
            .input_values()
            .iter()
            .any(|arg_def| arg.name() == arg_def.name());

        if !exists {
            let mut labels = vec![Label::new(arg.loc, "argument name not found")];
            if let Some(loc) = field_definition.loc {
                labels.push(Label::new(loc, "field declared here"));
            };
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    arg.loc.into(),
                    DiagnosticData::UndefinedArgument {
                        name: arg.name().into(),
                    },
                )
                .labels(labels),
            );
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
