use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationDatabase,
};

pub fn validate_field(
    db: &dyn ValidationDatabase,
    field: Arc<hir::Field>,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        field.directives().to_vec(),
        hir::DirectiveLocation::Field,
        var_defs.clone(),
    );

    if let Some(field_definition) = field.field_definition(db.upcast()) {
        if !field.arguments().is_empty() {
            diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));
            for arg in field.arguments() {
                let exists = field_definition
                    .arguments()
                    .input_values()
                    .iter()
                    .any(|arg_def| arg.name() == arg_def.name());

                if !exists {
                    let mut labels = vec![Label::new(arg.loc, "argument name not found")];
                    if let Some(loc) = field_definition.loc {
                        labels.push(Label::new(loc, "field declared here"));
                    };
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            arg.loc.into(),
                            DiagnosticData::UndefinedArgument {
                                name: arg.name().into(),
                            },
                        )
                        .labels(labels),
                    );
                }
                // Let var_usage be the input value where the original
                // argument for the current variable usage is defined.
                let var_usage = field_definition
                    .arguments()
                    .input_values()
                    .iter()
                    .find(|val| val.name() == arg.name())
                    .cloned();
                if let Some(diag) = db
                    .validate_variable_usage(var_usage, var_defs.clone(), arg.clone())
                    .err()
                {
                    diagnostics.push(diag)
                }
            }

            for arg in field.arguments() {
                if let hir::Value::Variable(var) = arg.value() {
                    // Let var_def be the VariableDefinition named
                    // variableName defined within operation.
                    let var_def = vars.iter().find(|v| v.name() == var.name());
                    // Let var_usage be the input value where the original
                    // argument for the current variable usage is defined.
                    let var_usage = field_definition
                        .arguments()
                        .input_values()
                        .iter()
                        .find(|val| val.name() == arg.name());
                    // 1. Let variableType be the expected type of variableDefinition.
                    let var_ty = var_def.map(|v| v.ty());
                    // 2. Let locationType be the expected type of the Argument,
                    // ObjectField, or ListValue entry where variableUsage is
                    // located.
                    let loc_ty = var_usage.map(|v| v.ty());
                    // 3. if locationType is a non-null type AND variableType is
                    // NOT a non-null type:
                    if loc_ty.is_some()
                        && var_ty.is_some()
                        && loc_ty.unwrap().is_non_null()
                        && !var_ty.unwrap().is_non_null()
                    {
                        // 3.a. let hasNonNullVariableDefaultValue be true
                        // if a default value exists for variableDefinition
                        // and is not the value null.
                        let has_non_null_default_value = var_def.iter().any(|var| {
                            var.default_value().is_some()
                                && (var.default_value().unwrap() != &hir::Value::Null)
                        });
                        // 3.b. Let hasLocationDefaultValue be true if a default
                        // value exists for the Argument or ObjectField where
                        // variableUsage is located.
                        let has_location_default_value = var_usage.iter().any(|val| {
                            val.default_value().is_some()
                                && (val.default_value().unwrap() != &hir::Value::Null)
                        });
                        // 3.c. If hasNonNullVariableDefaultValue is NOT true
                        // AND hasLocationDefaultValue is NOT true, return
                        // false.

                        // 3.d. Let nullableLocationType be the unwrapped
                        // nullable type of locationType.
                    }
                }
            }
        }

        for arg_def in field_definition.arguments().input_values() {
            let arg_value = field
                .arguments()
                .iter()
                .find(|value| value.name() == arg_def.name());
            let is_null = match arg_value {
                None => true,
                // Prevents explicitly providing `requiredArg: null`,
                // but you can still indirectly do the wrong thing by typing `requiredArg: $mayBeNull`
                // and it won't raise a validation error at this stage.
                Some(value) => value.value() == &hir::Value::Null,
            };

            if arg_def.is_required() && is_null {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    field.loc.into(),
                    DiagnosticData::RequiredArgument {
                        name: arg_def.name().into(),
                    },
                );
                diagnostic = diagnostic.label(Label::new(
                    field.loc,
                    format!("missing value for argument `{}`", arg_def.name()),
                ));
                if let Some(loc) = arg_def.loc {
                    diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));
                }

                diagnostics.push(diagnostic);
            }
        }
    }

    let field_type = field.ty(db.upcast());
    if let Some(field_type) = field_type {
        match db.validate_leaf_field_selection(field.clone(), field_type) {
            Err(diag) => diagnostics.push(diag),
            Ok(_) => diagnostics
                .extend(db.validate_selection_set(field.selection_set().clone(), var_defs)),
        }
    } else {
        let help = format!(
            "`{}` is not defined on `{}` type",
            field.name(),
            field.parent_obj.as_ref().unwrap(),
        );
        let fname = field.name();
        let diagnostic = ApolloDiagnostic::new(
            db,
            field.loc().into(),
            DiagnosticData::UndefinedField {
                field: fname.into(),
            },
        )
        .label(Label::new(
            field.loc(),
            format!("`{fname}` field is not defined"),
        ))
        .help(help);

        let parent_type_loc = db
            .find_type_definition_by_name(field.parent_obj.clone().unwrap())
            .map(|type_def| type_def.loc());

        let diagnostic = if let Some(parent_type_loc) = parent_type_loc {
            diagnostic.label(Label::new(
                parent_type_loc,
                format!("`{}` declared here", field.parent_obj.as_ref().unwrap()),
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
    field: hir::FieldDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        field.directives().to_vec(),
        hir::DirectiveLocation::FieldDefinition,
        // field definitions don't have variables
        Arc::new(Vec::new()),
    );

    diagnostics.extend(db.validate_arguments_definition(
        field.arguments,
        hir::DirectiveLocation::ArgumentDefinition,
    ));

    diagnostics
}

pub fn validate_field_definitions(
    db: &dyn ValidationDatabase,
    fields: Vec<hir::FieldDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen: HashMap<&str, &hir::FieldDefinition> = HashMap::new();

    for field in fields.iter() {
        diagnostics.extend(db.validate_field_definition(field.clone()));

        // Fields must be unique.
        //
        // Returns Unique Field error.
        let fname = field.name();
        let redefined_definition = field.loc().expect("undefined field definition location");

        if let Some(prev_field) = seen.get(fname) {
            let original_definition = prev_field
                .loc()
                .expect("undefined field definition location");

            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    original_definition.into(),
                    DiagnosticData::UniqueField {
                        field: fname.into(),
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
            seen.insert(fname, field);
        }

        // Field types in Object Types must be of output type
        let loc = field.loc().expect("undefined field definition location");
        if let Some(field_ty) = field.ty().type_def(db.upcast()) {
            if !field.ty().is_output_type(db.upcast()) {
                diagnostics.push(
                    ApolloDiagnostic::new(db, loc.into(), DiagnosticData::OutputType {
                        name: field.name().into(),
                        ty: field_ty.kind(),
                    })
                        .label(Label::new(loc, format!("this is of `{}` type", field_ty.kind())))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name())),
                );
            }
        } else if let Some(field_ty_loc) = field.ty().loc() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field_ty_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: field.name().into(),
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
                        name: field.ty().name(),
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
    field: Arc<hir::Field>,
    field_type: hir::Type,
) -> Result<(), ApolloDiagnostic> {
    let is_leaf = field.selection_set.selection.is_empty();
    let tname = field_type.name();
    let fname = field.name.src.clone();

    let type_def = match db.find_type_definition_by_name(tname.clone()) {
        Some(type_def) => type_def,
        None => return Ok(()),
    };

    let (label, diagnostic_data) = if is_leaf {
        let label = match type_def {
            hir::TypeDefinition::ObjectTypeDefinition(_) => {
                format!("field `{fname}` type `{tname}` is an object and must select fields")
            }
            hir::TypeDefinition::InterfaceTypeDefinition(_) => {
                format!("field `{fname}` type `{tname}` is an interface and must select fields")
            }
            hir::TypeDefinition::UnionTypeDefinition(_) => {
                format!("field `{fname}` type `{tname}` is an union and must select fields")
            }
            _ => return Ok(()),
        };
        (label, DiagnosticData::MissingSubselection)
    } else {
        let label = match type_def {
            hir::TypeDefinition::EnumTypeDefinition(_) => {
                format!("field `{fname}` of type `{tname}` is an enum and cannot select any fields")
            }
            hir::TypeDefinition::ScalarTypeDefinition(_) => format!(
                "field `{fname}` of type `{tname}` is a scalar and cannot select any fields"
            ),
            _ => return Ok(()),
        };
        (label, DiagnosticData::DisallowedSubselection)
    };

    Err(ApolloDiagnostic::new(db, field.loc.into(), diagnostic_data)
        .label(Label::new(field.loc, label))
        .label(Label::new(
            type_def.loc(),
            format!("`{tname}` declared here"),
        )))
}
