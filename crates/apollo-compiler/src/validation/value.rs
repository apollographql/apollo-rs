use crate::ast;
use crate::coordinate::TypeAttributeCoordinate;
use crate::parser::SourceSpan;
use crate::schema;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::variable::is_variable_usage_allowed;
use crate::validation::DiagnosticList;
use crate::Name;
use crate::Node;

/// The Argument, ObjectField, or other position into which a value (or nested value)
/// is being written.  Threaded through recursive value validation so the Variable arm
/// can apply spec rule 5.8.5 against the *innermost* position, and emit a
/// `DisallowedVariableUsage` diagnostic that names that position.
#[derive(Clone, Copy)]
pub(crate) struct VariablePosition<'a> {
    /// Position name — argument name at the top level, field name when recursing into
    /// an input object.
    name: &'a Name,
    /// Source location of the position's definition.
    location: Option<SourceSpan>,
    /// Whether the position declares a default value, per spec rule 5.8.5 step 3.b.
    has_default: bool,
}

fn unsupported_type(
    diagnostics: &mut DiagnosticList,
    value: &Node<ast::Value>,
    declared_type: &Node<ast::Type>,
) {
    diagnostics.push(
        value.location(),
        DiagnosticData::UnsupportedValueType {
            ty: declared_type.clone(),
            value: value.clone(),
            definition_location: declared_type.location(),
        },
    )
}

pub(crate) fn validate_values(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    arg_definition: &Node<ast::InputValueDefinition>,
    argument: &Node<ast::Argument>,
    var_defs: &[Node<ast::VariableDefinition>],
) {
    let position = VariablePosition {
        name: &arg_definition.name,
        location: arg_definition.location(),
        has_default: arg_definition.default_value.is_some(),
    };
    value_of_correct_type(
        diagnostics,
        schema,
        &arg_definition.ty,
        &argument.value,
        var_defs,
        Some(position),
    );
}

