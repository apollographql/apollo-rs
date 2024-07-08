//! Generating values that can appear both in schemas and in executable documents

use crate::arbitrary::entropy::Entropy;
use crate::executable;
use crate::schema;
use crate::schema::Value;
use crate::validation::Valid;
use crate::Name;
use crate::Node;
use crate::Schema;
use std::collections::HashMap;

pub(crate) fn arbitary_name(entropy: &mut Entropy<'_>) -> Name {
    // unwrap: `arbitary_name_string` should always generate valid GraphQL Name syntax
    Name::new(&arbitary_name_string(entropy)).unwrap()
}

fn arbitary_name_string(entropy: &mut Entropy<'_>) -> String {
    const NAME_START: &[u8; 53] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
    const NAME_CONTINUE: &[u8; 63] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789";
    let mut name = String::with_capacity(8);
    // unwrap: `NAME_START` and `NAME_CONTINUE` are not empty
    name.push(*entropy.choose(NAME_START).unwrap() as char);
    while entropy.bool() {
        name.push(*entropy.choose(NAME_CONTINUE).unwrap() as char);
    }
    name
}

/// Grab-bag of common parameters
pub(crate) struct Context<'a, 'b> {
    pub(crate) schema: &'a Valid<Schema>,
    pub(crate) directive_definitions_by_location: &'a DirectiveDefinitionsByLocation<'b>,
    pub(crate) entropy: &'a mut Entropy<'b>,
    pub(crate) variable_definitions: Option<&'a mut Vec<Node<executable::VariableDefinition>>>,
}

/// `variables` is `None` iff generating in a "const" context.
// Clippy false positive: https://github.com/rust-lang/rust-clippy/issues/13077
#[allow(clippy::needless_option_as_deref)]
pub(crate) fn arbitrary_arguments(
    context: &mut Context<'_, '_>,
    argument_definitions: &[Node<schema::InputValueDefinition>],
) -> Vec<Node<executable::Argument>> {
    let mut arguments = Vec::with_capacity(argument_definitions.len());
    for def in argument_definitions {
        let specified = def.is_required() || context.entropy.bool();
        if specified {
            arguments.push(
                executable::Argument {
                    name: def.name.clone(),
                    value: arbitrary_value(context, &def.ty).into(),
                }
                .into(),
            );
        }
    }
    arguments
}

/// `variables` is `None` iff generating a "const" value.
fn arbitrary_value(context: &mut Context<'_, '_>, expected_type: &schema::Type) -> Value {
    if !expected_type.is_non_null() {
        // Use null if entropy is exhausted
        let non_null = context.entropy.bool();
        if !non_null {
            return Value::Null;
        }
    }

    if let Some(variable_definitions) = &mut context.variable_definitions {
        let emit_variable = context.entropy.bool();
        if emit_variable {
            let new_variable = context.entropy.bool();
            if !new_variable {
                for var_def in variable_definitions.iter() {
                    if var_def.ty.is_assignable_to(expected_type) {
                        return Value::Variable(var_def.name.clone());
                    }
                }
            }
            let name = Name::try_from(format!("var{}", variable_definitions.len())).unwrap();
            let var_type = abritrary_type_assignable_to(context.entropy, expected_type);
            let var_def = executable::VariableDefinition {
                name: name.clone(),
                // No default if entropy is exhausted
                default_value: {
                    if context.entropy.bool() {
                        Some(arbitrary_value(context, &var_type).into())
                    } else {
                        None
                    }
                },
                ty: var_type.into(),
                directives: abritrary_directive_list(
                    context,
                    schema::DirectiveLocation::VariableDefinition,
                ),
            };
            context
                .variable_definitions
                .as_mut()
                .unwrap()
                .push(var_def.into());
            return Value::Variable(name);
        }
    }

    match expected_type {
        schema::Type::Named(name) | schema::Type::NonNullNamed(name) => {
            arbitrary_value_of_named_type(context, name)
        }
        schema::Type::List(inner) | schema::Type::NonNullList(inner) => {
            let mut list = Vec::new();
            while context.entropy.bool() {
                let item = arbitrary_value(context, inner);
                list.push(item.into())
            }
            Value::List(list)
        }
    }
}

