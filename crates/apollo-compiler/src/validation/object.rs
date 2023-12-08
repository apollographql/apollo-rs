use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    ValidationDatabase,
};
use std::collections::HashSet;

pub(crate) fn validate_object_type_definitions(
    db: &dyn ValidationDatabase,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = vec![];

    let defs = &db.ast_types().objects;
    for def in defs.values() {
        diagnostics.extend(db.validate_object_type_definition(def.clone()))
    }

    diagnostics
}

pub(crate) fn validate_object_type_definition(
    db: &dyn ValidationDatabase,
    object: ast::TypeWithExtensions<ast::ObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    diagnostics.extend(super::directive::validate_directives(
        db,
        object.directives(),
        ast::DirectiveLocation::Object,
        // objects don't use variables
        Default::default(),
    ));

    // Collect all fields, including duplicates
    let field_definitions: Vec<_> = object.fields().cloned().collect();
    let field_names: HashSet<_> = field_definitions
        .iter()
        .map(|field| field.name.clone())
        .collect();

    // Object Type field validations.
    diagnostics.extend(db.validate_field_definitions(field_definitions));

    // Implements Interfaces validation.
    let implements_interfaces: Vec<_> = object.implements_interfaces().cloned().collect();
    diagnostics.extend(super::interface::validate_implements_interfaces(
        db,
        &object.definition.clone().into(),
        &implements_interfaces,
    ));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    for implements_interface in object.implements_interfaces() {
        if let Some(interface) = schema.get_interface(implements_interface) {
            for interface_field in interface.fields.values() {
                if field_names.contains(&interface_field.name) {
                    continue;
                }

                let mut labels = vec![
                    Label::new(
                        implements_interface.location(),
                        format!("implementation of interface {implements_interface} declared here"),
                    ),
                    Label::new(
                        object.definition.location(),
                        format!("add `{}` field to this object", interface_field.name),
                    ),
                ];
                let loc = interface_field.location();
                labels.push(Label::new(
                    loc,
                    format!(
                        "`{}` was originally defined by {} here",
                        interface_field.name, implements_interface
                    ),
                ));
                diagnostics.push(ApolloDiagnostic::new(
                    db,
                    object.definition.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: interface.name.to_string(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.to_string(),
                        field: interface_field.name.to_string(),
                        field_location: interface_field.location(),
                    },
                ));
            }
        }
    }

    diagnostics
}
