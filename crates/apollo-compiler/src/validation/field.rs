use crate::diagnostics::{ApolloDiagnostic, DiagnosticData, Label};
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
                let mut labels = vec![Label::new(argument.location(), "argument name not found")];
                let loc = field_definition.location();
                labels.push(Label::new(loc, "field declared here"));

                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        argument.location(),
                        DiagnosticData::UndefinedArgument {
                            name: argument.name.to_string(),
                        },
                    )
                    .labels(labels),
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

            if arg_definition.is_required() && is_null && arg_definition.default_value.is_none() {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    field.location(),
                    DiagnosticData::RequiredArgument {
                        name: arg_definition.name.to_string(),
                    },
                );
                diagnostic = diagnostic.label(Label::new(
                    field.location(),
                    format!("missing value for argument `{}`", arg_definition.name),
                ));
                let loc = arg_definition.location();
                diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));

                diagnostics.push(diagnostic);
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
        if let Some(field_ty) = schema.types.get(field.ty.inner_named_type()) {
            if !field_ty.is_output_type() {
                // Output types are unreachable
                let (particle, kind) = match field_ty {
                    schema::ExtendedType::Scalar(_) => unreachable!(),
                    schema::ExtendedType::Union(_) => unreachable!(),
                    schema::ExtendedType::Enum(_) => unreachable!(),
                    schema::ExtendedType::Interface(_) => unreachable!(),
                    schema::ExtendedType::InputObject(_) => ("an", "input object"),
                    schema::ExtendedType::Object(_) => unreachable!(),
                };
                diagnostics.push(
                    ApolloDiagnostic::new(db, loc, DiagnosticData::OutputType {
                        name: field.name.to_string(),
                        ty: kind,
                    })
                        .label(Label::new(loc, format!("this is {particle} {kind}")))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name)),
                );
            }
        } else {
            let field_ty_loc = field.ty.inner_named_type().location();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field_ty_loc,
                    DiagnosticData::UndefinedDefinition {
                        name: field.ty.inner_named_type().to_string(),
                    },
                )
                .label(Label::new(field_ty_loc, "not found in this scope")),
            );
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
        None => return Ok(()),
    };

    let (label, diagnostic_data) = if is_leaf {
        let label = match type_def {
            schema::ExtendedType::Object(_) => {
                format!("field `{fname}` type `{tname}` is an object and must select fields")
            }
            schema::ExtendedType::Interface(_) => {
                format!("field `{fname}` type `{tname}` is an interface and must select fields")
            }
            schema::ExtendedType::Union(_) => {
                format!("field `{fname}` type `{tname}` is an union and must select fields")
            }
            _ => return Ok(()),
        };
        (label, DiagnosticData::MissingSubselection)
    } else {
        return Ok(());
    };

    Err(ApolloDiagnostic::new(db, field.location(), diagnostic_data)
        .label(Label::new(field.location(), label))
        .label(Label::new(
            type_def.location(),
            format!("`{tname}` declared here"),
        )))
}
