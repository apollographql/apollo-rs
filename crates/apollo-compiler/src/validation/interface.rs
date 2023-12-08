use crate::{
    ast, schema,
    validation::diagnostics::{DiagnosticData, ValidationError},
    ValidationDatabase,
};
use std::collections::HashSet;

pub(crate) fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for interface in db.ast_types().interfaces.values() {
        diagnostics.extend(db.validate_interface_definition(interface.clone()));
    }

    diagnostics
}

pub(crate) fn validate_interface_definition(
    db: &dyn ValidationDatabase,
    interface: ast::TypeWithExtensions<ast::InterfaceTypeDefinition>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    diagnostics.extend(super::directive::validate_directives(
        db,
        interface.directives(),
        ast::DirectiveLocation::Interface,
        // interfaces don't use variables
        Default::default(),
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
    for implements_interface in interface.implements_interfaces() {
        if *implements_interface == interface.definition.name {
            diagnostics.push(ValidationError::new(
                implements_interface.location(),
                DiagnosticData::RecursiveInterfaceDefinition {
                    name: implements_interface.to_string(),
                },
            ));
        }
    }

    // Interface Type field validation.
    let field_definitions = interface.fields().cloned().collect();
    diagnostics.extend(db.validate_field_definitions(field_definitions));

    // Implements Interfaceds validation.
    let implements_interfaces: Vec<_> = interface.implements_interfaces().cloned().collect();
    diagnostics.extend(validate_implements_interfaces(
        db,
        &interface.definition.clone().into(),
        &implements_interfaces,
    ));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    let field_names: HashSet<ast::Name> =
        interface.fields().map(|field| field.name.clone()).collect();
    for implements_interface in interface.implements_interfaces() {
        if let Some(schema::ExtendedType::Interface(super_interface)) =
            schema.types.get(implements_interface)
        {
            for super_field in super_interface.fields.values() {
                if field_names.contains(&super_field.name) {
                    continue;
                }
                diagnostics.push(ValidationError::new(
                    interface.definition.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: interface.definition.name.to_string(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.to_string(),
                        field: super_field.name.to_string(),
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
    implementor: &ast::Definition,
    implements_interfaces: &[ast::Name],
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
                name: interface_name.to_string(),
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

        let definition_loc = implementor.location();
        // let via_loc = via_interface
        //     .location();
        let transitive_loc = transitive_interface.location();
        diagnostics.push(ValidationError::new(
            definition_loc,
            DiagnosticData::TransitiveImplementedInterfaces {
                interface: implementor.name().unwrap().to_string(),
                via_interface: via_interface.to_string(),
                missing_interface: transitive_interface.to_string(),
                transitive_interface_location: transitive_loc,
            },
        ));
    }

    diagnostics
}
