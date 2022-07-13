use apollo_parser::SyntaxNode;

use crate::{ApolloDiagnostic, SourceDatabase};

// schema
pub mod schema;

// leaf nodes
pub mod enums;
pub mod scalars;
pub mod unions;

// composite nodes
pub mod directives;
pub mod input_object;
pub mod interfaces;

// executable definitions
pub mod operations;

pub mod unused_variables;

pub struct Validator<'a> {
    db: &'a dyn SourceDatabase,
    diagnostics: Vec<ApolloDiagnostic>,
}

impl<'a> Validator<'a> {
    pub fn new(db: &'a dyn SourceDatabase) -> Self {
        Self {
            db,
            diagnostics: Vec::new(),
        }
    }

    pub fn validate(&mut self) -> &mut [ApolloDiagnostic] {
        self.diagnostics.extend(self.db.syntax_errors());

        self.diagnostics.extend(schema::check(self.db));

        self.diagnostics.extend(scalars::check(self.db));
        self.diagnostics.extend(enums::check(self.db));
        self.diagnostics.extend(unions::check(self.db));

        self.diagnostics.extend(interfaces::check(self.db));
        self.diagnostics.extend(directives::check(self.db));
        self.diagnostics.extend(input_object::check(self.db));

        self.diagnostics.extend(operations::check(self.db));
        self.diagnostics.extend(unused_variables::check(self.db));

        self.diagnostics.as_mut()
    }
}

#[derive(Debug, Eq)]
struct ValidationSet {
    name: String,
    node: SyntaxNode,
}

impl std::hash::Hash for ValidationSet {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl PartialEq for ValidationSet {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
