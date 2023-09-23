use crate::Arc;
use crate::{hir::*, FileId, HirDatabase};
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

pub(crate) fn types_definitions_by_name(
    db: &dyn HirDatabase,
) -> Arc<IndexMap<String, TypeDefinition>> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.type_definitions_by_name.clone();
    }
    let mut map = IndexMap::new();
    macro_rules! add {
        ($get: ident, $variant: ident) => {
            for (name, def) in db.$get().iter() {
                map.entry(name.clone())
                    .or_insert_with(|| TypeDefinition::$variant(def.clone()));
            }
        };
    }
    add!(scalars, ScalarTypeDefinition);
    add!(object_types_with_built_ins, ObjectTypeDefinition);
    add!(interfaces, InterfaceTypeDefinition);
    add!(unions, UnionTypeDefinition);
    add!(enums_with_built_ins, EnumTypeDefinition);
    add!(input_objects, InputObjectTypeDefinition);
    Arc::new(map)
}

pub(crate) fn find_type_definition_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<TypeDefinition> {
    db.types_definitions_by_name().get(&name).cloned()
}

pub(crate) fn find_operation(
    db: &dyn HirDatabase,
    file_id: FileId,
    name: Option<String>,
) -> Option<Arc<OperationDefinition>> {
    let ops = db.operations(file_id);

    if let Some(name) = name {
        ops.iter().find(|def| def.name() == Some(&*name)).cloned()
    } else if ops.len() == 1 {
        Some(ops[0].clone())
    } else {
        None
    }
}

pub(crate) fn find_fragment_by_name(
    db: &dyn HirDatabase,
    file_id: FileId,
    name: String,
) -> Option<Arc<FragmentDefinition>> {
    db.fragments(file_id).get(&name).cloned()
}

pub(crate) fn find_object_type_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types_with_built_ins().get(&name).cloned()
}

pub(crate) fn find_union_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<UnionTypeDefinition>> {
    db.unions().get(&name).cloned()
}

pub(crate) fn find_enum_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<EnumTypeDefinition>> {
    db.enums_with_built_ins().get(&name).cloned()
}

pub(crate) fn find_scalar_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<ScalarTypeDefinition>> {
    db.scalars().get(&name).cloned()
}

pub(crate) fn find_interface_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().get(&name).cloned()
}

pub(crate) fn find_directive_definition_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<DirectiveDefinition>> {
    db.directive_definitions().get(&name).cloned()
}

/// Find any definitions that use the specified directive.
pub(crate) fn find_types_with_directive(
    db: &dyn HirDatabase,
    directive: String,
) -> Arc<Vec<TypeDefinition>> {
    let definitions = db
        .types_definitions_by_name()
        .values()
        .filter(|def| {
            def.self_directives()
                .iter()
                .any(|dir| dir.name() == directive)
        })
        .cloned()
        .collect();
    Arc::new(definitions)
}

pub(crate) fn find_input_object_by_name(
    db: &dyn HirDatabase,
    name: String,
) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().get(&name).cloned()
}

