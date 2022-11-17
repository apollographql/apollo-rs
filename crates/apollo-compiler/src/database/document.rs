use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use uuid::Uuid;

use crate::{hir::*, AstDatabase, HirDatabase, InputDatabase};

#[salsa::query_group(DocumentStorage)]
pub trait DocumentDatabase: InputDatabase + AstDatabase + HirDatabase {
    fn find_operation(&self, id: Uuid) -> Option<Arc<OperationDefinition>>;

    fn find_operation_by_name(&self, name: String) -> Option<Arc<OperationDefinition>>;

    fn find_fragment(&self, id: Uuid) -> Option<Arc<FragmentDefinition>>;

    fn find_fragment_by_name(&self, name: String) -> Option<Arc<FragmentDefinition>>;

    fn find_object_type(&self, id: Uuid) -> Option<Arc<ObjectTypeDefinition>>;

    fn find_object_type_by_name(&self, name: String) -> Option<Arc<ObjectTypeDefinition>>;

    fn find_union_by_name(&self, name: String) -> Option<Arc<UnionTypeDefinition>>;

    fn find_interface(&self, id: Uuid) -> Option<Arc<InterfaceTypeDefinition>>;

    fn find_interface_by_name(&self, name: String) -> Option<Arc<InterfaceTypeDefinition>>;

    fn find_directive_definition(&self, id: Uuid) -> Option<Arc<DirectiveDefinition>>;

    fn find_directive_definition_by_name(&self, name: String) -> Option<Arc<DirectiveDefinition>>;

    fn find_definitions_with_directive(&self, directive: String) -> Arc<Vec<Definition>>;

    fn find_input_object(&self, id: Uuid) -> Option<Arc<InputObjectTypeDefinition>>;

    fn find_input_object_by_name(&self, name: String) -> Option<Arc<InputObjectTypeDefinition>>;

    fn find_definition_by_name(&self, name: String) -> Option<Arc<Definition>>;

    fn find_type_system_definition(&self, id: Uuid) -> Option<Arc<Definition>>;

    fn find_type_system_definition_by_name(&self, name: String) -> Option<Arc<Definition>>;

    fn query_operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn mutation_operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn subscription_operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn operation_fields(&self, id: Uuid) -> Arc<Vec<Field>>;

    fn operation_inline_fragment_fields(&self, id: Uuid) -> Arc<Vec<Field>>;

    fn operation_fragment_spread_fields(&self, id: Uuid) -> Arc<Vec<Field>>;

    fn selection_variables(&self, id: Uuid) -> Arc<HashSet<Variable>>;

    fn operation_definition_variables(&self, id: Uuid) -> Arc<HashSet<Variable>>;

    fn subtype_map(&self) -> Arc<HashMap<String, HashSet<String>>>;

    fn is_subtype(&self, abstract_type: String, maybe_subtype: String) -> bool;
}

fn find_definition_by_name(db: &dyn DocumentDatabase, name: String) -> Option<Arc<Definition>> {
    db.db_definitions().iter().find_map(|def| {
        if let Some(n) = def.name() {
            if name == n {
                return Some(Arc::new(def.clone()));
            }
        }
        None
    })
}

fn find_type_system_definition(db: &dyn DocumentDatabase, id: Uuid) -> Option<Arc<Definition>> {
    db.type_system_definitions().iter().find_map(|op| {
        if let Some(op_id) = op.id() {
            if op_id == &id {
                return Some(Arc::new(op.clone()));
            }
        }
        None
    })
}

fn find_type_system_definition_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<Definition>> {
    db.type_system_definitions().iter().find_map(|def| {
        if let Some(n) = def.name() {
            if name == n {
                return Some(Arc::new(def.clone()));
            }
        }
        None
    })
}

fn find_operation(db: &dyn DocumentDatabase, id: Uuid) -> Option<Arc<OperationDefinition>> {
    db.operations().iter().find_map(|op| {
        if &id == op.id() {
            return Some(Arc::new(op.clone()));
        }
        None
    })
}

fn find_operation_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<OperationDefinition>> {
    db.operations().iter().find_map(|op| {
        if let Some(n) = op.name() {
            if n == name {
                return Some(Arc::new(op.clone()));
            }
        }
        None
    })
}

fn find_fragment(db: &dyn DocumentDatabase, id: Uuid) -> Option<Arc<FragmentDefinition>> {
    db.fragments().iter().find_map(|fragment| {
        if &id == fragment.id() {
            return Some(Arc::new(fragment.clone()));
        }
        None
    })
}

fn find_fragment_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<FragmentDefinition>> {
    db.fragments().iter().find_map(|fragment| {
        if name == fragment.name() {
            return Some(Arc::new(fragment.clone()));
        }
        None
    })
}

fn find_object_type(db: &dyn DocumentDatabase, id: Uuid) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types().iter().find_map(|object_type| {
        if &id == object_type.id() {
            return Some(Arc::new(object_type.clone()));
        }
        None
    })
}

fn find_object_type_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types().iter().find_map(|object_type| {
        if name == object_type.name() {
            return Some(Arc::new(object_type.clone()));
        }
        None
    })
}

fn find_union_by_name(db: &dyn DocumentDatabase, name: String) -> Option<Arc<UnionTypeDefinition>> {
    db.unions().iter().find_map(|union| {
        if name == union.name() {
            return Some(Arc::new(union.clone()));
        }
        None
    })
}

