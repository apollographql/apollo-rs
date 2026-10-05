use crate::ast;
use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::coordinate::TypeAttributeCoordinate;
use crate::schema::validation::BuiltInScalars;
use crate::schema::ExtendedType;
use crate::schema::InputObjectType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::value::value_of_correct_type;
use crate::validation::DiagnosticList;
use crate::Name;
use crate::Node;

/// Finds the input objects for which no finite value can be constructed.
///
/// These are exactly the input objects for which `InputObjectHasUnbreakableCycle()`
/// returns true. The spec's depth-first search has to backtrack through every
/// field of every @oneOf, which is exponential in the length of the chain. This
/// instead propagates constructibility from the input objects that need nothing
/// else, visiting each field once.
pub(crate) fn unconstructible_input_objects(schema: &crate::Schema) -> HashSet<Name> {
    let mut remaining = HashMap::default();
    let mut dependents: HashMap<&Name, Vec<&Name>> = HashMap::default();
    let mut ready = Vec::new();

    for input_object in schema.types.values().filter_map(|ty| match ty {
        ExtendedType::InputObject(def) => Some(def),
        _ => None,
    }) {
        let (required, count) = required_input_objects(schema, input_object);
        for field_type in required {
            dependents
                .entry(field_type)
                .or_default()
                .push(&input_object.name);
        }
        if count == 0 {
            ready.push(&input_object.name);
        } else {
            remaining.insert(&input_object.name, count);
        }
    }

    while let Some(name) = ready.pop() {
        for &dependent in dependents.get(name).into_iter().flatten() {
            // Input objects leave `remaining` once they are constructible.
            let Some(count) = remaining.get_mut(dependent) else {
                continue;
            };
            *count -= 1;
            if *count == 0 {
                remaining.remove(dependent);
                ready.push(dependent);
            }
        }
    }
    remaining.into_keys().cloned().collect()
}

/// Returns the input objects that `input_object`'s fields depend on, and how
/// many of them must become constructible before `input_object` is.
///
/// A regular input object needs every non-null input object field. A @oneOf
/// only needs one field, and needs nothing if any field already has a value
/// without another input object.
fn required_input_objects<'a>(
    schema: &crate::Schema,
    input_object: &'a InputObjectType,
) -> (Vec<&'a Name>, usize) {
    let is_one_of = input_object.is_one_of();
    let mut required = Vec::new();
    // An empty @oneOf is reported separately as an empty input object.
    let mut has_escape = input_object.fields.is_empty();
    for field in input_object.fields.values() {
        if !is_one_of && !field.ty.is_non_null() {
            continue;
        }
        match unbreakable_field_type(schema, field) {
            Some(field_type) => required.push(field_type),
            None => has_escape = true,
        }
    }
    let count = if !is_one_of {
        required.len()
    } else if has_escape {
        0
    } else {
        1
    };
    (required, count)
}

/// Returns the input object a field requires a value of, if any. Lists can
/// always be empty, and scalars and enums always have a value, so neither can
/// be part of a cycle.
fn unbreakable_field_type<'a>(
    schema: &crate::Schema,
    field: &'a ast::InputValueDefinition,
) -> Option<&'a Name> {
    let (ast::Type::Named(name) | ast::Type::NonNullNamed(name)) = &*field.ty else {
        return None;
    };
    schema.get_input_object(name).map(|_| name)
}

/// Follows fields into non-constructible input objects until one repeats, to
/// show why `input_object` cannot be constructed. The trace is ordered from the
/// field that closes the cycle back to the field on `input_object`.
fn unbreakable_cycle_trace(
    schema: &crate::Schema,
    unconstructible: &HashSet<Name>,
    input_object: &InputObjectType,
) -> Vec<Node<ast::InputValueDefinition>> {
    let leads_to_cycle = |field: &&Node<ast::InputValueDefinition>| {
        unbreakable_field_type(schema, field).is_some_and(|name| unconstructible.contains(name))
    };
    let mut seen = HashSet::default();
    seen.insert(input_object.name.clone());
    let mut trace = Vec::new();
    let mut current = input_object;
    loop {
        let mut fields = current.fields.values();
        let next = if current.is_one_of() {
            fields.rfind(leads_to_cycle)
        } else {
            fields.find(|field| field.ty.is_non_null() && leads_to_cycle(field))
        };
        let Some(field) = next else { break };
        trace.push(field.clone());
        let name = field.ty.inner_named_type();
        if !seen.insert(name.clone()) {
            break;
        }
        let Some(next_object) = schema.get_input_object(name) else {
            break;
        };
        current = next_object;
    }
    trace.reverse();
    trace
}

pub(crate) fn validate_input_object_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    unconstructible_input_objects: &HashSet<Name>,
    input_object: &Node<InputObjectType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        input_object.directives.iter(),
        ast::DirectiveLocation::InputObject,
        // input objects don't use variables
        Default::default(),
    );

    if unconstructible_input_objects.contains(&input_object.name) {
        diagnostics.push(
            input_object.location(),
            DiagnosticData::RecursiveInputObjectDefinition {
                name: input_object.name.clone(),
                trace: unbreakable_cycle_trace(schema, unconstructible_input_objects, input_object),
            },
        );
    }

    // @oneOf must not be provided by an input object type extension.
    // https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
    for directive in &input_object.directives.0 {
        if directive.name == "oneOf" {
            if let Some(ext_id) = directive.extension_id() {
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
    let fields: Vec<_> = input_object.fields.values().cloned().collect();
    validate_input_value_definitions(
        diagnostics,
        schema,
        built_in_scalars,
        &fields,
        ast::DirectiveLocation::InputFieldDefinition,
        "an input object field",
    );

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
            if schema.validate_default_values {
                if let Some(default) = &input_value.default_value {
                    let var_defs = &[];
                    value_of_correct_type(
                        diagnostics,
                        schema,
                        &input_value.ty,
                        default,
                        var_defs,
                        None,
                    );
                }
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
