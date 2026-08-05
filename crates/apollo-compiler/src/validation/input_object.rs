use crate::ast;
use crate::collections::HashMap;
use crate::coordinate::TypeAttributeCoordinate;
use crate::schema::validation::BuiltInScalars;
use crate::schema::InputObjectType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::CycleError;
use crate::validation::DiagnosticList;
use crate::validation::RecursionGuard;
use crate::validation::RecursionStack;
use crate::Name;
use crate::Node;

// Implements [Circular References](https://spec.graphql.org/September2025/#sec-Input-Objects.Circular-References)
// part of the input object validation spec.
struct FindRecursiveInputValue<'a> {
    schema: &'a crate::Schema,
}

impl FindRecursiveInputValue<'_> {
    fn input_value_definition(
        &self,
        seen: &mut RecursionGuard<'_>,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        match &*def.ty {
            // NonNull type followed by Named type is the one that's not allowed
            // to be cyclical, so this is only case we care about.
            //
            // Everything else may be a cyclical input value.
            ast::Type::NonNullNamed(name) => {
                if !seen.contains(name) {
                    if let Some(object_def) = self.schema.get_input_object(name) {
                        self.input_object_definition(seen.push(name)?, object_def)
                            .map_err(|err| err.trace(def))?
                    }
                } else if seen.first() == Some(name) {
                    return Err(CycleError::Recursed(vec![def.clone()]));
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionGuard<'_>,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        for input_value in input_object.fields.values() {
            self.input_value_definition(&mut seen, input_value)?;
        }

        Ok(())
    }

    fn check(
        schema: &crate::Schema,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let mut recursion_stack = RecursionStack::with_root(input_object.name.clone());
        FindRecursiveInputValue { schema }
            .input_object_definition(recursion_stack.guard(), input_object)
    }
}

// Catches cycles involving @oneOf that FindRecursiveInputValue misses.
//
// A field is an "unbreakable link" if it's NonNull, or if its parent is @oneOf
// (making it semantically non-null). For @oneOf types, a cycle is fatal only
// when *every* field leads into it (you pick one, so one escape suffices).
// For regular types, *any* single unbreakable link is fatal.
struct FindOneOfCycle<'a> {
    schema: &'a crate::Schema,
}

impl FindOneOfCycle<'_> {
    fn input_value_definition(
        &self,
        seen: &mut RecursionGuard<'_>,
        is_one_of: bool,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let name = match &*def.ty {
            ast::Type::NonNullNamed(name) => name,
            ast::Type::Named(name) if is_one_of => name,
            _ => return Ok(()),
        };

        if !seen.contains(name) {
            if let Some(object_def) = self.schema.get_input_object(name) {
                self.input_object_definition(seen.push(name)?, object_def)
                    .map_err(|err| err.trace(def))?
            }
        } else if seen.first() == Some(name) {
            return Err(CycleError::Recursed(vec![def.clone()]));
        }

        Ok(())
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionGuard<'_>,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let is_one_of = input_object.is_one_of();
        if is_one_of {
            let mut last_err = None;
            for field in input_object.fields.values() {
                match self.input_value_definition(&mut seen, is_one_of, field) {
                    Err(e) => last_err = Some(e),
                    Ok(()) => return Ok(()),
                }
            }
            last_err.map_or(Ok(()), Err)
        } else {
            for field in input_object.fields.values() {
                self.input_value_definition(&mut seen, is_one_of, field)?;
            }
            Ok(())
        }
    }

    fn check(
        schema: &crate::Schema,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let mut recursion_stack = RecursionStack::with_root(input_object.name.clone());
        FindOneOfCycle { schema }.input_object_definition(recursion_stack.guard(), input_object)
    }
}

/// Implements [InputObjectDefaultValueHasCycle](https://spec.graphql.org/September2025/#InputObjectDefaultValueHasCycle())
/// from input object type validation: default values must not form a cycle
/// where coercing one field's default (transitively, through omitted fields
/// of nested input objects) requires coercing that same default again.
///
/// Returns the field definition whose default value closes a cycle, if any.
fn input_object_default_value_has_cycle<'s>(
    schema: &'s crate::Schema,
    input_object: &'s InputObjectType,
    default_value: Option<&'s ast::Value>,
    visited_fields: &mut Vec<(&'s Name, &'s Name)>,
) -> Option<&'s Node<ast::InputValueDefinition>> {
    match default_value {
        Some(ast::Value::List(items)) => items.iter().find_map(|item| {
            input_object_default_value_has_cycle(schema, input_object, Some(item), visited_fields)
        }),
        // A missing default value is treated as an empty map: coercion would
        // fall back to the default value of every field.
        Some(ast::Value::Object(_)) | None => {
            let object = match default_value {
                Some(ast::Value::Object(object)) => &object[..],
                _ => &[],
            };
            input_object.fields.values().find_map(|field| {
                input_field_default_value_has_cycle(
                    schema,
                    &input_object.name,
                    field,
                    object,
                    visited_fields,
                )
            })
        }
        Some(_) => None,
    }
}

