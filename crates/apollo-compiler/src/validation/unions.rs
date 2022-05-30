use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // A Union type must include one or more unique member types.
    //
    // Return a Unique Value error in case of a duplicate member.
    for union_def in db.unions().iter() {
        let mut seen = HashSet::new();
        for union_member in union_def.union_members().iter() {
            let member = union_member.name();
            if seen.contains(&member) {
                errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueValue {
                    message: "Union member types must be unique".into(),
                    value: member.into(),
                }));
            } else {
                seen.insert(member);
            }
        }
    }

    errors
}
