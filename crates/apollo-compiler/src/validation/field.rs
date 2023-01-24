use crate::{hir, validation::ValidationDatabase, ApolloDiagnostic};

pub fn validate_field(db: &dyn ValidationDatabase, field: hir::Field) -> Vec<ApolloDiagnostic> {
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
