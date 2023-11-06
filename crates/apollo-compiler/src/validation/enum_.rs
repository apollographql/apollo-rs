use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    Node, ValidationDatabase,
};

pub(crate) fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for enum_ in db.ast_types().enums.values() {
        diagnostics.extend(db.validate_enum_definition(enum_.clone()));
    }

    diagnostics
}

pub(crate) fn validate_enum_definition(
    db: &dyn ValidationDatabase,
    enum_def: ast::TypeWithExtensions<ast::EnumTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
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
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives(
        db,
        enum_val.directives.iter(),
        ast::DirectiveLocation::EnumValue,
        // enum values don't use variables
        Default::default(),
    );

    // (convention) Values in an Enum Definition should be capitalized.
    //
    // Return a Capitalized Value warning if enum value is not capitalized.
    if enum_val.value != enum_val.value.to_uppercase().as_str() {
        let location = enum_val.location();
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                location,
                DiagnosticData::CapitalizedValue {
                    value: enum_val.value.to_string(),
                },
            )
            .label(Label::new(
                location,
                format!("consider capitalizing {}", enum_val.value),
            )),
        );
    }

    diagnostics
}
