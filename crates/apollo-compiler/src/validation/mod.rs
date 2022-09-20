pub mod validation;

// schema
mod schema;

// leaf nodes
mod enums;
mod scalars;
mod unions;

// composite nodes
mod directives;
mod input_objects;
mod interfaces;
mod objects;

// executable definitions
mod operations;

mod unused_variables;

use std::sync::Arc;

use apollo_parser::SyntaxNode;

use crate::{diagnostics, ApolloDiagnostic, Definitions, Document, DocumentParser, Inputs};

#[salsa::query_group(ValidationStorage)]
pub trait Validation: Document + Inputs + DocumentParser + Definitions {
    fn validate(&self) -> Arc<Vec<ApolloDiagnostic>>;
}

pub fn validate(db: &dyn Validation) -> Arc<Vec<ApolloDiagnostic>> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(schema::check(db));

    Arc::new(diagnostics)
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
