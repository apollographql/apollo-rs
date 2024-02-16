use crate::ast;
use crate::schema::ExtendedType;
use crate::schema::ObjectType;
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::field::validate_field_definitions;
use crate::Node;

pub(crate) fn validate_object_type_definitions(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    for ty in schema.types.values() {
        if let ExtendedType::Object(object) = ty {
            diagnostics.extend(validate_object_type_definition(schema, object))
        }
    }

    diagnostics
}

pub(crate) fn validate_object_type_definition(
    schema: &crate::Schema,
    object: &Node<ObjectType>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(super::directive::validate_directives(
        Some(schema),
        object.directives.iter_ast(),
        ast::DirectiveLocation::Object,
        // objects don't use variables
        Default::default(),
    ));

    // Object Type field validations.
    diagnostics.extend(validate_field_definitions(schema, &object.fields));

    // Implements Interfaces validation.
    diagnostics.extend(super::interface::validate_implements_interfaces(
        schema,
        &object.name,
        object.location(),
        &object.implements_interfaces,
    ));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    for implements_interface in &object.implements_interfaces {
        if let Some(interface) = schema.get_interface(implements_interface) {
            for interface_field in interface.fields.values() {
                if object.fields.contains_key(&interface_field.name) {
                    continue;
                }

                diagnostics.push(ValidationError::new(
                    object.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: object.name.clone(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.name.clone(),
                        field: interface_field.name.clone(),
                        field_location: interface_field.location(),
                    },
                ));
            }
        }
    }

    diagnostics
}
