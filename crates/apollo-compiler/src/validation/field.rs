use crate::diagnostics::{ApolloDiagnostic, DiagnosticData};
use crate::validation::{FileId, ValidationDatabase};
use crate::{ast, schema, Node};

use super::operation::OperationValidationConfig;

pub(crate) fn validate_field(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    // May be None if a parent selection was invalid
    against_type: Option<&ast::NamedType>,
    field: Node<ast::Field>,
    context: OperationValidationConfig<'_>,
) -> Vec<ApolloDiagnostic> {
    // First do all the validation that we can without knowing the type of the field.

    let mut diagnostics = super::directive::validate_directives(
        db,
        field.directives.iter(),
        ast::DirectiveLocation::Field,
        context.variables,
    );

    diagnostics.extend(super::argument::validate_arguments(db, &field.arguments));

    // Return early if we don't know the type--this can happen if we are nested deeply
    // inside a selection set that has a wrong field, or if we are validating a standalone
    // operation without a schema.
    let Some(against_type) = against_type else {
        return diagnostics;
    };

    let schema = db.schema();

    if let Ok(field_definition) = schema.type_field(against_type, &field.name) {
        for argument in &field.arguments {
            let arg_definition = field_definition
                .arguments
                .iter()
                .find(|val| val.name == argument.name)
                .cloned();
            if let Some(arg_definition) = arg_definition {
                if let Some(diag) = super::variable::validate_variable_usage(
                    db,
                    arg_definition.clone(),
                    context.variables,
                    argument,
                )
                .err()
                {
                    diagnostics.push(diag)
                } else {
                    diagnostics.extend(super::value::validate_values(
                        db,
                        &arg_definition.ty,
                        argument,
                        context.variables,
                    ));
                }
            } else {
                let loc = field_definition.location();

                diagnostics.push(ApolloDiagnostic::new(
                    argument.location(),
                    DiagnosticData::UndefinedArgument {
                        name: argument.name.clone(),
                        coordinate: format!("{}.{}", against_type, field.name),
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

            if arg_definition.is_required() && is_null && arg_definition.default_value.is_none() {
                diagnostics.push(ApolloDiagnostic::new(
                    field.location(),
                    DiagnosticData::RequiredArgument {
                        name: arg_definition.name.to_string(),
                        coordinate: format!(
                            "{}.{}({}:)",
                            against_type, field.name, arg_definition.name
                        ),
                        definition_location: arg_definition.location(),
                    },
                ));
            }
        }

        match validate_leaf_field_selection(db, field.clone(), &field_definition.ty) {
            Err(diag) => diagnostics.push(diag),
            Ok(_) => diagnostics.extend(super::selection::validate_selection_set(
                db,
                file_id,
                Some(field_definition.ty.inner_named_type()),
                &field.selection_set,
                context,
            )),
        }
    }

    diagnostics
}

pub(crate) fn validate_field_definition(
    db: &dyn ValidationDatabase,
    field: &Node<ast::FieldDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives(
        db,
        field.directives.iter(),
        ast::DirectiveLocation::FieldDefinition,
        // field definitions don't have variables
        Default::default(),
    );

    diagnostics.extend(super::input_object::validate_argument_definitions(
        db,
        &field.arguments,
        ast::DirectiveLocation::ArgumentDefinition,
    ));

    diagnostics
}

pub(crate) fn validate_field_definitions(
    db: &dyn ValidationDatabase,
    fields: Vec<Node<ast::FieldDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    for field in &fields {
        diagnostics.extend(validate_field_definition(db, field));

        // Field types in Object Types must be of output type
        let loc = field.location();
        let type_location = field.ty.inner_named_type().location();
        if let Some(field_ty) = schema.types.get(field.ty.inner_named_type()) {
            if !field_ty.is_output_type() {
                // Output types are unreachable
                let kind = match field_ty {
                    schema::ExtendedType::Scalar(_) => unreachable!(),
                    schema::ExtendedType::Union(_) => unreachable!(),
                    schema::ExtendedType::Enum(_) => unreachable!(),
                    schema::ExtendedType::Interface(_) => unreachable!(),
                    schema::ExtendedType::InputObject(_) => "input object",
                    schema::ExtendedType::Object(_) => unreachable!(),
                };
                diagnostics.push(ApolloDiagnostic::new(
                    loc,
                    DiagnosticData::OutputType {
                        name: field.name.to_string(),
                        ty: kind,
                        type_location,
                    },
                ));
            }
        } else {
            diagnostics.push(ApolloDiagnostic::new(
                type_location,
                DiagnosticData::UndefinedDefinition {
                    name: field.ty.inner_named_type().to_string(),
                },
            ));
        }
    }

    diagnostics
}

pub(crate) fn validate_leaf_field_selection(
    db: &dyn ValidationDatabase,
    field: Node<ast::Field>,
    field_type: &ast::Type,
) -> Result<(), ApolloDiagnostic> {
    let schema = db.schema();

    let is_leaf = field.selection_set.is_empty();
    let tname = field_type.inner_named_type();
    let fname = &field.name;

    let type_def = match schema.types.get(tname) {
        Some(type_def) => type_def,
        // If we don't have the type we can't check if it requires a subselection.
        None => return Ok(()),
    };

    if is_leaf {
        let kind = match type_def {
            schema::ExtendedType::Object(_) => "object",
            schema::ExtendedType::Interface(_) => "interface",
            schema::ExtendedType::Union(_) => "union",
            _ => return Ok(()),
        };
        Err(ApolloDiagnostic::new(
            field.location(),
            DiagnosticData::MissingSubselection {
                coordinate: format!("{tname}.{fname}"),
                ty: kind,
            },
        ))
    } else {
        Ok(())
    }
}
