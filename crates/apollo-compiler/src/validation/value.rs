use std::sync::Arc;

use apollo_parser::ast::Type;

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
                value: $value.value_name().into(),
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
                format!("argument declared here is of {} type", $value.value_name()),
            ),
        ])
    }};
}
pub fn value_of_correct_type(
    db: &dyn ValidationDatabase,
    type_def: hir::TypeDefinition,
    val: Value,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Result<(), Vec<ApolloDiagnostic>> {
    let mut diagnostics = Vec::new();
    value_of_correct_type_2(db, type_def, val, var_defs, &mut diagnostics);

    match diagnostics.len() {
        0 => Ok(()),
        _ => Err(diagnostics),
    }
}

pub fn value_of_correct_type_2(
    db: &dyn ValidationDatabase,
    type_def: hir::TypeDefinition,
    val: Value,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
    diagnostics: &mut Vec<ApolloDiagnostic>,
) {
    match val {
        Value::Int { .. } => match &type_def {
            TypeDefinition::ScalarTypeDefinition(scalar) => {
                if !scalar.is_int() || !scalar.is_float() || !scalar.is_id() {
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
                if !scalar.is_string() || !scalar.is_id() {
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
            TypeDefinition::EnumTypeDefinition(enum_) => enum_.values().for_each(|v| {
                if v.enum_value() != value.src() {
                    diagnostics.push(ApolloDiagnostic::new(
                        db,
                        loc.into(),
                        DiagnosticData::UndefinedEnumValue {
                            value: v.enum_value().into(),
                            definition: type_def.name().into(),
                        },
                    ))
                }
            }),
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::List(ref li) => match &type_def {
            TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_) => li.iter().for_each(|v| {
                value_of_correct_type_2(
                    db,
                    type_def.clone(),
                    v.clone(),
                    var_defs.clone(),
                    diagnostics,
                )
            }),
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
        Value::Object(ref obj) => match &type_def {
            TypeDefinition::InputObjectTypeDefinition(input_obj) => {
                input_obj.fields().for_each(|f| {
                    obj.iter().for_each(|(name, v)| {
                        if f.name() == name.src() {
                            if let Some(type_def) = f.ty().type_def(db.upcast()) {
                                value_of_correct_type_2(
                                    db,
                                    type_def.clone(),
                                    v.clone(),
                                    var_defs.clone(),
                                    diagnostics,
                                )
                            } else {
                                diagnostics.push(unsupported_type!(db, val, type_def))
                            }
                        } else {
                            diagnostics.push(unsupported_type!(db, val, type_def))
                        }
                    })
                })
            }
            _ => diagnostics.push(unsupported_type!(db, val, type_def)),
        },
    };
}
