use super::BuildError;
use crate::validation::Details;
use crate::validation::Diagnostics;
use crate::ExecutableDocument;
use crate::Schema;

pub(crate) fn validate_executable_document(
    errors: &mut Diagnostics,
    _schema: &Schema,
    document: &ExecutableDocument,
) {
    validate_standalone_executable(errors, document)
    // TODO
}

pub(crate) fn validate_standalone_executable(
    errors: &mut Diagnostics,
    document: &ExecutableDocument,
) {
    if let Some((file_id, source)) = &document.source {
        source.validate_parse_errors(errors, *file_id)
    }
    for build_error in &document.build_errors {
        validate_build_error(errors, build_error)
    }
    // TODO
}

fn validate_build_error(errors: &mut Diagnostics, build_error: &BuildError) {
    let location = match build_error {
        BuildError::TypeSystemDefinition { location, .. } => *location,
        _ => return, // TODO
    };
    errors.push(location, Details::ExecutableBuildError(build_error.clone()))
}
