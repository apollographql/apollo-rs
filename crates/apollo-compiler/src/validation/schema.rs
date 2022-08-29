use std::collections::HashMap;

use crate::{
    diagnostics::{QueryRootOperationType, UniqueDefinition},
    values::RootOperationTypeDefinition,
    ApolloDiagnostic, Document,
};

pub fn check(db: &dyn Document) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if db.schema().query(db).is_none() {
        if let Some(node) = db.schema().ast_node(db) {
            let offset: usize = node.text_range().start().into();
            let len: usize = node.text_range().len().into();
            diagnostics.push(ApolloDiagnostic::QueryRootOperationType(
                QueryRootOperationType {
                    src: db.input(),
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
            if prev_def.ast_node(db).is_some() && op_type.ast_node(db).is_some() {
                let prev_offset: usize = prev_def.ast_node(db).unwrap().text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db).unwrap().text_range().len().into();

                let current_offset: usize =
                    op_type.ast_node(db).unwrap().text_range().start().into();
                let current_node_len: usize =
                    op_type.ast_node(db).unwrap().text_range().len().into();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    name: name.clone(),
                    ty: "root operation type definition".into(),
                    src: db.input(),
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
