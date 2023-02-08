use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationDatabase,
};

pub fn validate_field(
    db: &dyn ValidationDatabase,
    field: Arc<hir::Field>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics =
        db.validate_directives(field.directives().to_vec(), hir::DirectiveLocation::Field);

    if !field.arguments().is_empty() {
        diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));
    }

    if let Some(field_definition) = field.field_definition(db.upcast()) {
        diagnostics.extend(db.validate_argument_types(
            field_definition.arguments().clone(),
            field.arguments().to_vec(),
        ));
    }

    let field_type = field.ty(db.upcast());
    if field_type.is_some() {
        diagnostics.extend(db.validate_selection_set(field.selection_set().clone()));
    } else {
        let help = format!(
            "`{}` is not defined on `{}` type",
            field.name(),
            field.parent_obj.as_ref().unwrap(),
        );
        let field_name = field.name();
        let diagnostic = ApolloDiagnostic::new(
            db,
            field.loc().into(),
            DiagnosticData::UndefinedField {
                field: field_name.into(),
            },
        )
        .label(Label::new(
            field.loc(),
            format!("`{field_name}` field is not defined"),
        ))
        .help(help);

        let parent_type_loc = db
            .find_type_definition_by_name(field.parent_obj.clone().unwrap())
            .and_then(|type_def| type_def.loc());

        let diagnostic = if let Some(parent_type_loc) = parent_type_loc {
            diagnostic.label(Label::new(
                parent_type_loc,
                format!("`{}` declared here", field.parent_obj.as_ref().unwrap()),
            ))
        } else {
            diagnostic
        };
        diagnostics.push(diagnostic);
    }

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
        let redefined_definition = field.loc();

        if let Some(prev_field) = seen.get(field_name) {
            let original_definition = prev_field.loc();

            diagnostics.push(
                ApolloDiagnostic::new(
                    db, original_definition.into(),
                    DiagnosticData::UniqueField {
                        field: field_name.into(),
                        original_definition: original_definition.into(),
                        redefined_definition: redefined_definition.into(),
                    },
                )
                    .labels([
                        Label::new(original_definition, format!("previous definition of `{field_name}` here")),
                        Label::new(redefined_definition, format!("`{field_name}` redefined here")),
                    ])
                    .help(format!("`{field_name}` field must only be defined once in this input object definition."))
            );
        } else {
            seen.insert(field_name, field);
        }

        // Field types in Object Types must be of output type
        if let Some(field_ty) = field.ty().type_def(db.upcast()) {
            if !field.ty().is_output_type(db.upcast()) {
                diagnostics.push(
                    ApolloDiagnostic::new(db, field.loc().into(), DiagnosticData::OutputType {
                        name: field.name().into(),
                        ty: field_ty.kind(),
                    })
                        .label(Label::new(field.loc(), format!("this is of `{}` type", field_ty.kind())))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name())),
                );
            }
        } else if let Some(field_ty_loc) = field.ty().loc() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field_ty_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: field.name().into(),
                    },
                )
                .label(Label::new(field_ty_loc, "not found in this scope")),
            );
        } else {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field.loc().into(),
                    DiagnosticData::UndefinedDefinition {
                        name: field.ty().name(),
                    },
                )
                .label(Label::new(field.loc(), "not found in this scope")),
            );
        }
    }

    diagnostics
}