fn abritrary_type_assignable_to(
    entropy: &mut Entropy<'_>,
    expected: &schema::Type,
) -> schema::Type {
    let generated = match expected {
        schema::Type::NonNullNamed(_) => expected.clone(),
        schema::Type::NonNullList(inner) => {
            schema::Type::NonNullList(Box::new(abritrary_type_assignable_to(entropy, inner)))
        }
        schema::Type::Named(name) => {
            if entropy.bool() {
                schema::Type::NonNullNamed(name.clone())
            } else {
                schema::Type::Named(name.clone())
            }
        }
        schema::Type::List(inner) => {
            let non_null = entropy.bool();
            let inner = Box::new(abritrary_type_assignable_to(entropy, inner));
            if non_null {
                schema::Type::NonNullList(inner)
            } else {
                schema::Type::List(inner)
            }
        }
    };
    assert!(generated.is_assignable_to(expected));
    generated
}

fn arbitrary_value_of_named_type(
    context: &mut Context<'_, '_>,
    expected_type: &schema::NamedType,
) -> Value {
    match &context.schema.types[expected_type] {
        schema::ExtendedType::Enum(def) => {
            let index = context
                .entropy
                .index(def.values.len())
                .expect("enum type with no values");
            Value::Enum(def.values[index].value.clone())
        }
        schema::ExtendedType::InputObject(def) => {
            let mut object = Vec::with_capacity(def.fields.len());
            for (name, field_def) in &def.fields {
                let specified = field_def.is_required() || context.entropy.bool();
                if specified {
                    let item = arbitrary_value(context, &field_def.ty);
                    object.push((name.clone(), item.into()));
                }
            }
            Value::Object(object)
        }
        schema::ExtendedType::Scalar(def) => match def.name.as_str() {
            "Int" | "ID" => Value::Int(context.entropy.i32().into()),
            "Float" => Value::Float(context.entropy.f64().into()),
            "String" => Value::String(arbitary_name_string(context.entropy)),
            "Boolean" => Value::Boolean(context.entropy.bool()),
            _ => Value::String("custom scalar".into()),
        },
        schema::ExtendedType::Object(_)
        | schema::ExtendedType::Interface(_)
        | schema::ExtendedType::Union(_) => {
            unreachable!("generating a GraphQL value of non-input type")
        }
    }
}

pub(crate) type DirectiveDefinitionsByLocation<'schema> =
    HashMap<schema::DirectiveLocation, Vec<&'schema schema::DirectiveDefinition>>;

pub(crate) fn gather_directive_definitions_by_location(
    schema: &Valid<Schema>,
) -> DirectiveDefinitionsByLocation<'_> {
    let mut by_location = DirectiveDefinitionsByLocation::new();
    for def in schema.directive_definitions.values() {
        for &location in &def.locations {
            by_location.entry(location).or_default().push(def)
        }
    }
    by_location
}

// Clippy false positive: https://github.com/rust-lang/rust-clippy/issues/13077
#[allow(clippy::needless_option_as_deref)]
pub(crate) fn abritrary_directive_list(
    context: &mut Context<'_, '_>,
    location: schema::DirectiveLocation,
) -> executable::DirectiveList {
    let Some(definitions) = context.directive_definitions_by_location.get(&location) else {
        // No directive definition for this location, generate an empty list
        return Default::default();
    };
    let mut list = executable::DirectiveList::new();
    // 75% of directive lists are empty. expected length: 0.33
    while context.entropy.u8() >= 192 {
        // unwrap: `gather_directive_definitions_by_location` only generates an entry
        // for at least one definition, so `definitions` is non-empty.
        let def = *context.entropy.choose(definitions).unwrap();
        if def.repeatable || !list.has(&def.name) {
            list.push(
                executable::Directive {
                    name: def.name.clone(),
                    arguments: arbitrary_arguments(context, &def.arguments),
                }
                .into(),
            );
        } else {
            // We already have this non-repeatable directive in this list
        }
    }
    list
}
