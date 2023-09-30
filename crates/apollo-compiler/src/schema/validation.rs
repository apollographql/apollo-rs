use crate::validation::Diagnostics;
use crate::Schema;

pub(crate) fn validate_schema(errors: &mut Diagnostics, schema: &Schema) {
    for (&file_id, source) in &schema.sources {
        source.validate_parse_errors(errors, file_id)
    }
    // TODO
}
