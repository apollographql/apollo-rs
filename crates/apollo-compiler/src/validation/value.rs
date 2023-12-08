use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData},
    schema,
    validation::ValidationDatabase,
    Node,
};

fn unsupported_type(value: &Node<ast::Value>, declared_type: &Node<ast::Type>) -> ApolloDiagnostic {
    ApolloDiagnostic::new(
        value.location(),
        DiagnosticData::UnsupportedValueType {
            value: value.kind().into(),
            ty: declared_type.to_string(),
            definition_location: declared_type.location(),
        },
    )
}

pub(crate) fn validate_values(
    db: &dyn ValidationDatabase,
    ty: &Node<ast::Type>,
    argument: &Node<ast::Argument>,
    var_defs: &[Node<ast::VariableDefinition>],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = vec![];
    value_of_correct_type(db, ty, &argument.value, var_defs, &mut diagnostics);
    diagnostics
}

pub(crate) fn value_of_correct_type(
    db: &dyn ValidationDatabase,
    ty: &Node<ast::Type>,
    arg_value: &Node<ast::Value>,
    var_defs: &[Node<ast::VariableDefinition>],
    diagnostics: &mut Vec<ApolloDiagnostic>,
) {
    let schema = db.schema();
    let Some(type_definition) = schema.types.get(ty.inner_named_type()) else {
        return;
    };

    match &**arg_value {
        // When expected as an input type, only integer input values are
        // accepted. All other input values, including strings with numeric
        // content, must raise a request error indicating an incorrect
        // type. If the integer input value represents a value less than
        // -2^31 or greater than or equal to 2^31, a request error should be
        // raised.
        // When expected as an input type, any string (such as "4") or
        // integer (such as 4 or -4) input value should be coerced to ID
        ast::Value::Int(int) => match &type_definition {
            // Any value is valid for a custom scalar.
            schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in() => {}
            schema::ExtendedType::Scalar(scalar) => match scalar.name.as_str() {
                // Any integer sequence is valid for an ID.
                "ID" => {}
                "Int" => {
                    if int.try_to_i32().is_err() {
                        diagnostics.push(ApolloDiagnostic::new(
                            arg_value.location(),
                            DiagnosticData::IntCoercionError {
                                value: int.as_str().to_owned(),
                            },
                        ))
                    }
                }
                "Float" => {
                    if int.try_to_f64().is_err() {
                        diagnostics.push(ApolloDiagnostic::new(
                            arg_value.location(),
                            DiagnosticData::FloatCoercionError {
                                value: int.as_str().to_owned(),
                            },
                        ))
                    }
                }
                _ => diagnostics.push(unsupported_type(arg_value, ty)),
            },
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        // When expected as an input type, both integer and float input
        // values are accepted. All other input values, including strings
        // with numeric content, must raise a request error indicating an
        // incorrect type.
        ast::Value::Float(float) => match &type_definition {
            // Any value is valid for a custom scalar.
            schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in() => {}
            schema::ExtendedType::Scalar(scalar) if scalar.name == "Float" => {
                if float.try_to_f64().is_err() {
                    diagnostics.push(ApolloDiagnostic::new(
                        arg_value.location(),
                        DiagnosticData::FloatCoercionError {
                            value: float.as_str().to_owned(),
                        },
                    ))
                }
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        // When expected as an input type, only valid Unicode string input
        // values are accepted. All other input values must raise a request
        // error indicating an incorrect type.
        // When expected as an input type, any string (such as "4") or
        // integer (such as 4 or -4) input value should be coerced to ID
        ast::Value::String(_) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) => {
                // specifically return diagnostics for ints, floats, and
                // booleans.
                // string, ids and custom scalars are ok, and
                // don't need a diagnostic.
                if scalar.is_built_in() && !matches!(scalar.name.as_str(), "String" | "ID") {
                    diagnostics.push(unsupported_type(arg_value, ty));
                }
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        // When expected as an input type, only boolean input values are
        // accepted. All other input values must raise a request error
        // indicating an incorrect type.
        ast::Value::Boolean(_) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) => {
                if scalar.is_built_in() && scalar.name.as_str() != "Boolean" {
                    diagnostics.push(unsupported_type(arg_value, ty));
                }
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        ast::Value::Null => {
            if ty.is_non_null() {
                diagnostics.push(unsupported_type(arg_value, ty));
            }
        }
        ast::Value::Variable(var_name) => match &type_definition {
            schema::ExtendedType::Scalar(_)
            | schema::ExtendedType::Enum(_)
            | schema::ExtendedType::InputObject(_) => {
                let var_def = var_defs.iter().find(|v| v.name == *var_name);
                if let Some(var_def) = var_def {
                    // we don't have the actual variable values here, so just
                    // compare if two Types are the same
                    // TODO(@goto-bus-stop) This should use the is_assignable_to check
                    if var_def.ty.inner_named_type() != ty.inner_named_type() {
                        diagnostics.push(unsupported_type(arg_value, ty));
                    } else if let Some(default_value) = &var_def.default_value {
                        if var_def.ty.is_non_null() && default_value.is_null() {
                            diagnostics.push(unsupported_type(default_value, &var_def.ty))
                        } else {
                            value_of_correct_type(
                                db,
                                &var_def.ty,
                                default_value,
                                var_defs,
                                diagnostics,
                            )
                        }
                    }
                }
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        ast::Value::Enum(value) => match &type_definition {
            schema::ExtendedType::Enum(enum_) => {
                if !enum_.values.contains_key(value) {
                    diagnostics.push(ApolloDiagnostic::new(
                        value.location(),
                        DiagnosticData::UndefinedEnumValue {
                            value: value.to_string(),
                            definition: enum_.name.to_string(),
                            definition_location: enum_.location(),
                        },
                    ));
                }
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
        // When expected as an input, list values are accepted only when
        // each item in the list can be accepted by the list’s item type.
        //
        // If the value passed as an input to a list type is not a list and
        // not the null value, then the result of input coercion is a list
        // of size one, where the single item value is the result of input
        // coercion for the list’s item type on the provided value (note
        // this may apply recursively for nested lists).
        ast::Value::List(li) => {
            let accepts_list = ty.is_list()
                // A named type can still accept a list if it is a custom scalar.
                || matches!(type_definition, schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in());
            if !accepts_list {
                diagnostics.push(unsupported_type(arg_value, ty))
            } else {
                let item_type = ty.same_location(ty.item_type().clone());
                if type_definition.is_input_type() {
                    for v in li {
                        value_of_correct_type(db, &item_type, v, var_defs, diagnostics);
                    }
                } else {
                    diagnostics.push(unsupported_type(arg_value, &item_type));
                }
            }
        }
        ast::Value::Object(obj) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in() => (),
            schema::ExtendedType::InputObject(input_obj) => {
                let undefined_field = obj
                    .iter()
                    .find(|(name, ..)| !input_obj.fields.contains_key(name));

                // Add a diagnostic if a value does not exist on the input
                // object type
                if let Some((name, value)) = undefined_field {
                    diagnostics.push(ApolloDiagnostic::new(
                        value.location(),
                        DiagnosticData::UndefinedInputValue {
                            value: name.to_string(),
                            definition: input_obj.name.to_string(),
                            definition_location: input_obj.location(),
                        },
                    ));
                }

                input_obj.fields.iter().for_each(|(input_name, f)| {
                    let ty = &f.ty;
                    let is_missing = !obj.iter().any(|(value_name, ..)| input_name == value_name);
                    let is_null = obj
                        .iter()
                        .any(|(name, value)| input_name == name && value.is_null());

                    // If the input object field type is non_null, and no
                    // default value is provided, or if the value provided
                    // is null or missing entirely, an error should be
                    // raised.
                    if (ty.is_non_null() && f.default_value.is_none()) && (is_missing || is_null) {
                        diagnostics.push(ApolloDiagnostic::new(
                            arg_value.location(),
                            DiagnosticData::RequiredArgument {
                                name: input_name.to_string(),
                                coordinate: format!("{}.{}", input_obj.name, input_name),
                                definition_location: f.location(),
                            },
                        ));
                    }

                    let used_val = obj.iter().find(|(obj_name, ..)| obj_name == input_name);

                    if let Some((_, v)) = used_val {
                        value_of_correct_type(db, ty, v, var_defs, diagnostics);
                    }
                })
            }
            _ => diagnostics.push(unsupported_type(arg_value, ty)),
        },
    }
}
