use std::collections::HashMap;

use crate::{
    diagnostics::{QueryRootOperationType, UniqueDefinition},
    hir::RootOperationTypeDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if db.schema().query(db.upcast()).is_none() {
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

    // All root operations in a schema definition must be unique.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen: HashMap<String, &RootOperationTypeDefinition> = HashMap::new();
    for op_type in db.schema().root_operation_type_definition().iter() {
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
