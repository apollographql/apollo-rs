use crate::{ApolloDiagnostic, SourceDatabase};

pub mod operations;
pub mod schema;
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
        self.errors.extend(self.db.syntax_errors());
        self.errors.extend(schema::check(self.db));
        self.errors.extend(operations::check(self.db));
        self.errors.extend(unused_variables::check(self.db));
        self.errors.as_mut()
    }
}
