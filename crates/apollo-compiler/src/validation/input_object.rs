use std::collections::HashMap;

use crate::{
    diagnostics::{UniqueDefinition, UniqueField},
    hir::{InputObjectTypeDefinition, InputValueDefinition},
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &InputObjectTypeDefinition> = HashMap::new();
    for input_object in db.input_objects().iter() {
        let name = input_object.name();
        if let Some(prev_def) = seen.get(name) {
            let prev_offset = prev_def.loc().offset();
            let prev_node_len = prev_def.loc().node_len();

            let current_offset = input_object.loc().offset();
            let current_node_len = input_object.loc().node_len();
            diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                ty: "input object".into(),
                name: name.into(),
                src: db.source_code(prev_def.loc().file_id()),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!(
                    "`{name}` must only be defined once in this document."
                )),
            }));
        } else {
            seen.insert(name, input_object);
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Value error.
    for input_objects in db.input_objects().iter() {
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
