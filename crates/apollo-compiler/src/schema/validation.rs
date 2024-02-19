use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definitions;
use crate::validation::input_object::validate_input_object_definitions;
use crate::validation::interface::validate_interface_definitions;
use crate::validation::object::validate_object_type_definitions;
use crate::validation::scalar::validate_scalar_definitions;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definitions;
use crate::validation::DiagnosticList;
use crate::Schema;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &Schema) {
    validate_schema_definition(errors, schema);
    validate_scalar_definitions(errors, schema);
    validate_enum_definitions(errors, schema);
    validate_union_definitions(errors, schema);
    validate_interface_definitions(errors, schema);
    validate_directive_definitions(errors, schema);
    validate_input_object_definitions(errors, schema);
    validate_object_type_definitions(errors, schema);
}
