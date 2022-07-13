use std::collections::HashMap;

use crate::{
    diagnostics::{UniqueDefinition, UniqueField},
    values::{InputObjectDefinition, InputValueDefinition},
    ApolloDiagnostic, SourceDatabase,
};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &InputObjectDefinition> = HashMap::new();
    for input_object in db.input_objects().iter() {
        let name = input_object.name();
        if let Some(prev_def) = seen.get(name) {
            let prev_offset: usize = prev_def.ast_node(db).text_range().start().into();
            let prev_node_len: usize = prev_def.ast_node(db).text_range().len().into();

            let current_offset: usize = input_object.ast_node(db).text_range().start().into();
            let current_node_len: usize = input_object.ast_node(db).text_range().len().into();
            errors.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                ty: "input object".into(),
                name: name.into(),
                src: db.input_string(()).to_string(),
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
                if prev_field.ast_node(db).is_some() && field.ast_node(db).is_some() {
                    let prev_offset: usize =
                        prev_field.ast_node(db).unwrap().text_range().start().into();
                    let prev_node_len: usize =
                        prev_field.ast_node(db).unwrap().text_range().len().into();

                    let current_offset: usize =
                        field.ast_node(db).unwrap().text_range().start().into();
                    let current_node_len: usize =
                        field.ast_node(db).unwrap().text_range().len().into();

                    errors.push(ApolloDiagnostic::UniqueField(UniqueField {
                        field: field_name.into(),
                        src: db.input_string(()).to_string(),
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

    errors
}
