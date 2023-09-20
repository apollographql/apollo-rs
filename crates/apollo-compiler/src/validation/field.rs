use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema,
    validation::ValidationDatabase,
    Node,
};
use std::collections::HashMap;

use super::operation::OperationValidationConfig;

pub fn validate_field(
    db: &dyn ValidationDatabase,
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
        context.variables.clone(),
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
                if let Some(diag) = super::variable::validate_variable_usage2(
                    db,
                    arg_definition.clone(),
                    context.variables.clone(),
                    argument,
                )
                .err()
                {
                    diagnostics.push(diag)
                } else {
                    diagnostics.extend(super::value::validate_values2(
                        db,
                        &arg_definition.ty,
                        argument,
                        context.variables,
                    ));
                }
            } else {
                let mut labels = vec![Label::new(
                    argument.location().unwrap(),
                    "argument name not found",
                )];
                if let Some(loc) = field_definition.location() {
                    labels.push(Label::new(loc, "field declared here"));
                };
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        argument.location().unwrap().into(),
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

            if arg_definition.is_required() && is_null {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    field.location().unwrap().into(),
                    DiagnosticData::RequiredArgument {
                        name: arg_definition.name.to_string(),
                    },
                );
                diagnostic = diagnostic.label(Label::new(
                    field.location().unwrap(),
                    format!("missing value for argument `{}`", arg_definition.name),
                ));
                if let Some(loc) = arg_definition.location() {
                    diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));
                }

                diagnostics.push(diagnostic);
            }
        }

        match validate_leaf_field_selection(db, field.clone(), &field_definition.ty) {
            Err(diag) => diagnostics.push(diag),
            Ok(_) => diagnostics.extend(super::selection::validate_selection_set2(
                db,
                Some(field_definition.ty.inner_named_type()),
                &field.selection_set,
                context,
            )),
        }
    } else {
        let fname = &field.name;
        let help = format!("`{fname}` is not defined on `{against_type}` type");
        let diagnostic = ApolloDiagnostic::new(
            db,
            field.location().unwrap().into(),
            DiagnosticData::UndefinedField {
                field: fname.to_string(),
                ty: against_type.to_string(),
            },
        )
        .label(Label::new(
            field.location().unwrap(),
            format!("`{fname}` field is not defined"),
        ))
        .help(help);

        let parent_type_loc = schema
            .types
            .get(against_type)
            .and_then(|type_def| type_def.location());

        let diagnostic = if let Some(parent_type_loc) = parent_type_loc {
            diagnostic.label(Label::new(
                parent_type_loc,
                format!("`{against_type}` declared here"),
            ))
        } else {
            diagnostic
        };
        diagnostics.push(diagnostic);
    }

    diagnostics
}

pub fn validate_field_definition(
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

    diagnostics.extend(super::input_object::validate_input_value_definitions(
        db,
        &field.arguments,
        ast::DirectiveLocation::ArgumentDefinition,
    ));

    diagnostics
}

pub fn validate_field_definitions(
    db: &dyn ValidationDatabase,
    fields: Vec<Node<ast::FieldDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let mut seen: HashMap<ast::Name, &Node<ast::FieldDefinition>> = HashMap::new();

    for field in &fields {
        diagnostics.extend(validate_field_definition(db, field));

        // Fields must be unique.
        //
        // Returns Unique Field error.
        let fname = &field.name;
        let redefined_definition = field
            .location()
            .expect("undefined field definition location");

        if let Some(prev_field) = seen.get(fname) {
            let original_definition = prev_field
                .location()
                .expect("undefined field definition location");

            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    original_definition.into(),
                    DiagnosticData::UniqueField {
                        field: fname.to_string(),
                        original_definition: original_definition.into(),
                        redefined_definition: redefined_definition.into(),
                    },
                )
                .labels([
                    Label::new(
                        original_definition,
                        format!("previous definition of `{fname}` here"),
                    ),
                    Label::new(redefined_definition, format!("`{fname}` redefined here")),
                ])
                .help(format!(
                    "`{fname}` field must only be defined once in this input object definition."
                )),
            );
        } else {
            seen.insert(fname.clone(), field);
        }

        // Field types in Object Types must be of output type
        let loc = field
            .location()
            .expect("undefined field definition location");
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
                    ApolloDiagnostic::new(db, loc.into(), DiagnosticData::OutputType {
                        name: field.name.to_string(),
                        ty: kind,
                    })
                        .label(Label::new(loc, format!("this is {particle} {kind}")))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name)),
                );
            }
        } else if let Some(field_ty_loc) = field.ty.inner_named_type().location() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field_ty_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: field.name.to_string(),
                    },
                )
                .label(Label::new(field_ty_loc, "not found in this scope")),
            );
        } else {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: field.ty.to_string(),
                    },
                )
                .label(Label::new(loc, "not found in this scope")),
            );
        }
    }

    diagnostics
}

pub fn validate_leaf_field_selection(
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
        let label = match type_def {
            schema::ExtendedType::Enum(_) => {
                format!("field `{fname}` of type `{tname}` is an enum and cannot select any fields")
            }
            schema::ExtendedType::Scalar(_) => format!(
                "field `{fname}` of type `{tname}` is a scalar and cannot select any fields"
            ),
            _ => return Ok(()),
        };
        (label, DiagnosticData::DisallowedSubselection)
    };

    Err(
        ApolloDiagnostic::new(db, field.location().unwrap().into(), diagnostic_data)
            .label(Label::new(field.location().unwrap(), label))
            .label(Label::new(
                type_def.location().unwrap(),
                format!("`{tname}` declared here"),
            )),
    )
}
