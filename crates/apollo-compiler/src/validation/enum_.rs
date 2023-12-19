use crate::validation::diagnostics::ValidationError;
use crate::{ast, Node, ValidationDatabase};

pub(crate) fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for enum_ in db.ast_types().enums.values() {
        diagnostics.extend(db.validate_enum_definition(enum_.clone()));
    }

    diagnostics
}

pub(crate) fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: ast::TypeWithExtensions<ast::EnumTypeDefinition>,
) -> Vec<ValidationError> {
    let mut diagnostics = super::directive::validate_directives(
        db,
        enum_def.directives(),
        ast::DirectiveLocation::Enum,
        // enums don't use variables
        Default::default(),
    );

    for enum_val in enum_def.values() {
        diagnostics.extend(validate_enum_value(db, enum_val));
    }

    diagnostics
}

pub(crate) fn validate_enum_value(
    db: &dyn ValidationDatabase,
    enum_val: &Node<ast::EnumValueDefinition>,
) -> Vec<ValidationError> {
    super::directive::validate_directives(
        db,
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
    )
}