pub(crate) fn query_operations(
    db: &dyn HirDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_query().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

pub(crate) fn subscription_operations(
    db: &dyn HirDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_subscription().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

pub(crate) fn mutation_operations(
    db: &dyn HirDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_mutation().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

pub(crate) fn operation_fields(
    _db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Arc<Vec<Field>> {
    let fields = selection_set
        .selection()
        .iter()
        .filter_map(|sel| match sel {
            Selection::Field(field) => Some(field.as_ref().clone()),
            _ => None,
        })
        .collect();
    Arc::new(fields)
}

pub(crate) fn operation_inline_fragment_fields(
    _db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Arc<Vec<Field>> {
    let fields = selection_set
        .selection()
        .iter()
        .filter_map(|sel| match sel {
            Selection::InlineFragment(fragment) => {
                let fields: Vec<Field> = fragment.selection_set().fields();
                Some(fields)
            }
            _ => None,
        })
        .flatten()
        .collect();
    Arc::new(fields)
}

pub(crate) fn operation_fragment_references(
    db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Arc<Vec<Arc<FragmentDefinition>>> {
    let fields = selection_set
        .selection()
        .iter()
        .filter_map(|sel| match sel {
            Selection::FragmentSpread(fragment_spread) => Some(fragment_spread.fragment(db)?),
            _ => None,
        })
        .collect();
    Arc::new(fields)
}

pub(crate) fn operation_fragment_spread_fields(
    db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Arc<Vec<Field>> {
    let fields = selection_set
        .selection()
        .iter()
        .filter_map(|sel| match sel {
            Selection::FragmentSpread(fragment_spread) => {
                let fields: Vec<Field> = fragment_spread.fragment(db)?.selection_set().fields();
                Some(fields)
            }
            _ => None,
        })
        .flatten()
        .collect();
    Arc::new(fields)
}

pub(crate) fn flattened_operation_fields(
    db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Vec<Arc<Field>> {
    fn flatten_selection_set(
        db: &dyn HirDatabase,
        selection_set: &SelectionSet,
        seen: &mut HashSet<SelectionSet>,
    ) -> Vec<Arc<Field>> {
        if seen.contains(selection_set) {
            return vec![];
        }
        seen.insert(selection_set.clone());

        selection_set
            .selection()
            .iter()
            .flat_map(|sel| match sel {
                Selection::Field(field) => {
                    vec![Arc::clone(field)]
                }
                Selection::FragmentSpread(fragment_spread) => fragment_spread
                    .fragment(db)
                    .map(|fragment| flatten_selection_set(db, fragment.selection_set(), seen))
                    .unwrap_or_default(),
                Selection::InlineFragment(fragment_spread) => {
                    flatten_selection_set(db, fragment_spread.selection_set(), seen)
                }
            })
            .collect()
    }

    flatten_selection_set(db, &selection_set, &mut HashSet::new())
}

// Should be part of operation's db
// NOTE: potentially want to return a hashmap of variables and their types?
pub(crate) fn selection_variables(
    db: &dyn HirDatabase,
    selection_set: SelectionSet,
) -> Arc<HashSet<Variable>> {
    let vars = db
        .operation_fields(selection_set)
        .iter()
        .flat_map(|field| {
            let vars: Vec<_> = field
                .arguments()
                .iter()
                .flat_map(|arg| get_field_variable_value(arg.value.clone()))
                .collect();
            vars
        })
        .collect();
    Arc::new(vars)
}

pub(crate) fn get_field_variable_value(val: Value) -> Vec<Variable> {
    match val {
        Value::Variable(var) => vec![var],
        Value::List { value: list, .. } => list
            .iter()
            .flat_map(|val| get_field_variable_value(val.clone()))
            .collect(),
        Value::Object { value: obj, .. } => obj
            .iter()
            .flat_map(|val| get_field_variable_value(val.1.clone()))
            .collect(),
        _ => Vec::new(),
    }
}

// should be part of operation's db
// NOTE: potentially want to return a hashset of variables and their types?
pub(crate) fn operation_definition_variables(
    _db: &dyn HirDatabase,
    variables: Arc<Vec<VariableDefinition>>,
) -> Arc<HashSet<Variable>> {
    let vars = variables
        .iter()
        .map(|v| Variable {
            name: v.name().to_owned(),
            loc: v.loc(),
        })
        .collect();
    Arc::new(vars)
}

pub(crate) fn subtype_map(db: &dyn HirDatabase) -> Arc<HashMap<String, HashSet<String>>> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.subtype_map.clone();
    }
    let mut map = HashMap::<String, HashSet<String>>::new();
    let mut add = |key: &str, value: &str| {
        map.entry(key.to_owned())
            .or_default()
            .insert(value.to_owned())
    };
    for (name, definition) in &*db.object_types_with_built_ins() {
        for implements in definition.self_implements_interfaces() {
            add(implements.interface(), name);
        }
        for extension in definition.extensions() {
            for implements in extension.implements_interfaces() {
                add(implements.interface(), name);
            }
        }
    }
    for (name, definition) in &*db.interfaces() {
        for implements in definition.self_implements_interfaces() {
            add(implements.interface(), name);
        }
        for extension in definition.extensions() {
            for implements in extension.implements_interfaces() {
                add(implements.interface(), name);
            }
        }
    }
    for (name, definition) in &*db.unions() {
        for member in definition.self_members() {
            add(name, member.name());
        }
        for extension in definition.extensions() {
            for member in extension.members() {
                add(name, member.name());
            }
        }
    }
    Arc::new(map)
}

pub(crate) fn is_subtype(
    db: &dyn HirDatabase,
    abstract_type: String,
    maybe_subtype: String,
) -> bool {
    db.subtype_map()
        .get(&abstract_type)
        .map_or(false, |set| set.contains(&maybe_subtype))
}
