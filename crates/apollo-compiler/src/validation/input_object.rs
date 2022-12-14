use std::collections::HashMap;

use crate::{
    diagnostics::{UniqueDefinition, UniqueField},
    hir::InputValueDefinition,
    validation::type_definitions,
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.input_objects();
    for (file_id, ast_def) in type_definitions::<ast::InputObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let ast_loc = (file_id, &ast_def).into();
            if *hir_def.loc() == ast_loc {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "input object".into(),
                    name: name.to_owned(),
                    src: db.source_code(hir_def.loc().file_id()),
                    original_definition: hir_def.loc().into(),
                    redefined_definition: ast_loc.into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            }
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Value error.
    for input_objects in db.input_objects().values() {
        let mut seen: HashMap<&str, &InputValueDefinition> = HashMap::new();

        let input_fields = input_objects.input_fields_definition();

        for field in input_fields {
            let field_name = field.name();
            if let Some(prev_field) = seen.get(&field_name) {
                if prev_field.loc().is_some() && field.loc().is_some() {
                    let prev_offset = prev_field.loc().unwrap().offset();
                    let prev_node_len = prev_field.loc().unwrap().node_len();

                    let current_offset = field.loc().unwrap().offset();
                    let current_node_len = field.loc().unwrap().node_len();

                    diagnostics.push(ApolloDiagnostic::UniqueField(UniqueField {
                        field: field_name.into(),
                        src: db.source_code(prev_field.loc().unwrap().file_id()),
                        original_field: (prev_offset, prev_node_len).into(),
                        redefined_field: (current_offset, current_node_len).into(),
                        help: Some(format!(
                            "{field_name} field must only be defined once in this input object definition."
                        )),
                    }));
                }
            } else {
                seen.insert(field_name, field);
            }
        }
    }

    diagnostics
}
