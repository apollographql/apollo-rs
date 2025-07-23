use crate::ast;
use crate::collections::IndexSet;
use crate::parser::SourceSpan;
use crate::schema::validation::BuiltInScalars;
use crate::schema::ComponentName;
use crate::schema::InterfaceType;
use crate::schema::Name;
use crate::schema::SchemaElement;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::field::validate_field_definitions;
use crate::validation::DiagnosticList;
use crate::Node;

pub(crate) fn validate_interface_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    interface: &Node<InterfaceType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        interface.directives.iter_ast(),
        ast::DirectiveLocation::Interface,
        // interfaces don't use variables
        Default::default(),
    );

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
            diagnostics.push(
                implements_interface.location(),
                DiagnosticData::RecursiveInterfaceDefinition {
                    name: implements_interface.name.clone(),
                },
            );
        }
    }

    // Interface Type field validation.
    validate_field_definitions(diagnostics, schema, built_in_scalars, &interface.fields);

    // validate there is at least one field on the type
    // https://spec.graphql.org/draft/#sel-HAHbnBFBABABxB4a
    if interface.fields.is_empty() {
        diagnostics.push(
            interface.location(),
            DiagnosticData::EmptyFieldSet {
                type_name: interface.name.clone(),
                type_location: interface.location(),
                extensions_locations: interface
                    .extensions()
                    .iter()
                    .map(|ext| ext.location())
                    .collect(),
            },
        );
    }

    // Implements Interfaceds validation.
    validate_implements_interfaces(
        diagnostics,
        schema,
        &interface.name,
        interface.location(),
        &interface.implements_interfaces,
    );

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
                diagnostics.push(
                    interface.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: interface.name.clone(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.name.clone(),
                        field: super_field.name.clone(),
                        field_location: super_field.location(),
                    },
                );
            }
        }
    }
}

pub(crate) fn validate_implements_interfaces(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    implementor_name: &Name,
    implementor_location: Option<SourceSpan>,
    implements_interfaces: &IndexSet<ComponentName>,
) {
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
        diagnostics.push(
            loc,
            DiagnosticData::UndefinedDefinition {
                name: interface_name.name.clone(),
            },
        );
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
        diagnostics.push(
            implementor_location,
            DiagnosticData::TransitiveImplementedInterfaces {
                interface: implementor_name.clone(),
                via_interface: via_interface.name.clone(),
                missing_interface: transitive_interface.clone(),
                transitive_interface_location: transitive_loc,
            },
        );
    }
}
