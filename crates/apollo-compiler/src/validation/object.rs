use crate::schema::ExtendedType;
use crate::schema::ObjectType;
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::field::validate_field_definitions;
use crate::Node;
use crate::{ast, ValidationDatabase};

pub(crate) fn validate_object_type_definitions(
    db: &dyn ValidationDatabase,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    for ty in db.schema().types.values() {
        if let ExtendedType::Object(object) = ty {
            diagnostics.extend(validate_object_type_definition(db, object))
        }
    }

    diagnostics
}

pub(crate) fn validate_object_type_definition(
    db: &dyn ValidationDatabase,
    object: &Node<ObjectType>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let has_schema = true;
    diagnostics.extend(super::directive::validate_directives(
        db,
        object.directives.iter_ast(),
        ast::DirectiveLocation::Object,
        // objects don't use variables
        Default::default(),
        has_schema,
    ));

    // Object Type field validations.
    diagnostics.extend(validate_field_definitions(db, &object.fields));

    // Implements Interfaces validation.
    diagnostics.extend(super::interface::validate_implements_interfaces(
        db,
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
