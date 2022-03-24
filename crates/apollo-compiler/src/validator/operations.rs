use crate::{ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();
    if db.operations().len() > 1 {
        let missing_ident: Vec<ApolloDiagnostic> = db
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
        errors.extend(missing_ident);
    }

    if db.subscriptions().len() >= 1 {
        // let single_root_field: Vec<ApolloDiagnostic> =
        // db.subscriptions().iter().filter_map(|op| {
        //     // if op.fields() > 1 || op.fragment_spread().fields() > 1 || op.inline_fragment().fields().
        // }).collect();
    }

    errors
}
