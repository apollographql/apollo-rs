use crate::{ApolloDiagnostic, SourceDatabase};

// schema
pub mod schema;

// leaf nodes
pub mod enums;
pub mod scalars;
pub mod unions;

// composite nodes
pub mod directives;
pub mod interfaces;

// executable definitions
pub mod operations;

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

        self.errors.extend(scalars::check(self.db));
        self.errors.extend(enums::check(self.db));
        self.errors.extend(unions::check(self.db));

        self.errors.extend(interfaces::check(self.db));
        self.errors.extend(directives::check(self.db));

        self.errors.extend(operations::check(self.db));
        self.errors.extend(unused_variables::check(self.db));

        self.errors.as_mut()
    }
}
