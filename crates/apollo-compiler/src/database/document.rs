use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{hir::*, FileId, HirDatabase};
use indexmap::IndexMap;

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
    add!(object_types, ObjectTypeDefinition);
    add!(interfaces, InterfaceTypeDefinition);
    add!(unions, UnionTypeDefinition);
    add!(enums, EnumTypeDefinition);
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
    db.operations(file_id)
        .iter()
        .find(|def| def.name() == name.as_deref())
        .cloned()
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
    db.object_types().get(&name).cloned()
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
    db.enums().get(&name).cloned()
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
        .filter(|def| def.directives().iter().any(|dir| dir.name() == directive))
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
        Value::List(list) => list
            .iter()
            .flat_map(|val| get_field_variable_value(val.clone()))
            .collect(),
        Value::Object(obj) => obj
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
    for (name, definition) in &*db.object_types() {
        for implements in definition.implements_interfaces() {
            add(implements.interface(), name);
        }
        for extension in definition.extensions() {
            for implements in extension.implements_interfaces() {
                add(implements.interface(), name);
            }
        }
    }
    for (name, definition) in &*db.interfaces() {
        for implements in definition.implements_interfaces() {
            add(implements.interface(), name);
        }
        for extension in definition.extensions() {
            for implements in extension.implements_interfaces() {
                add(implements.interface(), name);
            }
        }
    }
    for (name, definition) in &*db.unions() {
        for member in definition.union_members() {
            add(name, member.name());
        }
        for extension in definition.extensions() {
            for member in extension.union_members() {
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

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn find_definitions_with_directive() {
        let schema = r#"
            type ObjectOne @key(field: "id") {
              id: ID!
              inStock: Boolean!
            }

            type ObjectTwo @key(field: "name") {
              name: String!
              address: String!
            }

            type ObjectThree {
                price: Int
            }
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(schema, "schema.graphql");

        let key_definitions = compiler.db.find_types_with_directive(String::from("key"));
        let mut key_definition_names: Vec<&str> =
            key_definitions.iter().map(|def| def.name()).collect();
        key_definition_names.sort();
        assert_eq!(key_definition_names, ["ObjectOne", "ObjectTwo"])
    }

    #[test]
    fn is_subtype() {
        fn gen_schema_types(schema: &str) -> ApolloCompiler {
            let base_schema = with_supergraph_boilerplate(
                r#"
                type Query {
                me: String
                }
                type Foo {
                me: String
                }
                type Bar {
                me: String
                }
                type Baz {
                me: String
                }

                union UnionType2 = Foo | Bar
                "#,
            );
            let schema = format!("{base_schema}\n{schema}");
            let mut compiler = ApolloCompiler::new();
            compiler.add_document(&schema, "schema.graphql");
            compiler
        }

        fn gen_schema_interfaces(schema: &str) -> ApolloCompiler {
            let base_schema = with_supergraph_boilerplate(
                r#"
                type Query {
                me: String
                }
                interface Foo {
                me: String
                }
                interface Bar {
                me: String
                }
                interface Baz {
                me: String,
                }

                type ObjectType2 implements Foo & Bar { me: String }
                interface InterfaceType2 implements Foo & Bar { me: String }
                "#,
            );
            let schema = format!("{base_schema}\n{schema}");
            let mut compiler = ApolloCompiler::new();
            compiler.add_document(&schema, "schema.graphql");
            compiler
        }

        let ctx = gen_schema_types("union UnionType = Foo | Bar | Baz");
        assert!(ctx.db.is_subtype("UnionType".into(), "Foo".into()));
        assert!(ctx.db.is_subtype("UnionType".into(), "Bar".into()));
        assert!(ctx.db.is_subtype("UnionType".into(), "Baz".into()));

        let ctx =
            gen_schema_interfaces("type ObjectType implements Foo & Bar & Baz { me: String }");
        assert!(ctx.db.is_subtype("Foo".into(), "ObjectType".into()));
        assert!(ctx.db.is_subtype("Bar".into(), "ObjectType".into()));
        assert!(ctx.db.is_subtype("Baz".into(), "ObjectType".into()));

        let ctx = gen_schema_interfaces(
            "interface InterfaceType implements Foo & Bar & Baz { me: String }",
        );
        assert!(ctx.db.is_subtype("Foo".into(), "InterfaceType".into()));
        assert!(ctx.db.is_subtype("Bar".into(), "InterfaceType".into()));
        assert!(ctx.db.is_subtype("Baz".into(), "InterfaceType".into()));

        let ctx = gen_schema_types("extend union UnionType2 = Baz");
        assert!(ctx.db.is_subtype("UnionType2".into(), "Foo".into()));
        assert!(ctx.db.is_subtype("UnionType2".into(), "Bar".into()));
        assert!(ctx.db.is_subtype("UnionType2".into(), "Baz".into()));

        let ctx = gen_schema_interfaces("extend type ObjectType2 implements Baz { me2: String }");
        assert!(ctx.db.is_subtype("Foo".into(), "ObjectType2".into()));
        assert!(ctx.db.is_subtype("Bar".into(), "ObjectType2".into()));
        assert!(ctx.db.is_subtype("Baz".into(), "ObjectType2".into()));

        let ctx =
            gen_schema_interfaces("extend interface InterfaceType2 implements Baz { me2: String }");
        assert!(ctx.db.is_subtype("Foo".into(), "InterfaceType2".into()));
        assert!(ctx.db.is_subtype("Bar".into(), "InterfaceType2".into()));
        assert!(ctx.db.is_subtype("Baz".into(), "InterfaceType2".into()));
    }

    fn with_supergraph_boilerplate(content: &str) -> String {
        format!(
            "{}\n{}",
            r#"
            schema
                @core(feature: "https://specs.apollo.dev/core/v0.1")
                @core(feature: "https://specs.apollo.dev/join/v0.1") {
                query: Query
            }
            directive @core(feature: String!) repeatable on SCHEMA
            directive @join__graph(name: String!, url: String!) on ENUM_VALUE
            enum join__Graph {
                TEST @join__graph(name: "test", url: "http://localhost:4001/graphql")
            }

            "#,
            content
        )
    }
}
