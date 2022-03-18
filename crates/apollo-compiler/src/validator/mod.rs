use crate::{ApolloDiagnostic, SourceDatabase};

pub mod operation_name;
pub mod unused_implements_interfaces;
pub mod unused_variables;

pub struct Validator<'a> {
    db: &'a dyn SourceDatabase,
    errors: Vec<ApolloDiagnostic>,
}

impl<'a> Validator<'a> {
    pub fn new(db: &'a dyn SourceDatabase) -> Self {
        Self {
            db,
            errors: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> &mut [ApolloDiagnostic] {
        self.extend_errors(operation_name::check(self.db));
        self.extend_errors(unused_variables::check(self.db));
        self.errors.as_mut()
    }

    /// Set the validator's errors.
    pub fn extend_errors(&mut self, errors: Vec<ApolloDiagnostic>) {
        self.errors.extend(errors);
    }
}
