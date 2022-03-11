use crate::{ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) {
    if db.operations().len() > 1 {
        let operations_with_missing_ident: Vec<ApolloDiagnostic> = db
            .operations()
            .iter()
            .filter_map(|op| {
                if op.name().is_none() {
                    Some(ApolloDiagnostic::MissingIdent(
                        "Missing operation name".into(),
                    ));
                }
                None
            })
            .collect();
    }
}
