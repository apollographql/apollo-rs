use crate::{ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();
    if db.operations().len() > 1 {
        let operations_with_missing_ident: Vec<ApolloDiagnostic> = db
            .operations()
            .iter()
            .filter_map(|op| {
                if op.name().is_none() {
                    return Some(ApolloDiagnostic::MissingIdent(
                        "Missing operation name".into(),
                    ));
                }
                None
            })
            .collect();
        errors.extend(operations_with_missing_ident);
    }
    errors
}
