use crate::{values::Field, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();
    // It is possible to have an unnamed (anonymous) operation definition only
    // if there is **one** operation definition.
    //
    // Return a Missing Indent error if there are multiple operations and one or
    // more are missing a name.
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

    // Operation definitions must have unique names.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let duplicate_operations: Vec<ApolloDiagnostic> = db
        .operations()
        .iter()
        .filter_map(|op| {
            if let Some(name) = op.name() {
                if let Some(_dup) = db.operations().iter().find(|dup| dup.name() == op.name()) {
                    return Some(ApolloDiagnostic::UniqueOperationDefinition {
                        message: "Operation Definitions must have unique names".into(),
                        operation: name,
                    });
                }
            }
            None
        })
        .collect();

    errors.extend(duplicate_operations);

    // A Subscription operation definition can only have **one** root level
    // field.
    if db.subscriptions().len() >= 1 {
        let single_root_field: Vec<ApolloDiagnostic> = db
            .subscriptions()
            .iter()
            .filter_map(|op| {
                let mut top_level_fields: Vec<Field> = op.fields(db)?.as_ref().clone();
                top_level_fields.extend(op.fields_in_inline_fragments(db)?.as_ref().clone());
                top_level_fields.extend(op.fields_in_fragment_spread(db)?.as_ref().clone());
                if top_level_fields.len() > 1 {
                    Some(ApolloDiagnostic::SingleRootField(
                        "Subscription operations can only have one root field {}".into(),
                    ))
                } else {
                    None
                }
            })
            .collect();
        errors.extend(single_root_field);
    }

    errors
}
