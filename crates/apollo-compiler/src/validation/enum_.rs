use crate::schema::{EnumType, ExtendedType};
use crate::validation::DiagnosticList;
use crate::{ast, Node};

pub(crate) fn validate_enum_definitions(diagnostics: &mut DiagnosticList, schema: &crate::Schema) {
    for ty in schema.types.values() {
        if let ExtendedType::Enum(enum_) = ty {
            validate_enum_definition(diagnostics, schema, enum_);
        }
    }
}

pub(crate) fn validate_enum_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    enum_def: &EnumType,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        enum_def.directives.iter_ast(),
        ast::DirectiveLocation::Enum,
        // enums don't use variables
        Default::default(),
    );

    for enum_val in enum_def.values.values() {
        validate_enum_value(diagnostics, schema, enum_val);
    }
}

pub(crate) fn validate_enum_value(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    enum_val: &Node<ast::EnumValueDefinition>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
    )
}
