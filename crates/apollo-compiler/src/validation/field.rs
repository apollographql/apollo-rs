use std::sync::Arc;

use crate::{
    hir,
    validation::{directive, ValidationDatabase},
    ApolloDiagnostic,
};

pub fn validate(db: &dyn ValidationDatabase, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate_usage(
        db,
        field.directives().to_vec(),
        hir::DirectiveLocation::Field,
    ));
    diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));

    diagnostics
}
