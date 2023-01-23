use crate::{
    hir,
    validation::{directive, ValidationDatabase},
    ApolloDiagnostic,
};

pub fn validate(db: &dyn ValidationDatabase, field: hir::FieldDefinition) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate_usage(
        db,
        field.directives().to_vec(),
        hir::DirectiveLocation::FieldDefinition,
    ));

    diagnostics.extend(db.validate_arguments_definition(field.arguments));

    diagnostics
}
