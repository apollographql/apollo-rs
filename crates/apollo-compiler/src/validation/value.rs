use std::sync::Arc;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, TypeDefinition, Value},
    validation::ValidationDatabase,
};

pub fn value_of_correct_type(
    db: &dyn ValidationDatabase,
    type_def: hir::TypeDefinition,
    value: Value,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
) -> Result<(), Vec<ApolloDiagnostic>> {
    macro_rules! unsupported_type {
        ($db: expr, $value: expr, $type_def: expr) => {{
            ApolloDiagnostic::new(
                db,
                value.loc().into(),
                DiagnosticData::UnsupportedValueType {
                    value: value.value_name().into(),
                    ty: type_def.name().into(),
                },
            )
            .labels([
                Label::new(
                    type_def.loc(),
                    format!("field declared here as {} type", type_def.name()),
                ),
                Label::new(
                    value.loc(),
                    format!("argument declared here is of {} type", value.value_name()),
                ),
            ])
        }};
    }

    return match (value.clone(), type_def.clone()) {
        (
            Value::List(li),
            TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_),
        ) => {
            let errors: Vec<ApolloDiagnostic> = li
                .iter()
                .map(|val| {
                    db.value_of_correct_type(type_def.clone(), val.clone(), var_defs.clone())
                        .err()
                })
                .collect();
            if !errors.is_empty() {
                return Err(errors);
            } else {
                Ok(())
            }
        }
        (
            Value::Variable(var),
            TypeDefinition::EnumTypeDefinition(_)
            | TypeDefinition::ScalarTypeDefinition(_)
            | TypeDefinition::InputObjectTypeDefinition(_),
        ) => {
            let var_def = var_defs.iter().find(|v| v.name() == var.name());
            if let Some(var_def) = var_def {
                // we don't have the actual variable values here, so just
                // compare if two Types are the same
                if var_def.ty().name() != type_def.name() {
                    return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
                } else {
                    return Ok(());
                }
            } else {
                return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
            }
        }
        (Value::Float { .. }, TypeDefinition::ScalarTypeDefinition(scalar)) => {
            if !scalar.is_float() {
                return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
            }
            Ok(())
        }
        (Value::Int { .. }, TypeDefinition::ScalarTypeDefinition(scalar)) => {
            if !scalar.is_int() || !scalar.is_float() || !scalar.is_id() {
                return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
            }
            Ok(())
        }
        (Value::String { .. }, TypeDefinition::ScalarTypeDefinition(scalar)) => {
            if !scalar.is_string() || !scalar.is_id() {
                return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
            }
            Ok(())
        }
        (Value::Boolean { .. }, TypeDefinition::ScalarTypeDefinition(scalar)) => {
            if !scalar.is_boolean() {
                return Err(vec![unsupported_type!(db, value.clone(), type_def)]);
            }
            Ok(())
        }
        (
            Value::Null { .. },
            TypeDefinition::ScalarTypeDefinition(_) | TypeDefinition::EnumTypeDefinition(_),
        ) => Ok(()),
        (_, TypeDefinition::ScalarTypeDefinition(_)) => {
            return Err(vec![unsupported_type!(db, value.clone(), type_def)])
        }
        (
            Value::Enum {
                value: enum_val, ..
            },
            TypeDefinition::EnumTypeDefinition(enum_),
        ) => {
            let errors: Vec<ApolloDiagnostic> = enum_
                .values()
                .filter_map(|val| {
                    if val.enum_value() != enum_val.src() {
                        Some(ApolloDiagnostic::new(
                            db,
                            value.loc().into(),
                            DiagnosticData::UndefinedEnumValue {
                                value: val.enum_value().into(),
                                definition: type_def.name().into(),
                            },
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            if !errors.is_empty() {
                return Err(errors);
            } else {
                Ok(())
            }
        }
        (_, TypeDefinition::EnumTypeDefinition(_)) => {
            return Err(vec![unsupported_type!(db, value.clone(), type_def)])
        }
        (Value::Object(obj), TypeDefinition::InputObjectTypeDefinition(input_obj)) => input_obj
            .fields()
            .flat_map(|f| {
                obj.iter().map(|(name, val)| {
                    if f.name() == name.src() {
                        if let Some(type_def) = f.ty().type_def(db.upcast()) {
                            db.value_of_correct_type(type_def, val.clone(), var_defs.clone())
                        } else {
                            Err(vec![unsupported_type!(db, val.clone(), type_def)])
                        }
                    } else {
                        Err(vec![unsupported_type!(db, val.clone(), type_def)])
                    }
                })
            })
            .collect(),
        _ => return Err(vec![unsupported_type!(db, value.clone(), type_def)]),
    };
}