pub(crate) fn value_of_correct_type(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    ty: &Node<ast::Type>,
    arg_value: &Node<ast::Value>,
    var_defs: &[Node<ast::VariableDefinition>],
    position: Option<VariablePosition<'_>>,
) {
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
                        diagnostics.push(
                            arg_value.location(),
                            DiagnosticData::IntCoercionError {
                                value: int.as_str().to_owned(),
                            },
                        )
                    }
                }
                "Float" => {
                    if int.try_to_f64().is_err() {
                        diagnostics.push(
                            arg_value.location(),
                            DiagnosticData::FloatCoercionError {
                                value: int.as_str().to_owned(),
                            },
                        )
                    }
                }
                _ => unsupported_type(diagnostics, arg_value, ty),
            },
            _ => unsupported_type(diagnostics, arg_value, ty),
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
                    diagnostics.push(
                        arg_value.location(),
                        DiagnosticData::FloatCoercionError {
                            value: float.as_str().to_owned(),
                        },
                    )
                }
            }
            _ => unsupported_type(diagnostics, arg_value, ty),
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
                    unsupported_type(diagnostics, arg_value, ty);
                }
            }
            _ => unsupported_type(diagnostics, arg_value, ty),
        },
        // When expected as an input type, only boolean input values are
        // accepted. All other input values must raise a request error
        // indicating an incorrect type.
        ast::Value::Boolean(_) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) => {
                if scalar.is_built_in() && scalar.name.as_str() != "Boolean" {
                    unsupported_type(diagnostics, arg_value, ty);
                }
            }
            _ => unsupported_type(diagnostics, arg_value, ty),
        },
        ast::Value::Null => {
            if ty.is_non_null() {
                unsupported_type(diagnostics, arg_value, ty);
            }
        }
        ast::Value::Variable(var_name) => {
            if let Some(var_def) = var_defs.iter().find(|v| v.name == *var_name) {
                let location_has_default = position.is_some_and(|p| p.has_default);
                if !is_variable_usage_allowed(var_def, ty, location_has_default) {
                    if let Some(pos) = position {
                        diagnostics.push(
                            arg_value.location(),
                            DiagnosticData::DisallowedVariableUsage {
                                variable: var_name.clone(),
                                variable_type: (*var_def.ty).clone(),
                                variable_location: var_def.location(),
                                argument: pos.name.clone(),
                                argument_type: (**ty).clone(),
                                argument_location: pos.location,
                            },
                        );
                    } else {
                        unsupported_type(diagnostics, arg_value, ty);
                    }
                }
            } else {
                diagnostics.push(
                    arg_value.location(),
                    DiagnosticData::UndefinedVariable {
                        name: var_name.clone(),
                    },
                );
            }
        }
        ast::Value::Enum(value) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in() => {
                // Accept enum values as input for custom scalars
            }
            schema::ExtendedType::Enum(enum_) => {
                if !enum_.values.contains_key(value) {
                    diagnostics.push(
                        value.location(),
                        DiagnosticData::UndefinedEnumValue {
                            value: value.clone(),
                            definition: enum_.name.clone(),
                            definition_location: enum_.location(),
                        },
                    );
                }
            }
            _ => unsupported_type(diagnostics, arg_value, ty),
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
                unsupported_type(diagnostics, arg_value, ty)
            } else {
                let item_type = ty.same_location(ty.item_type().clone());
                if type_definition.is_input_type() {
                    for v in li {
                        // Per spec rule 5.8.5, a list value entry's location_ty is the
                        // item type — but it carries no default of its own.  Inherit the
                        // enclosing position's name/location so a nested variable error
                        // still names the surrounding argument or field.
                        let item_position = position.map(|p| VariablePosition {
                            has_default: false,
                            ..p
                        });
                        value_of_correct_type(
                            diagnostics,
                            schema,
                            &item_type,
                            v,
                            var_defs,
                            item_position,
                        );
                    }
                } else {
                    unsupported_type(diagnostics, arg_value, &item_type);
                }
            }
        }
        ast::Value::Object(obj) => match &type_definition {
            schema::ExtendedType::Scalar(scalar) if !scalar.is_built_in() => {}
            schema::ExtendedType::InputObject(input_obj) => {
                let undefined_field = obj
                    .iter()
                    .find(|(name, ..)| !input_obj.fields.contains_key(name));

                // Add a diagnostic if a value does not exist on the input
                // object type
                if let Some((name, value)) = undefined_field {
                    diagnostics.push(
                        value.location(),
                        DiagnosticData::UndefinedInputValue {
                            value: name.clone(),
                            definition: input_obj.name.clone(),
                            definition_location: input_obj.location(),
                        },
                    );
                }

                // @oneOf: exactly one key must be provided.
                // https://spec.graphql.org/draft/#sec-OneOf-Input-Objects
                //
                // Per-field non-null / nullable-variable checks fall out of the recursion
                // below — @oneOf field positions are treated as non-null (coerced when
                // recursing), so the generic Null and Variable arms catch the violations.
                if input_obj.is_one_of() && obj.len() != 1 {
                    diagnostics.push(
                        arg_value.location(),
                        DiagnosticData::OneOfInputObjectFieldCount {
                            name: input_obj.name.clone(),
                            provided_fields: obj
                                .iter()
                                .map(|(name, _)| (name.clone(), name.location()))
                                .collect(),
                        },
                    );
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
                        diagnostics.push(
                            arg_value.location(),
                            DiagnosticData::RequiredField {
                                name: input_name.clone(),
                                coordinate: TypeAttributeCoordinate {
                                    ty: input_obj.name.clone(),
                                    attribute: input_name.clone(),
                                },
                                expected_type: ty.clone(),
                                definition_location: f.location(),
                            },
                        );
                    }

                    let used_val = obj.iter().find(|(obj_name, ..)| obj_name == input_name);

                    if let Some((_, v)) = used_val {
                        let field_position = VariablePosition {
                            name: input_name,
                            location: f.location(),
                            has_default: f.default_value.is_some(),
                        };
                        // @oneOf: coerce the field's declared nullable type to non-null
                        // so the generic Null and Variable arms reject `null` literals
                        // and nullable variables at this position.
                        let coerced;
                        let effective_ty = if input_obj.is_one_of() && !ty.is_non_null() {
                            coerced = ty.same_location(ty.as_ref().clone().non_null());
                            &coerced
                        } else {
                            ty
                        };
                        value_of_correct_type(
                            diagnostics,
                            schema,
                            effective_ty,
                            v,
                            var_defs,
                            Some(field_position),
                        );
                    }
                })
            }
            _ => unsupported_type(diagnostics, arg_value, ty),
        },
    }
}
