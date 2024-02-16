use crate::coordinate::{FieldArgumentCoordinate, TypeAttributeCoordinate};
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::{ast, executable, schema, ExecutableDocument, Node};

use super::operation::OperationValidationConfig;
use crate::ast::Name;
use crate::schema::Component;
use indexmap::IndexMap;

pub(crate) fn validate_field(
    document: &ExecutableDocument,
    // May be None if a parent selection was invalid
    against_type: Option<(&crate::Schema, &ast::NamedType)>,
    field: &Node<executable::Field>,
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    // First do all the validation that we can without knowing the type of the field.

    let mut diagnostics = super::directive::validate_directives(
        context.schema,
        field.directives.iter(),
        ast::DirectiveLocation::Field,
        context.variables,
    );

    diagnostics.extend(super::argument::validate_arguments(&field.arguments));

    // Return early if we don't know the type--this can happen if we are nested deeply
    // inside a selection set that has a wrong field, or if we are validating a standalone
    // operation without a schema.
    let Some((schema, against_type)) = against_type else {
        return diagnostics;
    };

    if let Ok(field_definition) = schema.type_field(against_type, &field.name) {
        for argument in &field.arguments {
            let arg_definition = field_definition
                .arguments
                .iter()
                .find(|val| val.name == argument.name);
            if let Some(arg_definition) = arg_definition {
                if let Some(diag) = super::variable::validate_variable_usage(
                    arg_definition,
                    context.variables,
                    argument,
                )
                .err()
                {
                    diagnostics.push(diag)
                } else {
                    diagnostics.extend(super::value::validate_values(
                        schema,
                        &arg_definition.ty,
                        argument,
                        context.variables,
                    ));
                }
            } else {
                let loc = field_definition.location();

                diagnostics.push(ValidationError::new(
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
                ));
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
                diagnostics.push(ValidationError::new(
                    field.location(),
                    DiagnosticData::RequiredArgument {
                        name: arg_definition.name.clone(),
                        coordinate: FieldArgumentCoordinate {
                            ty: against_type.clone(),
                            field: field.name.clone(),
                            argument: arg_definition.name.clone(),
                        }
                        .into(),
                        definition_location: arg_definition.location(),
                    },
                ));
            }
        }

        match validate_leaf_field_selection(schema, field, &field_definition.ty) {
            Err(diag) => diagnostics.push(diag),
            Ok(_) => diagnostics.extend(super::selection::validate_selection_set(
                document,
                Some((schema, field_definition.ty.inner_named_type())),
                &field.selection_set,
                context,
            )),
        }
    }

    diagnostics
}

pub(crate) fn validate_field_definition(
    schema: &crate::Schema,
    field: &Node<ast::FieldDefinition>,
) -> Vec<ValidationError> {
    let mut diagnostics = super::directive::validate_directives(
        Some(schema),
        field.directives.iter(),
        ast::DirectiveLocation::FieldDefinition,
        // field definitions don't have variables
        Default::default(),
    );

    diagnostics.extend(super::input_object::validate_argument_definitions(
        schema,
        &field.arguments,
        ast::DirectiveLocation::ArgumentDefinition,
    ));

    diagnostics
}

pub(crate) fn validate_field_definitions(
    schema: &crate::Schema,
    fields: &IndexMap<Name, Component<ast::FieldDefinition>>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for field in fields.values() {
        diagnostics.extend(validate_field_definition(schema, field));

        // Field types in Object Types must be of output type
        let loc = field.location();
        let type_location = field.ty.inner_named_type().location();
        if let Some(field_ty) = schema.types.get(field.ty.inner_named_type()) {
            if !field_ty.is_output_type() {
                // Output types are unreachable
                diagnostics.push(ValidationError::new(
                    loc,
                    DiagnosticData::OutputType {
                        name: field.name.clone(),
                        describe_type: field_ty.describe(),
                        type_location,
                    },
                ));
            }
        } else {
            diagnostics.push(ValidationError::new(
                type_location,
                DiagnosticData::UndefinedDefinition {
                    name: field.ty.inner_named_type().clone(),
                },
            ));
        }
    }

    diagnostics
}

pub(crate) fn validate_leaf_field_selection(
    schema: &crate::Schema,
    field: &Node<executable::Field>,
    field_type: &ast::Type,
) -> Result<(), ValidationError> {
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
        Err(ValidationError::new(
            field.location(),
            DiagnosticData::MissingSubselection {
                coordinate: TypeAttributeCoordinate {
                    ty: tname.clone(),
                    attribute: fname.clone(),
                },
                describe_type: type_def.describe(),
            },
        ))
    } else {
        Ok(())
    }
}
