use super::ExtendedType;
use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definition;
use crate::validation::input_object::validate_input_object_definition;
use crate::validation::interface::validate_interface_definition;
use crate::validation::object::validate_object_type_definition;
use crate::validation::scalar::validate_scalar_definition;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definition;
use crate::validation::DiagnosticList;
use crate::Schema;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &mut Schema) {
    validate_schema_definition(errors, schema);
    validate_directive_definitions(errors, schema);
    for def in schema.types.values() {
        match def {
            ExtendedType::Scalar(def) => validate_scalar_definition(errors, schema, def),
            ExtendedType::Object(def) => validate_object_type_definition(errors, schema, def),
            ExtendedType::Interface(def) => validate_interface_definition(errors, schema, def),
            ExtendedType::Union(def) => validate_union_definition(errors, schema, def),
            ExtendedType::Enum(def) => validate_enum_definition(errors, schema, def),
            ExtendedType::InputObject(def) => validate_input_object_definition(errors, schema, def),
        }
    }
}
