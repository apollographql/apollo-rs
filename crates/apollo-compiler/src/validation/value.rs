use std::sync::Arc;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, TypeDefinition, Value},
    validation::ValidationDatabase,
};

macro_rules! unsupported_type {
    ($db: expr, $value: expr, $type_def: expr) => {{
        ApolloDiagnostic::new(
            $db,
            $value.loc().into(),
            DiagnosticData::UnsupportedValueType {
                value: $value.value_kind().into(),
                ty: $type_def.name().into(),
            },
        )
        .labels([
            Label::new(
                $type_def.loc(),
                format!("field declared here as {} type", $type_def.name()),
            ),
            Label::new(
                $value.loc(),
                format!("argument declared here is of {} type", $value.value_kind()),
            ),
        ])
    }};
}
pub fn validate_values(
    db: &dyn ValidationDatabase,
    ty: &hir::Type,
    arg: &hir::Argument,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Result<(), Vec<ApolloDiagnostic>> {
    let mut diagnostics = Vec::new();
    let type_def = ty.type_def(db.upcast());

    if let Some(type_def) = type_def {
        value_of_correct_type(
            db,
            type_def,
            arg.value().clone(),
            var_defs,
            &mut diagnostics,
        );

        match diagnostics.len() {
            0 => Ok(()),
            _ => Err(diagnostics),
        }
    } else {
        Ok(())
    }
}

pub fn value_of_correct_type(
    db: &dyn ValidationDatabase,
    type_def: hir::TypeDefinition,
    val: Value,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
    diagnostics: &mut Vec<ApolloDiagnostic>,
) {
    match val {
        Value::Int { value: int, .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(scalar) => {
                if scalar.is_int() || scalar.is_float() || scalar.is_id() {
                    if int.to_i32_checked().is_none() {
                        diagnostics.push(
                            ApolloDiagnostic::new(
                                db,
                                val.loc().into(),
                                DiagnosticData::IntCoercionError {
                                    value: int.get().to_string(),
                                },
                            )
                            .label(Label::new(
                                val.loc(),
                                "cannot be coerced to an 32-bit integer",
                            )),
                        )
                    }
                } else {
                    diagnostics.push(unsupported_type!(db, val, type_def));
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Float { .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(scalar) => {
                if !scalar.is_float() {
                    diagnostics.push(unsupported_type!(db, val, type_def));
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::String { .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(scalar) => {
                if !scalar.is_string() && !scalar.is_id() {
                    diagnostics.push(unsupported_type!(db, val, type_def));
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Boolean { .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(scalar) => {
                if !scalar.is_boolean() {
                    diagnostics.push(unsupported_type!(db, val, type_def));
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Null { .. } => {
            if !type_def.is_enum_type_definition() || !type_def.is_scalar_type_definition() {
                diagnostics.push(unsupported_type!(db, val, type_def));
            }
        }
        Value::Variable(ref var) => match &type_def {
            TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_) => {
                let var_def = var_defs.iter().find(|v| v.name() == var.name());
                if let Some(var_def) = var_def {
                    // we don't have the actual variable values here, so just
                    // compare if two Types are the same
                    if var_def.ty().name() != type_def.name() {
                        diagnostics.push(unsupported_type!(db, val.clone(), type_def));
                    }
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Enum { ref value, loc } => match &type_def {
            TypeDefinition::EnumTypeDefinition(enum_) => {
                let enum_val = enum_.values().find(|v| v.enum_value() == value.src());
                if enum_val.is_none() {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            loc.into(),
                            DiagnosticData::UndefinedValue {
                                value: value.src().into(),
                                definition: type_def.name().into(),
                            },
                        )
                        .label(Label::new(
                            val.loc(),
                            format!("does not exist on `{}` type", type_def.name()),
                        )),
                    );
                }
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::List { value: ref li, .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_) => li.iter().for_each(|v| {
                value_of_correct_type(
                    db,
                    type_def.clone(),
                    v.clone(),
                    var_defs.clone(),
                    diagnostics,
                )
            }),
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Object { value: ref obj, .. } => match &type_def {
            TypeDefinition::InputObjectTypeDefinition(input_obj) => {
                let undefined_field = obj.iter().find_map(|(name, value)| {
                    let is_undefined = !input_obj.fields().any(|f| f.name() == name.src());
                    if is_undefined {
                        return Some((name, value));
                    }
                    None
                });

                if let Some((name, value)) = undefined_field {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            value.loc().into(),
                            DiagnosticData::UndefinedValue {
                                value: name.src().into(),
                                definition: type_def.name().into(),
                            },
                        )
                        .label(Label::new(
                            value.loc(),
                            format!("does not exist on `{}` type", type_def.name()),
                        )),
                    );
                }

                input_obj.fields().for_each(|f| {
                    let ty = f.ty();
                    let is_missing = !obj.iter().any(|(name, ..)| f.name() == name.src());

                    if (ty.is_non_null() && f.default_value().is_none()) && is_missing {
                        let mut diagnostic = ApolloDiagnostic::new(
                            db,
                            val.loc().into(),
                            DiagnosticData::RequiredArgument {
                                name: f.name().into(),
                            },
                        );
                        diagnostic = diagnostic.label(Label::new(
                            val.loc(),
                            format!("missing value for argument `{}`", f.name()),
                        ));
                        if let Some(loc) = ty.loc() {
                            diagnostic = diagnostic.label(Label::new(loc, "argument defined here"));
                        }

                        diagnostics.push(diagnostic)
                    }

                    obj.iter().for_each(|(name, v)| {
                        let type_def = ty.type_def(db.upcast());
                        if name.src() == f.name() {
                            if let Some(type_def) = type_def {
                                value_of_correct_type(
                                    db,
                                    type_def,
                                    v.clone(),
                                    var_defs.clone(),
                                    diagnostics,
                                )
                            }
                        }
                    })
                })
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
    };
}
