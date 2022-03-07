use crate::{values, SourceDatabase};

pub mod unused_implements_interfaces;
pub mod unused_variables;

pub struct Validator<'a> {
    db: &'a dyn SourceDatabase,
    errors: Vec<values::Error>,
}

impl<'a> Validator<'a> {
    pub fn new(db: &'a dyn SourceDatabase) -> Self {
        Self {
            db,
            errors: Vec::new(),
        }
    }

    pub fn validate(&self) -> Vec<values::Error> {
        todo!()
    }
}
