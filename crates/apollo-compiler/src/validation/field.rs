use crate::ast;
use crate::ast::Name;
use crate::coordinate::FieldArgumentCoordinate;
use crate::coordinate::TypeAttributeCoordinate;
use crate::executable;
use crate::schema;
use crate::schema::Component;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;
use crate::validation::OperationValidationContext;
use crate::ExecutableDocument;
use crate::Node;
use indexmap::IndexMap;

pub(crate) fn validate_field(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    // May be None if a parent selection was invalid
    against_type: Option<(&crate::Schema, &ast::NamedType)>,
    field: &Node<executable::Field>,
    context: &mut OperationValidationContext<'_>,
) {
    // First do all the validation that we can without knowing the type of the field.

    super::directive::validate_directives(
        diagnostics,
        context.schema(),
        field.directives.iter(),
        ast::DirectiveLocation::Field,
        context.variables,
    );

    super::argument::validate_arguments(diagnostics, &field.arguments);

    // Return early if we don't know the type--this can happen if we are nested deeply
    // inside a selection set that has a wrong field, or if we are validating a standalone
    // operation without a schema.
    let Some((schema, against_type)) = against_type else {
        return;
    };

    if let Ok(field_definition) = schema.type_field(against_type, &field.name) {
        for argument in &field.arguments {
            let arg_definition = field_definition
                .arguments
                .iter()
                .find(|val| val.name == argument.name);
            if let Some(arg_definition) = arg_definition {
                if super::variable::validate_variable_usage(
                    diagnostics,
                    arg_definition,
                    context.variables,
                    argument,
                )
                .is_ok()
                {
                    super::value::validate_values(
                        diagnostics,
                        schema,
                        &arg_definition.ty,
                        argument,
                        context.variables,
                    );
                }
            } else {
                let loc = field_definition.location();

                diagnostics.push(
                    argument.location(),
                    DiagnosticData::UndefinedArgument {
                        name: argument.name.clone(),
                        coordinate: TypeAttributeCoordinate {
                            ty: against_type.clone(),
                            attribute: field.name.clone(),
                        }
                        .into(),
                        definition_location: loc,
                    },
                );
            }
        }

        for arg_definition in &field_definition.arguments {
            let arg_value = field.arguments.iter().find_map(|argument| {
                (argument.name == arg_definition.name).then_some(&argument.value)
            });
            let is_null = match arg_value {
                None => true,
                // Prevents explicitly providing `requiredArg: null`,
                // but you can still indirectly do the wrong thing by typing `requiredArg: $mayBeNull`
                // and it won't raise a validation error at this stage.
                Some(value) => value.is_null(),
            };

            if arg_definition.is_required() && is_null {
                diagnostics.push(
                    field.location(),
                    DiagnosticData::RequiredArgument {
                        name: arg_definition.name.clone(),
                        expected_type: arg_definition.ty.clone(),
                        coordinate: FieldArgumentCoordinate {
                            ty: against_type.clone(),
                            field: field.name.clone(),
                            argument: arg_definition.name.clone(),
                        }
                        .into(),
                        definition_location: arg_definition.location(),
                    },
                );
            }
        }

        if validate_leaf_field_selection(
            diagnostics,
            schema,
            against_type,
            field,
            &field_definition.ty,
        )
        .is_ok()
        {
            super::selection::validate_selection_set(
                diagnostics,
                document,
                Some((schema, field_definition.ty.inner_named_type())),
                &field.selection_set,
                context,
            )
        }
    }
}

pub(crate) fn validate_field_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    field: &Node<ast::FieldDefinition>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        field.directives.iter(),
        ast::DirectiveLocation::FieldDefinition,
        // field definitions don't have variables
        Default::default(),
    );

    super::input_object::validate_argument_definitions(
        diagnostics,
        schema,
        &field.arguments,
        ast::DirectiveLocation::ArgumentDefinition,
    );
}

pub(crate) fn validate_field_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    fields: &IndexMap<Name, Component<ast::FieldDefinition>>,
) {
    for field in fields.values() {
        validate_field_definition(diagnostics, schema, field);

        // Field types in Object Types must be of output type
        let loc = field.location();
        let type_location = field.ty.inner_named_type().location();
        if let Some(field_ty) = schema.types.get(field.ty.inner_named_type()) {
            if !field_ty.is_output_type() {
                // Output types are unreachable
                diagnostics.push(
                    loc,
                    DiagnosticData::OutputType {
                        name: field.name.clone(),
                        describe_type: field_ty.describe(),
                        type_location,
                    },
                );
            }
        } else {
            diagnostics.push(
                type_location,
                DiagnosticData::UndefinedDefinition {
                    name: field.ty.inner_named_type().clone(),
                },
            );
        }
    }
}

pub(crate) fn validate_leaf_field_selection(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    parent_type: &ast::NamedType,
    field: &Node<executable::Field>,
    field_type: &ast::Type,
) -> Result<(), ()> {
    let is_leaf = field.selection_set.is_empty();
    let tname = field_type.inner_named_type();
    let fname = &field.name;

    let type_def = match schema.types.get(tname) {
        Some(type_def) => type_def,
        // If we don't have the type we can't check if it requires a subselection.
        None => return Ok(()),
    };

    if is_leaf
        && matches!(
            type_def,
            schema::ExtendedType::Object(_)
                | schema::ExtendedType::Interface(_)
                | schema::ExtendedType::Union(_)
        )
    {
        diagnostics.push(
            field.location(),
            DiagnosticData::MissingSubselection {
                coordinate: TypeAttributeCoordinate {
                    ty: parent_type.clone(),
                    attribute: fname.clone(),
                },
                output_type: tname.clone(),
                describe_type: type_def.describe(),
            },
        );
        Err(())
    } else {
        Ok(())
    }
}
