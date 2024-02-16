use crate::schema::{ComponentName, ExtendedType, InterfaceType, Name};
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::field::validate_field_definitions;
use crate::{ast, Node, NodeLocation, ValidationDatabase};
use indexmap::IndexSet;

pub(crate) fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();
    for ty in db.schema().types.values() {
        if let ExtendedType::Interface(interface) = ty {
            diagnostics.extend(validate_interface_definition(db, interface));
        }
    }

    diagnostics
}

pub(crate) fn validate_interface_definition(
    db: &dyn ValidationDatabase,
    interface: &Node<InterfaceType>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let has_schema = true;
    diagnostics.extend(super::directive::validate_directives(
        db,
        interface.directives.iter_ast(),
        ast::DirectiveLocation::Interface,
        // interfaces don't use variables
        Default::default(),
        has_schema,
    ));

    // Interface must not implement itself.
    //
    // Return Recursive Definition error.
    //
    // NOTE(@lrlna): we should also check for more sneaky cyclic references for interfaces like this, for example:
    //
    // interface Node implements Named & Node {
    //   id: ID!
    //   name: String
    // }
    //
    // interface Named implements Node & Named {
    //   id: ID!
    //   name: String
    // }
    for implements_interface in &interface.implements_interfaces {
        if *implements_interface == interface.name {
            diagnostics.push(ValidationError::new(
                implements_interface.location(),
                DiagnosticData::RecursiveInterfaceDefinition {
                    name: implements_interface.name.clone(),
                },
            ));
        }
    }

    // Interface Type field validation.
    diagnostics.extend(validate_field_definitions(db, &interface.fields));

    // Implements Interfaceds validation.
    diagnostics.extend(validate_implements_interfaces(
        db,
        &interface.name,
        interface.location(),
        &interface.implements_interfaces,
    ));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    for implements_interface in &interface.implements_interfaces {
        if let Some(super_interface) = schema.get_interface(implements_interface) {
            for super_field in super_interface.fields.values() {
                if interface.fields.contains_key(&super_field.name) {
                    continue;
                }
                diagnostics.push(ValidationError::new(
                    interface.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: interface.name.clone(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.name.clone(),
                        field: super_field.name.clone(),
                        field_location: super_field.location(),
                    },
                ));
            }
        }
    }

    diagnostics
}

pub(crate) fn validate_implements_interfaces(
    db: &dyn ValidationDatabase,
    implementor_name: &Name,
    implementor_location: Option<NodeLocation>,
    implements_interfaces: &IndexSet<ComponentName>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let interface_definitions = implements_interfaces
        .iter()
        .filter_map(|name| {
            schema
                .get_interface(name)
                .map(|interface| (name, interface))
        })
        .collect::<Vec<_>>();

    // Implements Interfaces must be defined.
    //
    // Returns Undefined Definition error.
    for interface_name in implements_interfaces {
        if schema.get_interface(interface_name).is_some() {
            continue;
        }

        // interface_name.loc should always be Some
        let loc = interface_name.location();
        diagnostics.push(ValidationError::new(
            loc,
            DiagnosticData::UndefinedDefinition {
                name: interface_name.name.clone(),
            },
        ));
    }

    // Transitively implemented interfaces must be defined on an implementing
    // type or interface.
    //
    // Returns Transitive Implemented Interfaces error.
    let transitive_interfaces = interface_definitions.iter().flat_map(|&(name, interface)| {
        interface
            .implements_interfaces
            .iter()
            .map(|component| &component.name)
            .zip(std::iter::repeat(name))
    });
    for (transitive_interface, via_interface) in transitive_interfaces {
        if implements_interfaces.contains(transitive_interface) {
            continue;
        }

        let transitive_loc = transitive_interface.location();
        diagnostics.push(ValidationError::new(
            implementor_location,
            DiagnosticData::TransitiveImplementedInterfaces {
                interface: implementor_name.clone(),
                via_interface: via_interface.name.clone(),
                missing_interface: transitive_interface.clone(),
                transitive_interface_location: transitive_loc,
            },
        ));
    }

    diagnostics
}
