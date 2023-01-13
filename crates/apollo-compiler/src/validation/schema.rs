use std::collections::HashMap;

use crate::{
    diagnostics::{Diagnostic2, DiagnosticData, Label},
    hir::RootOperationTypeDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if db.schema().query(db.upcast()).is_none() {
        if let Some(loc) = db.schema().loc() {
            diagnostics.push(ApolloDiagnostic::Diagnostic2(
                Diagnostic2::new(db, loc.into(), DiagnosticData::QueryRootOperationType).label(
                    Label::new(loc, "`query` root operation type must be defined here"),
                ),
            ));
        }
    }

    // All root operations in a schema definition must be unique.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen: HashMap<String, &RootOperationTypeDefinition> = HashMap::new();
    for op_type in db.schema().root_operation_type_definition().iter() {
        let name = op_type.named_type().name();
        if let Some(prev_def) = seen.get(&name) {
            if let (Some(original_definition), Some(redefined_definition)) =
                (prev_def.loc(), op_type.loc())
            {
                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        db,
                        redefined_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "root operation type definition",
                            name: name.clone(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{name}` redefined here")),
                    ])
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                ));
            }
        } else {
            seen.insert(name, op_type);
        }
    }

    diagnostics
}