fn find_interface(db: &dyn DocumentDatabase, id: Uuid) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().iter().find_map(|interface| {
        if &id == interface.id() {
            return Some(Arc::new(interface.clone()));
        }
        None
    })
}

fn find_interface_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().iter().find_map(|interface| {
        if name == interface.name() {
            return Some(Arc::new(interface.clone()));
        }
        None
    })
}

fn find_directive_definition(
    db: &dyn DocumentDatabase,
    id: Uuid,
) -> Option<Arc<DirectiveDefinition>> {
    db.directive_definitions().iter().find_map(|directive_def| {
        if &id == directive_def.id() {
            return Some(Arc::new(directive_def.clone()));
        }
        None
    })
}

fn find_directive_definition_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<DirectiveDefinition>> {
    db.directive_definitions().iter().find_map(|directive_def| {
        if name == directive_def.name() {
            return Some(Arc::new(directive_def.clone()));
        }
        None
    })
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

fn find_input_object(
    db: &dyn DocumentDatabase,
    id: Uuid,
) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().iter().find_map(|input_obj| {
        if &id == input_obj.id() {
            return Some(Arc::new(input_obj.clone()));
        }
        None
    })
}

fn find_input_object_by_name(
    db: &dyn DocumentDatabase,
    name: String,
) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().iter().find_map(|input_obj| {
        if name == input_obj.name() {
            return Some(Arc::new(input_obj.clone()));
        }
        None
    })
}

fn query_operations(db: &dyn DocumentDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_query().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn subscription_operations(db: &dyn DocumentDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_subscription().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn mutation_operations(db: &dyn DocumentDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_mutation().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn operation_fields(db: &dyn DocumentDatabase, id: Uuid) -> Arc<Vec<Field>> {
    let fields = match db.find_operation(id) {
        Some(op) => op
            .selection_set()
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::Field(field) => Some(field.as_ref().clone()),
                _ => None,
            })
            .collect(),
        None => Vec::new(),
    };
    Arc::new(fields)
}

fn operation_inline_fragment_fields(db: &dyn DocumentDatabase, id: Uuid) -> Arc<Vec<Field>> {
    let fields: Vec<Field> = match db.find_operation(id) {
        Some(op) => op
            .selection_set()
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
            .collect(),
        None => Vec::new(),
    };
    Arc::new(fields)
}

fn operation_fragment_spread_fields(db: &dyn DocumentDatabase, id: Uuid) -> Arc<Vec<Field>> {
    let fields: Vec<Field> = match db.find_operation(id) {
        Some(op) => op
            .selection_set()
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
            .collect(),
        None => Vec::new(),
    };
    Arc::new(fields)
}

// Should be part of operation's db
// NOTE: potentially want to return a hashmap of variables and their types?
fn selection_variables(db: &dyn DocumentDatabase, id: Uuid) -> Arc<HashSet<Variable>> {
    let vars = db
        .operation_fields(id)
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
fn operation_definition_variables(db: &dyn DocumentDatabase, id: Uuid) -> Arc<HashSet<Variable>> {
    let vars: HashSet<Variable> = match db.find_operation(id) {
        Some(op) => op
            .variables()
            .iter()
            .map(|v| Variable {
                name: v.name().to_owned(),
                ast_ptr: v.ast_ptr().clone(),
            })
            .collect(),
        None => HashSet::new(),
    };
    Arc::new(vars)
}

fn subtype_map(db: &dyn DocumentDatabase) -> Arc<HashMap<String, HashSet<String>>> {
    let mut map = HashMap::<String, HashSet<String>>::new();
    let mut add = |key, value| map.entry(key).or_default().insert(value);
    for definition in &*db.type_system_definitions() {
        match definition {
            Definition::ObjectTypeDefinition(def) => {
                for implements in def.implements_interfaces() {
                    add(implements.interface().to_owned(), def.name().to_owned());
                }
            }
            Definition::ObjectTypeExtension(def) => {
                for implements in def.implements_interfaces() {
                    add(implements.interface().to_owned(), def.name().to_owned());
                }
            }
            Definition::InterfaceTypeDefinition(def) => {
                for implements in def.implements_interfaces() {
                    add(implements.interface().to_owned(), def.name().to_owned());
                }
            }
            Definition::InterfaceTypeExtension(def) => {
                for implements in def.implements_interfaces() {
                    add(implements.interface().to_owned(), def.name().to_owned());
                }
            }
            Definition::UnionTypeDefinition(def) => {
                for member in def.union_members() {
                    add(def.name().to_owned(), member.name().to_owned());
                }
            }
            Definition::UnionTypeExtension(def) => {
                for member in def.union_members() {
                    add(def.name().to_owned(), member.name().to_owned());
                }
            }
            Definition::InputObjectTypeDefinition(_)
            | Definition::EnumTypeDefinition(_)
            | Definition::ScalarTypeDefinition(_)
            | Definition::DirectiveDefinition(_)
            | Definition::OperationDefinition(_)
            | Definition::FragmentDefinition(_)
            | Definition::SchemaDefinition(_)
            | Definition::SchemaExtension(_)
            | Definition::ScalarTypeExtension(_)
            | Definition::EnumTypeExtension(_)
            | Definition::InputObjectTypeExtension(_) => {}
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

        let ctx = ApolloCompiler::new(schema);
        let key_definitions = ctx.db.find_definitions_with_directive(String::from("key"));
        let key_definition_names: Vec<&str> = key_definitions
            .iter()
            .filter_map(|def| def.name())
            .collect();
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
            ApolloCompiler::new(&schema)
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
            ApolloCompiler::new(&schema)
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
