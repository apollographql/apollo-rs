// schema
mod schema;

// leaf nodes
mod enum_;
mod scalar;
mod union_;

// composite nodes
mod directive;
mod input_object;
mod interface;
mod object;

// executable definitions
mod operation;

mod arguments;
mod unused_variable;

use apollo_parser::SyntaxNode;

use crate::{
    database::db::Upcast, ApolloDiagnostic, AstDatabase, DocumentDatabase, HirDatabase,
    InputDatabase,
};

#[salsa::query_group(ValidationStorage)]
pub trait ValidationDatabase:
    Upcast<dyn DocumentDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    fn validate(&self) -> Vec<ApolloDiagnostic>;
    fn validate_schema(&self) -> Vec<ApolloDiagnostic>;
    fn validate_scalar(&self) -> Vec<ApolloDiagnostic>;
    fn validate_enum(&self) -> Vec<ApolloDiagnostic>;
    fn validate_union(&self) -> Vec<ApolloDiagnostic>;
    fn validate_interface(&self) -> Vec<ApolloDiagnostic>;
    fn validate_directive(&self) -> Vec<ApolloDiagnostic>;
    fn validate_input_object(&self) -> Vec<ApolloDiagnostic>;
    fn validate_object(&self) -> Vec<ApolloDiagnostic>;
    fn validate_operation(&self) -> Vec<ApolloDiagnostic>;
    fn validate_arguments(&self) -> Vec<ApolloDiagnostic>;
    fn validate_unused_variable(&self) -> Vec<ApolloDiagnostic>;
}

pub fn validate(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(db.syntax_errors());

    diagnostics.extend(db.validate_schema());

    diagnostics.extend(db.validate_scalar());
    diagnostics.extend(db.validate_enum());
    diagnostics.extend(db.validate_union());

    diagnostics.extend(db.validate_interface());
    diagnostics.extend(db.validate_directive());
    diagnostics.extend(db.validate_input_object());
    diagnostics.extend(db.validate_object());
    diagnostics.extend(db.validate_operation());

    diagnostics.extend(db.validate_arguments());
    diagnostics.extend(db.validate_unused_variable());

    diagnostics
}

pub fn validate_schema(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    schema::check(db)
}

pub fn validate_scalar(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    scalar::check(db)
}

pub fn validate_enum(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    enum_::check(db)
}

pub fn validate_union(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    union_::check(db)
}

pub fn validate_interface(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    interface::check(db)
}

pub fn validate_directive(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    directive::check(db)
}

pub fn validate_input_object(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    input_object::check(db)
}

pub fn validate_object(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    object::check(db)
}

pub fn validate_operation(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    operation::check(db)
}

pub fn validate_arguments(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    arguments::check(db)
}

pub fn validate_unused_variable(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    unused_variable::check(db)
}

// #[salsa::query_group(ValidationStorage)]
// pub trait Validation: Document + Inputs + DocumentParser + Definitions {
//     fn validate(&self) -> Arc<Vec<ApolloDiagnostic>>;
// }
//
// pub fn validate(db: &dyn Validation) -> Arc<Vec<ApolloDiagnostic>> {
//     let mut diagnostics = Vec::new();
//     diagnostics.extend(schema::check(db));
//
//     Arc::new(diagnostics)
// }

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
