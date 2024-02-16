use crate::validation::Details;
use crate::validation::DiagnosticList;
use crate::Schema;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &Schema) {
    compiler_validation(errors, schema)
}

/// TODO: replace this with validation based on `Schema` without a database
fn compiler_validation(errors: &mut DiagnosticList, schema: &Schema) {
    for diagnostic in crate::validation::validate_type_system(schema) {
        errors.push(diagnostic.location, Details::CompilerDiagnostic(diagnostic))
    }
}
