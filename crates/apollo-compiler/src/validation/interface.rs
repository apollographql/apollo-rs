use crate::ast;
use crate::ast::Type;
use crate::collections::IndexSet;
use crate::parser::SourceSpan;
use crate::schema::validation::BuiltInScalars;
use crate::schema::ComponentName;
use crate::schema::InterfaceType;
use crate::schema::Name;
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

    // Validate that fields in the implementing interface have return types that are
    // proper subtypes of the super-interface fields.
    validate_implementation_field_types(
        diagnostics,
        schema,
        &interface.name,
        &interface.fields,
        &interface.implements_interfaces,
    );
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

/// GraphQL spec: [IsValidImplementationFieldType](https://spec.graphql.org/draft/#IsValidImplementationFieldType())
pub(crate) fn is_valid_implementation_field_type(
    schema: &crate::Schema,
    interface_field_type: &Type,
    impl_field_type: &Type,
) -> bool {
    match (interface_field_type, impl_field_type) {
        // NonNull interface field requires NonNull implementation
        (Type::NonNullNamed(_) | Type::NonNullList(_), Type::Named(_) | Type::List(_)) => false,
        // Both NonNull named: inner names must match or impl must be a subtype
        (Type::NonNullNamed(iface_name), Type::NonNullNamed(impl_name)) => {
            iface_name == impl_name || schema.is_subtype(iface_name, impl_name)
        }
        // Both NonNull lists: recurse on item types
        (Type::NonNullList(iface_inner), Type::NonNullList(impl_inner)) => {
            is_valid_implementation_field_type(schema, iface_inner, impl_inner)
        }
        // NonNull list vs NonNull named or vice versa
        (Type::NonNullNamed(_), Type::NonNullList(_))
        | (Type::NonNullList(_), Type::NonNullNamed(_)) => false,
        // Nullable named: impl can be nullable or non-null, same name or subtype
        (Type::Named(iface_name), Type::Named(impl_name) | Type::NonNullNamed(impl_name)) => {
            iface_name == impl_name || schema.is_subtype(iface_name, impl_name)
        }
        // Nullable list: impl can be nullable or non-null list, recurse on items
        (Type::List(iface_inner), Type::List(impl_inner) | Type::NonNullList(impl_inner)) => {
            is_valid_implementation_field_type(schema, iface_inner, impl_inner)
        }
        // Named vs List mismatch
        (Type::Named(_), Type::List(_) | Type::NonNullList(_)) => false,
        (Type::List(_), Type::Named(_) | Type::NonNullNamed(_)) => false,
    }
}

/// Validates that fields in an implementing type have return types that are proper subtypes of
/// the corresponding interface fields.
pub(crate) fn validate_implementation_field_types(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    implementor_name: &Name,
    implementor_fields: &crate::collections::IndexMap<Name, crate::schema::Component<crate::ast::FieldDefinition>>,
    implements_interfaces: &IndexSet<ComponentName>,
) {
    for interface_name in implements_interfaces {
        let Some(interface) = schema.get_interface(interface_name) else {
            continue;
        };
        for (field_name, interface_field) in &interface.fields {
            let Some(impl_field) = implementor_fields.get(field_name) else {
                continue; // Missing field is reported separately
            };
            if !is_valid_implementation_field_type(schema, &interface_field.ty, &impl_field.ty) {
                diagnostics.push(
                    impl_field.location(),
                    DiagnosticData::InvalidImplementationFieldType {
                        name: implementor_name.clone(),
                        interface: interface_name.name.clone(),
                        field: field_name.clone(),
                        interface_type: Type::clone(&interface_field.ty),
                        actual_type: Type::clone(&impl_field.ty),
                        field_location: impl_field.location(),
                        interface_field_location: interface_field.location(),
                    },
                );
            }
        }
    }
}
