use crate::schema::{EnumType, ExtendedType};
use crate::validation::diagnostics::ValidationError;
use crate::{ast, Node};

pub(crate) fn validate_enum_definitions(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for ty in schema.types.values() {
        if let ExtendedType::Enum(enum_) = ty {
            diagnostics.extend(validate_enum_definition(schema, enum_));
        }
    }

    diagnostics
}

pub(crate) fn validate_enum_definition(
    schema: &crate::Schema,
    enum_def: &EnumType,
) -> Vec<ValidationError> {
    let mut diagnostics = super::directive::validate_directives(
        Some(schema),
        enum_def.directives.iter_ast(),
        ast::DirectiveLocation::Enum,
        // enums don't use variables
        Default::default(),
    );

    for enum_val in enum_def.values.values() {
        diagnostics.extend(validate_enum_value(schema, enum_val));
    }

    diagnostics
}

pub(crate) fn validate_enum_value(
    schema: &crate::Schema,
    enum_val: &Node<ast::EnumValueDefinition>,
) -> Vec<ValidationError> {
    super::directive::validate_directives(
        Some(schema),
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
    )
}
