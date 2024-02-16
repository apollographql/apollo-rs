use crate::schema::{EnumType, ExtendedType};
use crate::validation::diagnostics::ValidationError;
use crate::{ast, Node, ValidationDatabase};

pub(crate) fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for ty in db.schema().types.values() {
        if let ExtendedType::Enum(enum_) = ty {
            diagnostics.extend(validate_enum_definition(db, enum_));
        }
    }

    diagnostics
}

pub(crate) fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: &EnumType,
) -> Vec<ValidationError> {
    let has_schema = true;
    let mut diagnostics = super::directive::validate_directives(
        db,
        enum_def.directives.iter_ast(),
        ast::DirectiveLocation::Enum,
        // enums don't use variables
        Default::default(),
        has_schema,
    );

    for enum_val in enum_def.values.values() {
        diagnostics.extend(validate_enum_value(db, enum_val));
    }

    diagnostics
}

pub(crate) fn validate_enum_value(
    db: &dyn ValidationDatabase,
    enum_val: &Node<ast::EnumValueDefinition>,
) -> Vec<ValidationError> {
    let has_schema = true;
    super::directive::validate_directives(
        db,
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
        has_schema,
    )
}
