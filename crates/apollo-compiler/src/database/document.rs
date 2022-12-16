use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{hir::*, AstDatabase, FileId, HirDatabase, InputDatabase};
use indexmap::IndexMap;

#[salsa::query_group(DocumentStorage)]
pub trait DocumentDatabase: InputDatabase + AstDatabase + HirDatabase {
    fn find_operation_by_name(
        &self,
        file_id: FileId,
        name: String,
    ) -> Option<Arc<OperationDefinition>>;

    fn find_fragment_by_name(
        &self,
        file_id: FileId,
        name: String,
    ) -> Option<Arc<FragmentDefinition>>;

    fn find_object_type_by_name(&self, name: String) -> Option<Arc<ObjectTypeDefinition>>;

    fn find_union_by_name(&self, name: String) -> Option<Arc<UnionTypeDefinition>>;

    fn find_enum_by_name(&self, name: String) -> Option<Arc<EnumTypeDefinition>>;

    fn find_interface_by_name(&self, name: String) -> Option<Arc<InterfaceTypeDefinition>>;

    fn find_directive_definition_by_name(&self, name: String) -> Option<Arc<DirectiveDefinition>>;

    fn find_definitions_with_directive(&self, directive: String) -> Arc<Vec<Definition>>;

    fn find_input_object_by_name(&self, name: String) -> Option<Arc<InputObjectTypeDefinition>>;

    fn find_definition_by_name(&self, name: String) -> Option<Definition>;

    fn find_type_system_definition_by_name(&self, name: String) -> Option<Definition>;

    fn types_definitions_by_name(&self) -> Arc<IndexMap<String, TypeDefinition>>;

    fn find_type_definition_by_name(&self, name: String) -> Option<TypeDefinition>;

    fn query_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    fn mutation_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    fn subscription_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    fn operation_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    fn operation_inline_fragment_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    fn operation_fragment_spread_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    fn selection_variables(&self, selection_set: SelectionSet) -> Arc<HashSet<Variable>>;

    fn operation_definition_variables(
        &self,
        variables: Arc<Vec<VariableDefinition>>,
    ) -> Arc<HashSet<Variable>>;

    fn subtype_map(&self) -> Arc<HashMap<String, HashSet<String>>>;

    fn is_subtype(&self, abstract_type: String, maybe_subtype: String) -> bool;
}

fn find_definition_by_name(db: &dyn DocumentDatabase, name: String) -> Option<Definition> {
    db.db_definitions()
        .iter()
        .find(|def| def.name() == Some(&*name))
        .cloned()
}

fn find_type_system_definition_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Definition> {
    db.type_system_definitions()
        .iter()
        .find(|def| def.name() == Some(&*name))
        .cloned()
}

fn types_definitions_by_name(db: &dyn DocumentDatabase) -> Arc<IndexMap<String, TypeDefinition>> {
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

fn find_type_definition_by_name(db: &dyn DocumentDatabase, name: String) -> Option<TypeDefinition> {
    db.types_definitions_by_name().get(&name).cloned()
}

fn find_operation_by_name(
    db: &dyn DocumentDatabase,
    file_id: FileId,
    name: String,
) -> Option<Arc<OperationDefinition>> {
    db.operations(file_id)
        .iter()
        .find(|def| def.name() == Some(&*name))
        .cloned()
}

fn find_fragment_by_name(
    db: &dyn DocumentDatabase,
    file_id: FileId,
    name: String,
) -> Option<Arc<FragmentDefinition>> {
    db.fragments(file_id).get(&name).cloned()
}

fn find_object_type_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types().get(&name).cloned()
}

fn find_union_by_name(db: &dyn DocumentDatabase, name: String) -> Option<Arc<UnionTypeDefinition>> {
    db.unions().get(&name).cloned()
}

fn find_enum_by_name(db: &dyn DocumentDatabase, name: String) -> Option<Arc<EnumTypeDefinition>> {
    db.enums().get(&name).cloned()
}

fn find_interface_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().get(&name).cloned()
}

fn find_directive_definition_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<DirectiveDefinition>> {
    db.directive_definitions().get(&name).cloned()
}

/// Find any definitions that use the specified directive.
fn find_definitions_with_directive(
    db: &dyn DocumentDatabase,
    directive: String,
) -> Arc<Vec<Definition>> {
    let mut definitions = Vec::new();
    for def in db.db_definitions().iter() {
        let any = def.directives().iter().any(|dir| dir.name() == directive);

        if any {
            definitions.push(def.clone())
        }
    }

    Arc::new(definitions)
}

fn find_input_object_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().get(&name).cloned()
}

fn query_operations(
    db: &dyn DocumentDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_query().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn subscription_operations(
    db: &dyn DocumentDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_subscription().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn mutation_operations(
    db: &dyn DocumentDatabase,
    file_id: FileId,
) -> Arc<Vec<Arc<OperationDefinition>>> {
    let operations = db
        .operations(file_id)
        .iter()
        .filter_map(|op| op.operation_ty.is_mutation().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn operation_fields(_db: &dyn DocumentDatabase, selection_set: SelectionSet) -> Arc<Vec<Field>> {
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

fn operation_inline_fragment_fields(
    _db: &dyn DocumentDatabase,
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

fn operation_fragment_spread_fields(
    db: &dyn DocumentDatabase,
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
fn selection_variables(
    db: &dyn DocumentDatabase,
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

fn get_field_variable_value(val: Value) -> Vec<Variable> {
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
fn operation_definition_variables(
    _db: &dyn DocumentDatabase,
    variables: Arc<Vec<VariableDefinition>>,
) -> Arc<HashSet<Variable>> {
    let vars = variables
        .iter()
        .map(|v| Variable {
            name: v.name().to_owned(),
            loc: *v.loc(),
        })
        .collect();
    Arc::new(vars)
}

fn subtype_map(db: &dyn DocumentDatabase) -> Arc<HashMap<String, HashSet<String>>> {
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

fn is_subtype(db: &dyn DocumentDatabase, abstract_type: String, maybe_subtype: String) -> bool {
    db.subtype_map()
        .get(&abstract_type)
        .map_or(false, |set| set.contains(&maybe_subtype))
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::DocumentDatabase;

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
        compiler.create_document(schema, "schema.graphql");

        let key_definitions = compiler
            .db
            .find_definitions_with_directive(String::from("key"));
        let mut key_definition_names: Vec<&str> = key_definitions
            .iter()
            .filter_map(|def| def.name())
            .collect();
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
            let schema = format!("{}\n{}", base_schema, schema);
            let mut compiler = ApolloCompiler::new();
            compiler.create_document(&schema, "schema.graphql");
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
            let schema = format!("{}\n{}", base_schema, schema);
            let mut compiler = ApolloCompiler::new();
            compiler.create_document(&schema, "schema.graphql");
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
