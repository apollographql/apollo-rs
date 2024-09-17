use super::ExtendedType;
use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::schema::ScalarType;
use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definition;
use crate::validation::input_object::validate_input_object_definition;
use crate::validation::interface::validate_interface_definition;
use crate::validation::object::validate_object_type_definition;
use crate::validation::scalar::validate_scalar_definition;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definition;
use crate::validation::DiagnosticList;
use crate::Name;
use crate::Node;
use crate::Schema;
use std::sync::OnceLock;

pub(crate) fn validate_schema(errors: &mut DiagnosticList, schema: &mut Schema) {
    let mut builtin_scalars = BuiltInScalars::new();
    validate_schema_definition(errors, schema);
    validate_directive_definitions(errors, schema, &mut builtin_scalars);
    for def in schema.types.values() {
        match def {
            ExtendedType::Scalar(def) => validate_scalar_definition(errors, schema, def),
            ExtendedType::Object(def) => {
                validate_object_type_definition(errors, schema, &mut builtin_scalars, def)
            }
            ExtendedType::Interface(def) => {
                validate_interface_definition(errors, schema, &mut builtin_scalars, def)
            }
            ExtendedType::Union(def) => validate_union_definition(errors, schema, def),
            ExtendedType::Enum(def) => validate_enum_definition(errors, schema, def),
            ExtendedType::InputObject(def) => {
                validate_input_object_definition(errors, schema, &mut builtin_scalars, def)
            }
        }
    }
    // Remove definitions of unused built-in scalars
    if !builtin_scalars.all_used() {
        schema.types.retain(|name, def| {
            // Keep all custom (not built-in) definitions
            if !def.is_built_in() {
                return true;
            }

            // Keep built-in non-scalars (such as schema-introspection types)
            if !builtin_scalars.all.contains_key(name) {
                return true;
            }

            // Keep used definitions
            if builtin_scalars.used_and_defined.contains(name) {
                return true;
            }

            // Reached only for unused built-in scalars: remove
            false
        })
    }
    // Insert missing definitions
    for name in builtin_scalars.used_and_undefined {
        let def = &builtin_scalars.all[&name];
        schema
            .types
            .insert(def.name.clone(), ExtendedType::Scalar(def.clone()));
    }
}

/// Keeps track of usage of [built-in scalars] in a schema to determine which definitions
/// should be removed or added.
///
/// > When returning the set of types from the `__Schema` introspection type,
/// > all referenced built-in scalars must be included.
/// > If a built-in scalar type is not referenced anywhere in a schema
/// > (there is no field, argument, or input field of that type) then it must not be included.
///
/// We reflect this behavior of introspection in the `types` map of a `Valid<Schema>`.
///
/// [built-in scalars]: https://spec.graphql.org/draft/#sec-Scalars.Built-in-Scalars
pub(crate) struct BuiltInScalars {
    all: &'static HashMap<Name, Node<ScalarType>>,
    used_and_defined: HashSet<Name>,
    used_and_undefined: HashSet<Name>,
}

impl BuiltInScalars {
    fn new() -> Self {
        static ALL: OnceLock<HashMap<Name, Node<ScalarType>>> = OnceLock::new();
        let all = ALL.get_or_init(|| {
            super::SchemaBuilder::built_in()
                .schema
                .types
                .iter()
                .filter_map(|(name, def)| {
                    if let ExtendedType::Scalar(def) = def {
                        Some((name.clone(), def.clone()))
                    } else {
                        None
                    }
                })
                .collect()
        });
        Self {
            all,
            used_and_defined: HashSet::default(),
            used_and_undefined: HashSet::default(),
        }
    }

    /// Records a type reference to keep track of which built-in scalars are used in a schema,
    /// and returns whether this type name is for a built-in scalar
    pub(crate) fn record_type_ref(&mut self, schema: &Schema, name: &Name) -> bool {
        let is_built_in_scalar = self.all.contains_key(name);
        if is_built_in_scalar {
            if schema.types.contains_key(name) {
                self.used_and_defined.insert(name.clone());
            } else {
                self.used_and_undefined.insert(name.clone());
            }
        }
        is_built_in_scalar
    }

    fn all_used(&self) -> bool {
        let used_count = self.used_and_defined.len() + self.used_and_undefined.len();
        used_count == self.all.len()
    }
}
