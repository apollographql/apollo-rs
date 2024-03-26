use crate::ast;
use crate::schema::ExtendedType;
use crate::schema::ObjectType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::field::validate_field_definitions;
use crate::validation::DiagnosticList;
use crate::Node;

pub(crate) fn validate_object_type_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
) {
    for ty in schema.types.values() {
        if let ExtendedType::Object(object) = ty {
            validate_object_type_definition(diagnostics, schema, object)
        }
    }
}

pub(crate) fn validate_object_type_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    object: &Node<ObjectType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        object.directives.iter_ast(),
        ast::DirectiveLocation::Object,
        // objects don't use variables
        Default::default(),
    );

    // Object Type field validations.
    validate_field_definitions(diagnostics, schema, &object.fields);

    // validate there is at least one field on the type
    // https://spec.graphql.org/draft/#sel-FAHZhCFDBAACDA4qe
    if object.fields.is_empty() {
        diagnostics.push(
            object.location(),
            DiagnosticData::EmptyFieldSet {
                type_name: object.name.clone(),
                type_location: object.location(),
                extensions_locations: object.extensions().iter().map(|ext| ext.location()).collect(),
            },
        );
    }

    // Implements Interfaces validation.
    super::interface::validate_implements_interfaces(
        diagnostics,
        schema,
        &object.name,
        object.location(),
        &object.implements_interfaces,
    );

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

                diagnostics.push(
                    object.location(),
                    DiagnosticData::MissingInterfaceField {
                        name: object.name.clone(),
                        implements_location: implements_interface.location(),
                        interface: implements_interface.name.clone(),
                        field: interface_field.name.clone(),
                        field_location: interface_field.location(),
                    },
                );
            }
        }
    }
}
