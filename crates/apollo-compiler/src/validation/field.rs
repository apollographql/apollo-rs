use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{OutputType, UndefinedDefinition, UniqueField},
    hir,
    validation::ValidationDatabase,
    ApolloDiagnostic,
};

pub fn validate_field(
    db: &dyn ValidationDatabase,
    field: Arc<hir::Field>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics =
        db.validate_directives(field.directives().to_vec(), hir::DirectiveLocation::Field);
    diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));

    diagnostics
}

pub fn validate_field_definition(
    db: &dyn ValidationDatabase,
    field: hir::FieldDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        field.directives().to_vec(),
        hir::DirectiveLocation::FieldDefinition,
    );

    diagnostics.extend(db.validate_arguments_definition(
        field.arguments,
        hir::DirectiveLocation::ArgumentDefinition,
    ));

    diagnostics
}

pub fn validate_field_definitions(
    db: &dyn ValidationDatabase,
    fields: Vec<hir::FieldDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen: HashMap<&str, &hir::FieldDefinition> = HashMap::new();

    for field in fields.iter() {
        diagnostics.extend(db.validate_field_definition(field.clone()));

        // Fields must be unique.
        //
        // Returns Unique Field error.
        let field_name = field.name();
        let offset = field.loc().offset();
        let len = field.loc().node_len();

        if let Some(prev_field) = seen.get(field_name) {
            let prev_offset = prev_field.loc().offset();
            let prev_node_len = prev_field.loc().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueField(UniqueField {
                field: field_name.into(),
                src: db.source_code(prev_field.loc().file_id()),
                original_field: (prev_offset, prev_node_len).into(),
                redefined_field: (offset, len).into(),
                help: Some(format!(
                    "`{field_name}` field must only be defined once in this definition."
                )),
            }));
        } else {
            seen.insert(field_name, field);
        }

        // Field types in Object Types must be of output type
        if let Some(field_ty) = field.ty().type_def(db.upcast()) {
            if !field.ty().is_output_type(db.upcast()) {
                diagnostics.push(ApolloDiagnostic::OutputType(OutputType {
                    name: field.name().into(),
                    ty: field_ty.kind(),
                    src: db.source_code(field.loc().file_id()),
                    definition: (offset, len).into(),
                }))
            }
        } else if let Some(loc) = field.ty().loc() {
            let field_ty_offset = loc.offset();
            let field_ty_len = loc.node_len();
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: field.ty().name(),
                src: db.source_code(field.loc().file_id()),
                definition: (field_ty_offset, field_ty_len).into(),
            }))
        } else {
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: field.ty().name(),
                src: db.source_code(field.loc().file_id()),
                definition: (offset, len).into(),
            }))
        }
    }

    diagnostics
}
