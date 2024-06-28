use crate::ast;
use crate::schema::EnumType;
use crate::schema::ExtendedType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;
use crate::Node;

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
    enum_def: &Node<EnumType>,
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

    // validate there is at least one enum value on the enum type
    // https://spec.graphql.org/draft/#sel-DAHfFVFBAAEXBAAh7S
    if enum_def.values.is_empty() {
        diagnostics.push(
            enum_def.location(),
            DiagnosticData::EmptyValueSet {
                type_name: enum_def.name.clone(),
                type_location: enum_def.location(),
                extensions_locations: enum_def
                    .extensions()
                    .iter()
                    .map(|ext| ext.location())
                    .collect(),
            },
        );
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
