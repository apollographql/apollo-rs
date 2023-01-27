use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{QueryRootOperationType, UniqueDefinition},
    hir, ApolloDiagnostic, ValidationDatabase,
};

pub fn validate_schema_definition(
    db: &dyn ValidationDatabase,
    schema_def: Arc<hir::SchemaDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if schema_def.query(db.upcast()).is_none() {
        if let Some(loc) = db.schema().loc() {
            let offset = loc.offset();
            let len = loc.node_len();
            diagnostics.push(ApolloDiagnostic::QueryRootOperationType(
                QueryRootOperationType {
                    src: db.source_code(loc.file_id()),
                    schema: (offset, len).into(),
                },
            ));
        }
    }
    diagnostics.extend(
        db.validate_root_operation_definitions(
            schema_def.root_operation_type_definition().to_vec(),
        ),
    );

    diagnostics.extend(db.validate_directives(
        schema_def.directives().to_vec(),
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
        if let Some(prev_def) = seen.get(&name) {
            if prev_def.loc().is_some() && op_type.loc().is_some() {
                let prev_offset = prev_def.loc().unwrap().offset();
                let prev_node_len = prev_def.loc().unwrap().node_len();

                let current_offset = op_type.loc().unwrap().offset();
                let current_node_len = op_type.loc().unwrap().node_len();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    name: name.clone(),
                    ty: "root operation type definition".into(),
                    src: db.source_code(prev_def.loc().unwrap().file_id()),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            }
        } else {
            seen.insert(name, op_type);
        }
    }

    diagnostics
}