/// Implements [InputFieldDefaultValueHasCycle](https://spec.graphql.org/September2025/#InputFieldDefaultValueHasCycle())
fn input_field_default_value_has_cycle<'s>(
    schema: &'s crate::Schema,
    input_object_name: &'s Name,
    field: &'s Node<ast::InputValueDefinition>,
    default_value: &'s [(Name, Node<ast::Value>)],
    visited_fields: &mut Vec<(&'s Name, &'s Name)>,
) -> Option<&'s Node<ast::InputValueDefinition>> {
    let named_field_type = field.ty.inner_named_type();
    let field_type = schema.get_input_object(named_field_type)?;
    if let Some((_, provided)) = default_value.iter().find(|(name, _)| *name == field.name) {
        // An explicitly provided value never triggers this field's own default,
        // so the field is not added to the visited set.
        input_object_default_value_has_cycle(schema, field_type, Some(provided), visited_fields)
    } else {
        let field_default = field.default_value.as_deref()?;
        let key = (input_object_name, &field.name);
        if visited_fields.contains(&key) {
            return Some(field);
        }
        visited_fields.push(key);
        let result = input_object_default_value_has_cycle(
            schema,
            field_type,
            Some(field_default),
            visited_fields,
        );
        visited_fields.pop();
        result
    }
}

pub(crate) fn validate_input_object_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    input_object: &Node<InputObjectType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        input_object.directives.iter_ast(),
        ast::DirectiveLocation::InputObject,
        // input objects don't use variables
        Default::default(),
    );

    match FindRecursiveInputValue::check(schema, input_object) {
        Ok(_) => match FindOneOfCycle::check(schema, input_object) {
            Ok(_) => {}
            Err(CycleError::Recursed(trace)) => diagnostics.push(
                input_object.location(),
                DiagnosticData::RecursiveInputObjectDefinition {
                    name: input_object.name.clone(),
                    trace,
                },
            ),
            Err(CycleError::Limit(_)) => {
                diagnostics.push(
                    input_object.location(),
                    DiagnosticData::DeeplyNestedType {
                        name: input_object.name.clone(),
                        describe_type: "input object",
                    },
                );
            }
        },
        Err(CycleError::Recursed(trace)) => diagnostics.push(
            input_object.location(),
            DiagnosticData::RecursiveInputObjectDefinition {
                name: input_object.name.clone(),
                trace,
            },
        ),
        Err(CycleError::Limit(_)) => {
            diagnostics.push(
                input_object.location(),
                DiagnosticData::DeeplyNestedType {
                    name: input_object.name.clone(),
                    describe_type: "input object",
                },
            );
        }
    }

    // @oneOf must not be provided by an input object type extension.
    // https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
    for directive in &input_object.directives.0 {
        if directive.name == "oneOf" {
            if let Some(ext_id) = directive.origin.extension_id() {
                diagnostics.push(
                    directive.location(),
                    DiagnosticData::OneOfDirectiveOnExtension {
                        type_name: input_object.name.clone(),
                        extension_location: ext_id.location(),
                    },
                );
            }
        }
    }

    // @oneOf input objects: all fields must be nullable and must not have default values.
    // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects
    if input_object.is_one_of() {
        for (field_name, field) in &input_object.fields {
            if field.ty.is_non_null() {
                diagnostics.push(
                    field.location(),
                    DiagnosticData::OneOfInputObjectFieldNonNull {
                        coordinate: TypeAttributeCoordinate {
                            ty: input_object.name.clone(),
                            attribute: field_name.clone(),
                        },
                        definition_location: field.location(),
                    },
                );
            }
            if field.default_value.is_some() {
                let default_location = field.default_value.as_ref().and_then(|v| v.location());
                diagnostics.push(
                    field.location(),
                    DiagnosticData::UnsupportedDefault {
                        coordinate: TypeAttributeCoordinate {
                            ty: input_object.name.clone(),
                            attribute: field_name.clone(),
                        },
                        default_location,
                    },
                );
            }
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    let fields: Vec<_> = input_object
        .fields
        .values()
        .map(|c| c.node.clone())
        .collect();
    validate_input_value_definitions(
        diagnostics,
        schema,
        built_in_scalars,
        &fields,
        ast::DirectiveLocation::InputFieldDefinition,
        "an input object field",
    );

    // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
    // > InputObjectDefaultValueHasCycle(inputObject) must be false.
    if let Some(field) =
        input_object_default_value_has_cycle(schema, input_object, None, &mut Vec::new())
    {
        diagnostics.push(
            field.location(),
            DiagnosticData::RecursiveInputObjectDefaultValue {
                type_name: input_object.name.clone(),
                field_name: field.name.clone(),
                default_value_location: field.default_value.as_ref().and_then(|v| v.location()),
            },
        );
    }

    // validate there is at least one input value on the input object type
    // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
    if input_object.fields.is_empty() {
        diagnostics.push(
            input_object.location(),
            DiagnosticData::EmptyInputValueSet {
                type_name: input_object.name.clone(),
                type_location: input_object.location(),
                extensions_locations: input_object
                    .extensions()
                    .iter()
                    .map(|ext| ext.location())
                    .collect(),
            },
        );
    }
}

pub(crate) fn validate_argument_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) {
    validate_input_value_definitions(
        diagnostics,
        schema,
        built_in_scalars,
        input_values,
        directive_location,
        "an argument",
    );

    let mut seen: HashMap<Name, &Node<ast::InputValueDefinition>> = HashMap::default();
    for input_value in input_values {
        let name = &input_value.name;
        if let Some(prev_value) = seen.get(name) {
            let (original_definition, redefined_definition) =
                (prev_value.location(), input_value.location());

            diagnostics.push(
                original_definition,
                DiagnosticData::UniqueInputValue {
                    name: name.clone(),
                    original_definition,
                    redefined_definition,
                },
            );
        } else {
            seen.insert(name.clone(), input_value);
        }
    }
}

