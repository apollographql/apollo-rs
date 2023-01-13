use std::collections::HashMap;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::InputValueDefinition,
    validation::ast_type_definitions,
    ValidationDatabase,
};
use apollo_parser::ast;

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Input Object Definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.input_objects();
    for (file_id, ast_def) in ast_type_definitions::<ast::InputObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let original_definition = hir_def.loc();
            let redefined_definition = (file_id, &ast_def).into();
            if original_definition == redefined_definition {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        original_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "input object",
                            name: name.to_owned(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    ))
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{name}` redefined here")),
                    ]),
                );
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
                if let (Some(original_definition), Some(redefined_definition)) =
                    (prev_field.loc(), field.loc())
                {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db, original_definition.into(),
                            DiagnosticData::UniqueField {
                                field: field_name.into(),
                                original_definition: original_definition.into(),
                                redefined_definition: redefined_definition.into(),
                            }
                        )
                        .labels([
                            Label::new(original_definition, format!("previous definition of `{field_name}` here")),
                            Label::new(redefined_definition, format!("`{field_name}` redefined here")),
                        ])
                        .help(format!("`{field_name}` field must only be defined once in this input object definition."))
                    );
                }
            } else {
                seen.insert(field_name, field);
            }
        }
    }

    diagnostics
}
