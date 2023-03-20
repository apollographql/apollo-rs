use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{DiagnosticData, Label},
    hir, ApolloDiagnostic, ValidationDatabase,
};

pub fn validate_schema_definition(
    db: &dyn ValidationDatabase,
    schema_def: Arc<hir::SchemaDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if schema_def.query().is_none() {
        if let Some(loc) = db.schema().loc() {
            diagnostics.push(
                ApolloDiagnostic::new(db, loc.into(), DiagnosticData::QueryRootOperationType)
                    .label(Label::new(
                        loc,
                        "`query` root operation type must be defined here",
                    )),
            );
        }
    }
    diagnostics
        .extend(db.validate_root_operation_definitions(schema_def.self_root_operations().to_vec()));

    diagnostics.extend(db.validate_directives(
        schema_def.directives().cloned().collect(),
        hir::DirectiveLocation::Schema,
    ));

    diagnostics
}

// All root operations in a schema definition must be unique.
//
// Return a Unique Operation Definition error in case of a duplicate name.
pub fn validate_root_operation_definitions(
    db: &dyn ValidationDatabase,
    root_op_defs: Vec<hir::RootOperationTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen: HashMap<String, &hir::RootOperationTypeDefinition> = HashMap::new();

    for op_type in root_op_defs.iter() {
        let name = op_type.named_type().name();

        // All root operations in a schema definition must be unique.
        //
        // Return a Unique Operation Definition error in case of a duplicate name.
        if let Some(prev_def) = seen.get(&name) {
            if let (Some(original_definition), Some(redefined_definition)) =
                (prev_def.loc(), op_type.loc())
            {
                diagnostics.push(
                    ApolloDiagnostic::new(
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
                );
            }
        } else {
            seen.insert(name, op_type);
        }

        // Root Operation Named Type must be of Object Type.
        //
        // Return a Object Type error if it's any other type definition.
        let type_def = db.find_type_definition_by_name(op_type.named_type().name());
        if let Some(type_def) = type_def {
            if !type_def.is_object_type_definition() {
                if let Some(op_loc) = op_type.loc() {
                    let kind = type_def.kind();
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            op_loc.into(),
                            DiagnosticData::ObjectType {
                                name: op_type.named_type().name(),
                                ty: kind,
                            },
                        )
                        .label(Label::new(op_loc, format!("This is of `{kind}` type")))
                        .help("root operation type must be of an Object Type"),
                    );
                }
            }
        } else if let Some(op_loc) = op_type.loc() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    op_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: op_type.named_type().name(),
                    },
                )
                .label(Label::new(op_loc, "not found in this scope")),
            );
        }
    }

    diagnostics
}