pub(crate) fn validate_input_value_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
    describe: &'static str,
) {
    for input_value in input_values {
        crate::schema::validation::validate_type_system_name(
            diagnostics,
            &input_value.name,
            describe,
        );
        super::directive::validate_directives(
            diagnostics,
            Some(schema),
            input_value.directives.iter(),
            directive_location,
            Default::default(), // No variables in an input value definition
        );
        // https://spec.graphql.org/September2025/#sec--deprecated
        // > The @deprecated directive must not appear on required (non-null
        // > without a default) arguments or input object field definitions.
        if input_value.ty.is_non_null() && input_value.default_value.is_none() {
            if let Some(deprecated) = input_value.directives.get("deprecated") {
                diagnostics.push(
                    deprecated.location(),
                    DiagnosticData::DeprecatedRequiredInputValue {
                        name: input_value.name.clone(),
                        describe,
                        definition_location: input_value.location(),
                    },
                );
            }
        }
        // Input values must only contain input types.
        let loc = input_value.location();
        let named_type = input_value.ty.inner_named_type();
        let is_built_in = built_in_scalars.record_type_ref(schema, named_type);
        if let Some(field_ty) = schema.types.get(named_type) {
            if !field_ty.is_input_type() {
                diagnostics.push(
                    loc,
                    DiagnosticData::InputType {
                        name: input_value.name.clone(),
                        describe_type: field_ty.describe(),
                        type_location: input_value.ty.location(),
                    },
                );
            }
            // https://spec.graphql.org/September2025/#sec-Objects.Type-Validation
            // > If the argument has a default value it must be compatible with
            // > argumentType as per the coercion rules for that type.
            if let Some(default) = &input_value.default_value {
                let var_defs = &[];
                super::value::value_of_correct_type(
                    diagnostics,
                    schema,
                    &input_value.ty,
                    default,
                    var_defs,
                    None,
                );
            }
        } else if is_built_in {
            // `validate_schema()` will insert the missing definition
        } else {
            let loc = named_type.location();
            diagnostics.push(
                loc,
                DiagnosticData::UndefinedDefinition {
                    name: named_type.clone(),
                },
            );
        }
    }
}
