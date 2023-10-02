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
    // TODO
}
