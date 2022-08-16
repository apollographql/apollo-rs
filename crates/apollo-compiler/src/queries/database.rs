// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use std::collections::HashSet;
use std::sync::Arc;

use apollo_parser::ast::{AstChildren, AstNode, SyntaxNodePtr};
use apollo_parser::{ast, Parser, SyntaxTree};
use uuid::Uuid;

use crate::diagnostics::{ApolloDiagnostic, SyntaxError};
// use crate::diagnostics::{ApolloDiagnostic, ErrorDiagnostic};
use crate::values::*;

#[salsa::database(ASTDatabase)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}

// NOTE @lrlna: this is the root database.
// In the long run we will create child databases based on definitions: i.e.
// OperationDefinition DB, ObjectTypeDefinition etc. This is mostly going to be
// useful for readability of this code.
#[salsa::query_group(ASTDatabase)]
pub trait SourceDatabase {
    #[salsa::input]
    fn input_string(&self, key: ()) -> Arc<String>;

    fn parse(&self) -> Arc<SyntaxTree>;

    fn document(&self) -> Arc<ast::Document>;

    fn syntax_errors(&self) -> Vec<ApolloDiagnostic>;

    fn definitions(&self) -> Arc<Vec<ast::Definition>>;

    fn db_definitions(&self) -> Arc<Vec<Definition>>;

    fn type_system_definitions(&self) -> Arc<Vec<Definition>>;

    fn operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn fragments(&self) -> Arc<Vec<FragmentDefinition>>;

    fn schema(&self) -> Arc<SchemaDefinition>;

    fn object_types(&self) -> Arc<Vec<ObjectTypeDefinition>>;

    fn scalars(&self) -> Arc<Vec<ScalarTypeDefinition>>;

    fn enums(&self) -> Arc<Vec<EnumTypeDefinition>>;

    fn unions(&self) -> Arc<Vec<UnionTypeDefinition>>;

    fn interfaces(&self) -> Arc<Vec<InterfaceTypeDefinition>>;

    fn directive_definitions(&self) -> Arc<Vec<DirectiveDefinition>>;

    fn input_objects(&self) -> Arc<Vec<InputObjectTypeDefinition>>;

    fn query_operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn mutation_operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn subscription_operations(&self) -> Arc<Vec<OperationDefinition>>;

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

    fn find_input_object(&self, id: Uuid) -> Option<Arc<InputObjectTypeDefinition>>;

    fn find_input_object_by_name(&self, name: String) -> Option<Arc<InputObjectTypeDefinition>>;

    fn find_definition_by_name(&self, name: String) -> Option<Arc<Definition>>;

    fn find_type_system_definition(&self, id: Uuid) -> Option<Arc<Definition>>;

    fn find_type_system_definition_by_name(&self, name: String) -> Option<Arc<Definition>>;

    fn operation_definition_variables(&self, id: Uuid) -> Arc<HashSet<Variable>>;

    fn selection_variables(&self, id: Uuid) -> Arc<HashSet<Variable>>;

    fn operation_fields(&self, id: Uuid) -> Arc<Vec<Field>>;

    fn operation_inline_fragment_fields(&self, id: Uuid) -> Arc<Vec<Field>>;

    fn operation_fragment_spread_fields(&self, id: Uuid) -> Arc<Vec<Field>>;
}

// this is top level entry to the source db
fn parse(db: &dyn SourceDatabase) -> Arc<SyntaxTree> {
    let input = db.input_string(());
    let parser = Parser::new(&input);
    Arc::new(parser.parse())
}

// TODO @lrlna: a very expensive clone - should more tightly couple the parser and the
// source database for a cleaner solution
fn document(db: &dyn SourceDatabase) -> Arc<ast::Document> {
    Arc::new(db.parse().as_ref().clone().document())
}

fn syntax_errors(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.parse()
        .errors()
        .into_iter()
        .map(|err| {
            ApolloDiagnostic::SyntaxError(SyntaxError {
                src: db.input_string(()).to_string(),
                span: (err.index(), err.data().len()).into(), // (offset, length of error token)
                message: err.message().into(),
            })
        })
        .collect()
}

fn definitions(db: &dyn SourceDatabase) -> Arc<Vec<ast::Definition>> {
    Arc::new(db.document().definitions().into_iter().collect())
}

fn db_definitions(db: &dyn SourceDatabase) -> Arc<Vec<Definition>> {
    let mut definitions = Vec::new();

    let operations: Vec<Definition> = db
        .operations()
        .iter()
        .map(|def| Definition::OperationDefinition(def.clone()))
        .collect();
    let fragments: Vec<Definition> = db
        .fragments()
        .iter()
        .map(|def| Definition::FragmentDefinition(def.clone()))
        .collect();
    let directives: Vec<Definition> = db
        .directive_definitions()
        .iter()
        .map(|def| Definition::DirectiveDefinition(def.clone()))
        .collect();
    let scalars: Vec<Definition> = db
        .scalars()
        .iter()
        .map(|def| Definition::ScalarTypeDefinition(def.clone()))
        .collect();
    let objects: Vec<Definition> = db
        .object_types()
        .iter()
        .map(|def| Definition::ObjectTypeDefinition(def.clone()))
        .collect();
    let interfaces: Vec<Definition> = db
        .interfaces()
        .iter()
        .map(|def| Definition::InterfaceTypeDefinition(def.clone()))
        .collect();
    let unions: Vec<Definition> = db
        .unions()
        .iter()
        .map(|def| Definition::UnionTypeDefinition(def.clone()))
        .collect();
    let enums: Vec<Definition> = db
        .enums()
        .iter()
        .map(|def| Definition::EnumTypeDefinition(def.clone()))
        .collect();
    let input_objects: Vec<Definition> = db
        .input_objects()
        .iter()
        .map(|def| Definition::InputObjectTypeDefinition(def.clone()))
        .collect();
    let schema = Definition::SchemaDefinition(db.schema().as_ref().clone());

    definitions.extend(operations);
    definitions.extend(fragments);
    definitions.extend(directives);
    definitions.extend(scalars);
    definitions.extend(objects);
    definitions.extend(interfaces);
    definitions.extend(unions);
    definitions.extend(enums);
    definitions.extend(input_objects);
    definitions.push(schema);

    Arc::new(definitions)
}

fn type_system_definitions(db: &dyn SourceDatabase) -> Arc<Vec<Definition>> {
    let mut definitions = Vec::new();

    let directives: Vec<Definition> = db
        .directive_definitions()
        .iter()
        .map(|def| Definition::DirectiveDefinition(def.clone()))
        .collect();
    let scalars: Vec<Definition> = db
        .scalars()
        .iter()
        .map(|def| Definition::ScalarTypeDefinition(def.clone()))
        .collect();
    let objects: Vec<Definition> = db
        .object_types()
        .iter()
        .map(|def| Definition::ObjectTypeDefinition(def.clone()))
        .collect();
    let interfaces: Vec<Definition> = db
        .interfaces()
        .iter()
        .map(|def| Definition::InterfaceTypeDefinition(def.clone()))
        .collect();
    let unions: Vec<Definition> = db
        .unions()
        .iter()
        .map(|def| Definition::UnionTypeDefinition(def.clone()))
        .collect();
    let enums: Vec<Definition> = db
        .enums()
        .iter()
        .map(|def| Definition::EnumTypeDefinition(def.clone()))
        .collect();
    let input_objects: Vec<Definition> = db
        .input_objects()
        .iter()
        .map(|def| Definition::InputObjectTypeDefinition(def.clone()))
        .collect();
    let schema = Definition::SchemaDefinition(db.schema().as_ref().clone());

    definitions.extend(directives);
    definitions.extend(scalars);
    definitions.extend(objects);
    definitions.extend(interfaces);
    definitions.extend(unions);
    definitions.extend(enums);
    definitions.extend(input_objects);
    definitions.push(schema);

    Arc::new(definitions)
}

fn find_definition_by_name(db: &dyn SourceDatabase, name: String) -> Option<Arc<Definition>> {
    db.db_definitions().iter().find_map(|def| {
        if let Some(n) = def.name() {
            if name == n {
                return Some(Arc::new(def.clone()));
            }
        }
        None
    })
}

fn find_type_system_definition(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<Definition>> {
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
    db: &dyn SourceDatabase,
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

// NOTE: we might want to the values::OperationDefinition creation even further.
// At the moment all fields in this struct are created here, instead individual
// queries for selection_set, variables, directives etc can be created.
fn operations(db: &dyn SourceDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::OperationDefinition(op_def) => {
                Some(operation_definition(db, op_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(operations)
}

fn query_operations(db: &dyn SourceDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_query().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn subscription_operations(db: &dyn SourceDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_subscription().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn mutation_operations(db: &dyn SourceDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.operation_ty.is_mutation().then(|| op.clone()))
        .collect();
    Arc::new(operations)
}

fn find_operation(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<OperationDefinition>> {
    db.operations().iter().find_map(|op| {
        if &id == op.id() {
            return Some(Arc::new(op.clone()));
        }
        None
    })
}

fn find_operation_by_name(
    db: &dyn SourceDatabase,
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

// NOTE: potentially want to return a hashset of variables and their types?
fn operation_definition_variables(db: &dyn SourceDatabase, id: Uuid) -> Arc<HashSet<Variable>> {
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

fn operation_fields(db: &dyn SourceDatabase, id: Uuid) -> Arc<Vec<Field>> {
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

fn operation_inline_fragment_fields(db: &dyn SourceDatabase, id: Uuid) -> Arc<Vec<Field>> {
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

fn operation_fragment_spread_fields(db: &dyn SourceDatabase, id: Uuid) -> Arc<Vec<Field>> {
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

// NOTE: potentially want to return a hashmap of variables and their types?
fn selection_variables(db: &dyn SourceDatabase, id: Uuid) -> Arc<HashSet<Variable>> {
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

fn fragments(db: &dyn SourceDatabase) -> Arc<Vec<FragmentDefinition>> {
    let fragments: Vec<FragmentDefinition> = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::FragmentDefinition(fragment_def) => {
                Some(fragment_definition(db, fragment_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(fragments)
}

fn find_fragment(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<FragmentDefinition>> {
    db.fragments().iter().find_map(|fragment| {
        if &id == fragment.id() {
            return Some(Arc::new(fragment.clone()));
        }
        None
    })
}

fn find_fragment_by_name(db: &dyn SourceDatabase, name: String) -> Option<Arc<FragmentDefinition>> {
    db.fragments().iter().find_map(|fragment| {
        if name == fragment.name() {
            return Some(Arc::new(fragment.clone()));
        }
        None
    })
}

fn schema(db: &dyn SourceDatabase) -> Arc<SchemaDefinition> {
    let schema = db
        .definitions()
        .iter()
        .find_map(|definition| match definition {
            ast::Definition::SchemaDefinition(schema) => Some(schema.clone()),
            _ => None,
        });
    let mut schema_def = schema.map_or(SchemaDefinition::default(), |s| schema_definition(db, s));

    // NOTE(@lrlna):
    //
    // "Query", "Subscription", "Mutation" object type definitions do not need
    // to be explicitly defined in a schema definition, but are implicitly
    // added.
    //
    // There will be a time when we need to distinguish between implicit and
    // explicit definitions for validation purposes.
    let type_defs = add_object_type_id_to_schema(db);
    type_defs
        .iter()
        .for_each(|ty| schema_def.set_root_operation_type_definition(ty.clone()));

    Arc::new(schema_def)
}

fn object_types(db: &dyn SourceDatabase) -> Arc<Vec<ObjectTypeDefinition>> {
    let objects = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::ObjectTypeDefinition(obj_def) => {
                Some(object_type_definition(obj_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(objects)
}

fn find_object_type(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types().iter().find_map(|object_type| {
        if &id == object_type.id() {
            return Some(Arc::new(object_type.clone()));
        }
        None
    })
}

fn find_object_type_by_name(
    db: &dyn SourceDatabase,
    name: String,
) -> Option<Arc<ObjectTypeDefinition>> {
    db.object_types().iter().find_map(|object_type| {
        if name == object_type.name() {
            return Some(Arc::new(object_type.clone()));
        }
        None
    })
}

fn scalars(db: &dyn SourceDatabase) -> Arc<Vec<ScalarTypeDefinition>> {
    let scalars = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::ScalarTypeDefinition(scalar_def) => {
                Some(scalar_definition(scalar_def.clone()))
            }
            _ => None,
        })
        .collect();
    let scalars = built_in_scalars(scalars);

    Arc::new(scalars)
}

fn enums(db: &dyn SourceDatabase) -> Arc<Vec<EnumTypeDefinition>> {
    let enums = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::EnumTypeDefinition(enum_def) => {
                Some(enum_definition(enum_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(enums)
}

fn unions(db: &dyn SourceDatabase) -> Arc<Vec<UnionTypeDefinition>> {
    let unions = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::UnionTypeDefinition(union_def) => {
                Some(union_definition(db, union_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(unions)
}

fn find_union_by_name(db: &dyn SourceDatabase, name: String) -> Option<Arc<UnionTypeDefinition>> {
    db.unions().iter().find_map(|union| {
        if name == union.name() {
            return Some(Arc::new(union.clone()));
        }
        None
    })
}

fn interfaces(db: &dyn SourceDatabase) -> Arc<Vec<InterfaceTypeDefinition>> {
    let interfaces = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::InterfaceTypeDefinition(interface_def) => {
                Some(interface_definition(interface_def.clone()))
            }
            _ => None,
        })
        .collect();
    Arc::new(interfaces)
}

fn find_interface(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().iter().find_map(|interface| {
        if &id == interface.id() {
            return Some(Arc::new(interface.clone()));
        }
        None
    })
}

fn find_interface_by_name(
    db: &dyn SourceDatabase,
    name: String,
) -> Option<Arc<InterfaceTypeDefinition>> {
    db.interfaces().iter().find_map(|interface| {
        if name == interface.name() {
            return Some(Arc::new(interface.clone()));
        }
        None
    })
}

fn directive_definitions(db: &dyn SourceDatabase) -> Arc<Vec<DirectiveDefinition>> {
    let directives = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::DirectiveDefinition(directive_def) => {
                Some(directive_definition(directive_def.clone()))
            }
            _ => None,
        })
        .collect();

    let directives = built_in_directives(directives);

    Arc::new(directives)
}

fn find_directive_definition(
    db: &dyn SourceDatabase,
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
    db: &dyn SourceDatabase,
    name: String,
) -> Option<Arc<DirectiveDefinition>> {
    db.directive_definitions().iter().find_map(|directive_def| {
        if name == directive_def.name() {
            return Some(Arc::new(directive_def.clone()));
        }
        None
    })
}

fn input_objects(db: &dyn SourceDatabase) -> Arc<Vec<InputObjectTypeDefinition>> {
    let directives = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::InputObjectTypeDefinition(input_obj) => {
                Some(input_object_definition(input_obj.clone()))
            }
            _ => None,
        })
        .collect();

    Arc::new(directives)
}

fn find_input_object(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().iter().find_map(|input_obj| {
        if &id == input_obj.id() {
            return Some(Arc::new(input_obj.clone()));
        }
        None
    })
}

fn find_input_object_by_name(
    db: &dyn SourceDatabase,
    name: String,
) -> Option<Arc<InputObjectTypeDefinition>> {
    db.input_objects().iter().find_map(|input_obj| {
        if name == input_obj.name() {
            return Some(Arc::new(input_obj.clone()));
        }
        None
    })
}

fn operation_definition(
    db: &dyn SourceDatabase,
    op_def: ast::OperationDefinition,
) -> OperationDefinition {
    // check if there are already operations
    // if there are operations, they must have names
    // if there are no names, an error must be raised that all operations must have a name
    let name = op_def.name().map(|name| name.text().to_string());
    let ty = operation_type(op_def.operation_type());
    let object_id = object_type_uuid(db, ty);
    let variables = variable_definitions(op_def.variable_definitions());
    let selection_set = selection_set(db, op_def.selection_set(), object_id);
    let directives = directives(op_def.directives());
    let ast_ptr = SyntaxNodePtr::new(op_def.syntax());

    OperationDefinition {
        id: Uuid::new_v4(),
        operation_ty: ty,
        name,
        variables,
        selection_set,
        directives,
        object_id,
        ast_ptr,
    }
}

fn fragment_definition(
    db: &dyn SourceDatabase,
    fragment_def: ast::FragmentDefinition,
) -> FragmentDefinition {
    let name = name(
        fragment_def
            .fragment_name()
            .expect("Fragment Definition must have a name")
            .name(),
    );
    let type_condition = fragment_def
        .type_condition()
        .expect("Fragment Definition must have a type condition")
        .named_type()
        .expect("Type Condition must have a name")
        .name()
        .expect("Name must have text")
        .text()
        .to_string();
    let reference_ty_id = db
        .find_type_system_definition_by_name(type_condition.clone())
        .and_then(|def| def.id().cloned());
    let selection_set = selection_set(db, fragment_def.selection_set(), reference_ty_id);
    let directives = directives(fragment_def.directives());
    let ast_ptr = SyntaxNodePtr::new(fragment_def.syntax());

    FragmentDefinition {
        id: Uuid::new_v4(),
        name,
        type_condition,
        reference_ty_id,
        selection_set,
        directives,
        ast_ptr,
    }
}

fn schema_definition(
    db: &dyn SourceDatabase,
    schema_def: ast::SchemaDefinition,
) -> SchemaDefinition {
    let description = description(schema_def.description());
    let directives = directives(schema_def.directives());
    let root_operation_type_definition =
        root_operation_type_definition(db, schema_def.root_operation_type_definitions());
    let ast_ptr = SyntaxNodePtr::new(schema_def.syntax());

    SchemaDefinition {
        description,
        directives,
        root_operation_type_definition,
        ast_ptr: Some(ast_ptr),
    }
}

fn object_type_definition(obj_def: ast::ObjectTypeDefinition) -> ObjectTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(obj_def.description());
    let name = name(obj_def.name());
    let implements_interfaces = implements_interfaces(obj_def.implements_interfaces());
    let directives = directives(obj_def.directives());
    let fields_definition = fields_definition(obj_def.fields_definition());
    let ast_ptr = SyntaxNodePtr::new(obj_def.syntax());

    ObjectTypeDefinition {
        id,
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        ast_ptr,
    }
}

fn scalar_definition(scalar_def: ast::ScalarTypeDefinition) -> ScalarTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(scalar_def.description());
    let name = name(scalar_def.name());
    let directives = directives(scalar_def.directives());
    let ast_ptr = SyntaxNodePtr::new(scalar_def.syntax());

    ScalarTypeDefinition {
        id,
        description,
        name,
        directives,
        ast_ptr: Some(ast_ptr),
        built_in: false,
    }
}

fn enum_definition(enum_def: ast::EnumTypeDefinition) -> EnumTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(enum_def.description());
    let name = name(enum_def.name());
    let directives = directives(enum_def.directives());
    let enum_values_definition = enum_values_definition(enum_def.enum_values_definition());
    let ast_ptr = SyntaxNodePtr::new(enum_def.syntax());

    EnumTypeDefinition {
        id,
        description,
        name,
        directives,
        enum_values_definition,
        ast_ptr,
    }
}

fn enum_values_definition(
    enum_values_def: Option<ast::EnumValuesDefinition>,
) -> Arc<Vec<EnumValueDefinition>> {
    match enum_values_def {
        Some(enum_values) => {
            let enum_values = enum_values
                .enum_value_definitions()
                .into_iter()
                .map(enum_value_definition)
                .collect();
            Arc::new(enum_values)
        }
        None => Arc::new(Vec::new()),
    }
}

fn enum_value_definition(enum_value_def: ast::EnumValueDefinition) -> EnumValueDefinition {
    let description = description(enum_value_def.description());
    let enum_value = enum_value(enum_value_def.enum_value());
    let directives = directives(enum_value_def.directives());
    let ast_ptr = SyntaxNodePtr::new(enum_value_def.syntax());

    EnumValueDefinition {
        description,
        enum_value,
        directives,
        ast_ptr,
    }
}

fn union_definition(
    db: &dyn SourceDatabase,
    union_def: ast::UnionTypeDefinition,
) -> UnionTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(union_def.description());
    let name = name(union_def.name());
    let directives = directives(union_def.directives());
    let union_members = union_members(db, union_def.union_member_types());
    let ast_ptr = SyntaxNodePtr::new(union_def.syntax());

    UnionTypeDefinition {
        id,
        description,
        name,
        directives,
        union_members,
        ast_ptr,
    }
}

fn union_members(
    db: &dyn SourceDatabase,
    union_members: Option<ast::UnionMemberTypes>,
) -> Arc<Vec<UnionMember>> {
    match union_members {
        Some(members) => {
            let mems = members
                .named_types()
                .into_iter()
                .map(|u| union_member(db, u))
                .collect();
            Arc::new(mems)
        }
        None => Arc::new(Vec::new()),
    }
}

fn union_member(db: &dyn SourceDatabase, member: ast::NamedType) -> UnionMember {
    let name = name(member.name());
    let object_id = db
        .find_object_type_by_name(name.clone())
        .map(|object_type| *object_type.id());
    let ast_ptr = SyntaxNodePtr::new(member.syntax());

    UnionMember {
        name,
        object_id,
        ast_ptr,
    }
}

fn interface_definition(interface_def: ast::InterfaceTypeDefinition) -> InterfaceTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(interface_def.description());
    let name = name(interface_def.name());
    let implements_interfaces = implements_interfaces(interface_def.implements_interfaces());
    let directives = directives(interface_def.directives());
    let fields_definition = fields_definition(interface_def.fields_definition());
    let ast_ptr = SyntaxNodePtr::new(interface_def.syntax());

    InterfaceTypeDefinition {
        id,
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        ast_ptr,
    }
}

fn directive_definition(directive_def: ast::DirectiveDefinition) -> DirectiveDefinition {
    let name = name(directive_def.name());
    let description = description(directive_def.description());
    let arguments = arguments_definition(directive_def.arguments_definition());
    let repeatable = directive_def.repeatable_token().is_some();
    let directive_locations = directive_locations(directive_def.directive_locations());
    let ast_ptr = SyntaxNodePtr::new(directive_def.syntax());

    DirectiveDefinition {
        id: Uuid::new_v4(),
        description,
        name,
        arguments,
        repeatable,
        directive_locations,
        ast_ptr: Some(ast_ptr),
    }
}

fn input_object_definition(input_obj: ast::InputObjectTypeDefinition) -> InputObjectTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(input_obj.description());
    let name = name(input_obj.name());
    let directives = directives(input_obj.directives());
    let input_fields_definition = input_fields_definition(input_obj.input_fields_definition());
    let ast_ptr = SyntaxNodePtr::new(input_obj.syntax());

    InputObjectTypeDefinition {
        id,
        description,
        name,
        directives,
        input_fields_definition,
        ast_ptr,
    }
}

fn add_object_type_id_to_schema(db: &dyn SourceDatabase) -> Arc<Vec<RootOperationTypeDefinition>> {
    // Schema Definition does not have to be present in the SDL if ObjectType name is
    // - Query
    // - Subscription
    // - Mutation
    //
    // Compiler's internal schema, however, should have a reference to these
    // object types if they are present
    let type_defs: Vec<RootOperationTypeDefinition> = db
        .object_types()
        .iter()
        .filter_map(|obj_type| {
            let obj_name = obj_type.name();
            if matches!(obj_name, "Query" | "Subscription" | "Mutation") {
                let operation_type = obj_name.into();
                Some(RootOperationTypeDefinition {
                    object_type_id: Some(*obj_type.id()),
                    operation_type,
                    named_type: Type::Named {
                        name: obj_name.to_string(),
                        ast_ptr: None,
                    },
                    ast_ptr: None,
                })
            } else {
                None
            }
        })
        .collect();

    Arc::new(type_defs)
}

fn implements_interfaces(
    implements_interfaces: Option<ast::ImplementsInterfaces>,
) -> Arc<Vec<ImplementsInterface>> {
    let interfaces: Vec<ImplementsInterface> = implements_interfaces
        .iter()
        .flat_map(|interfaces| {
            let types: Vec<ImplementsInterface> = interfaces
                .named_types()
                .map(|n| ImplementsInterface {
                    interface: n.name().expect("Name must have text").text().to_string(),
                    ast_ptr: SyntaxNodePtr::new(n.syntax()),
                })
                .collect();
            types
        })
        .collect();

    Arc::new(interfaces)
}

fn fields_definition(
    fields_definition: Option<ast::FieldsDefinition>,
) -> Arc<Vec<FieldDefinition>> {
    match fields_definition {
        Some(fields_def) => {
            let fields: Vec<FieldDefinition> = fields_def
                .field_definitions()
                .map(field_definition)
                .collect();
            Arc::new(fields)
        }
        None => Arc::new(Vec::new()),
    }
}

fn field_definition(field: ast::FieldDefinition) -> FieldDefinition {
    let description = description(field.description());
    let name = name(field.name());
    let arguments = arguments_definition(field.arguments_definition());
    let ty = ty(field.ty().expect("Field must have a type"));
    let directives = directives(field.directives());
    let ast_ptr = SyntaxNodePtr::new(field.syntax());

    FieldDefinition {
        description,
        name,
        arguments,
        ty,
        directives,
        ast_ptr,
    }
}

fn arguments_definition(
    arguments_definition: Option<ast::ArgumentsDefinition>,
) -> ArgumentsDefinition {
    match arguments_definition {
        Some(arguments) => {
            let input_values = input_value_definitions(arguments.input_value_definitions());
            let ast_ptr = SyntaxNodePtr::new(arguments.syntax());

            ArgumentsDefinition {
                input_values,
                ast_ptr: Some(ast_ptr),
            }
        }
        None => ArgumentsDefinition {
            input_values: Arc::new(Vec::new()),
            ast_ptr: None,
        },
    }
}

fn input_fields_definition(
    input_fields: Option<ast::InputFieldsDefinition>,
) -> Arc<Vec<InputValueDefinition>> {
    match input_fields {
        Some(fields) => input_value_definitions(fields.input_value_definitions()),
        None => Arc::new(Vec::new()),
    }
}

fn input_value_definitions(
    input_values: AstChildren<ast::InputValueDefinition>,
) -> Arc<Vec<InputValueDefinition>> {
    let input_values: Vec<InputValueDefinition> = input_values
        .map(|input| {
            let description = description(input.description());
            let name = name(input.name());
            let ty = ty(input.ty().expect("Input Definition must have a type"));
            let default_value = default_value(input.default_value());
            let directives = directives(input.directives());
            let ast_ptr = SyntaxNodePtr::new(input.syntax());

            InputValueDefinition {
                description,
                name,
                ty,
                default_value,
                directives,
                ast_ptr: Some(ast_ptr),
            }
        })
        .collect();
    Arc::new(input_values)
}

fn default_value(default_value: Option<ast::DefaultValue>) -> Option<DefaultValue> {
    default_value.map(|val| value(val.value().expect("Default Value must have a value token")))
}

fn root_operation_type_definition(
    db: &dyn SourceDatabase,
    root_type_def: AstChildren<ast::RootOperationTypeDefinition>,
) -> Arc<Vec<RootOperationTypeDefinition>> {
    let type_defs: Vec<RootOperationTypeDefinition> = root_type_def
        .into_iter()
        .map(|ty| {
            let operation_type = operation_type(ty.operation_type());
            let named_type = named_type(
                ty.named_type()
                    .expect("Root Operation Type Definition must have Named Type.")
                    .name(),
            );
            let object_type_id = db
                .find_object_type_by_name(named_type.name())
                .map(|object_type| *object_type.id());
            let ast_ptr = SyntaxNodePtr::new(ty.syntax());

            RootOperationTypeDefinition {
                object_type_id,
                operation_type,
                named_type,
                ast_ptr: Some(ast_ptr),
            }
        })
        .collect();

    Arc::new(type_defs)
}

fn operation_type(op_type: Option<ast::OperationType>) -> OperationType {
    match op_type {
        Some(ty) => {
            if ty.query_token().is_some() {
                OperationType::Query
            } else if ty.mutation_token().is_some() {
                OperationType::Mutation
            } else if ty.subscription_token().is_some() {
                OperationType::Subscription
            } else {
                OperationType::Query
            }
        }
        None => OperationType::Query,
    }
}

fn variable_definitions(
    variable_definitions: Option<ast::VariableDefinitions>,
) -> Arc<Vec<VariableDefinition>> {
    match variable_definitions {
        Some(vars) => {
            let variable_definitions = vars
                .variable_definitions()
                .into_iter()
                .map(variable_definition)
                .collect();
            Arc::new(variable_definitions)
        }
        None => Arc::new(Vec::new()),
    }
}

fn variable_definition(var: ast::VariableDefinition) -> VariableDefinition {
    let name = name(
        var.variable()
            .expect("Variable Definition must have a variable")
            .name(),
    );
    let directives = directives(var.directives());
    let default_value = default_value(var.default_value());
    let ty = ty(var.ty().expect("Variable Definition must have a type"));
    let ast_ptr = SyntaxNodePtr::new(var.syntax());

    VariableDefinition {
        name,
        directives,
        ty,
        default_value,
        ast_ptr,
    }
}

fn ty(ty_: ast::Type) -> Type {
    match ty_ {
        ast::Type::NamedType(name) => named_type(name.name()),
        ast::Type::ListType(list) => Type::List {
            ty: Box::new(ty(list.ty().expect("List Type must have a type"))),
            ast_ptr: Some(SyntaxNodePtr::new(list.syntax())),
        },
        ast::Type::NonNullType(non_null) => {
            if let Some(n) = non_null.named_type() {
                let named_type = named_type(n.name());
                Type::NonNull {
                    ty: Box::new(named_type),
                    ast_ptr: Some(SyntaxNodePtr::new(n.syntax())),
                }
            } else if let Some(list) = non_null.list_type() {
                let list_type = Type::List {
                    ty: Box::new(ty(list.ty().expect("List Type must have a type"))),
                    ast_ptr: Some(SyntaxNodePtr::new(list.syntax())),
                };
                Type::NonNull {
                    ty: Box::new(list_type),
                    ast_ptr: Some(SyntaxNodePtr::new(list.syntax())),
                }
            } else {
                // TODO: parser should have caught an error if there wasn't
                // either a named type or list type. Figure out a graceful way
                // to surface this error from the parser.
                panic!("Parser should have caught this error");
            }
        }
    }
}

fn named_type(n: Option<ast::Name>) -> Type {
    let name = n.expect("Named Type must have a name");

    Type::Named {
        name: name.text().to_string(),
        ast_ptr: Some(SyntaxNodePtr::new(name.syntax())),
    }
}

fn directive_locations(
    directive_locations: Option<ast::DirectiveLocations>,
) -> Arc<Vec<DirectiveLocation>> {
    match directive_locations {
        Some(directive_loc) => {
            let locations: Vec<DirectiveLocation> = directive_loc
                .directive_locations()
                .into_iter()
                .map(|loc| loc.into())
                .collect();
            Arc::new(locations)
        }
        None => Arc::new(Vec::new()),
    }
}

fn directives(directives: Option<ast::Directives>) -> Arc<Vec<Directive>> {
    match directives {
        Some(directives) => {
            let directives = directives.directives().into_iter().map(directive).collect();
            Arc::new(directives)
        }
        None => Arc::new(Vec::new()),
    }
}

fn directive(directive: ast::Directive) -> Directive {
    let name = name(directive.name());
    let arguments = arguments(directive.arguments());
    let ast_ptr = SyntaxNodePtr::new(directive.syntax());

    Directive {
        name,
        arguments,
        ast_ptr,
    }
}

fn arguments(arguments: Option<ast::Arguments>) -> Arc<Vec<Argument>> {
    match arguments {
        Some(arguments) => {
            let arguments = arguments.arguments().into_iter().map(argument).collect();
            Arc::new(arguments)
        }
        None => Arc::new(Vec::new()),
    }
}

fn argument(argument: ast::Argument) -> Argument {
    let name = name(argument.name());
    let value = value(argument.value().expect("Argument must have a value"));
    let ast_ptr = SyntaxNodePtr::new(argument.syntax());

    Argument {
        name,
        value,
        ast_ptr,
    }
}

fn value(val: ast::Value) -> Value {
    match val {
        ast::Value::Variable(var) => Value::Variable(Variable {
            name: name(var.name()),
            ast_ptr: SyntaxNodePtr::new(var.syntax()),
        }),
        ast::Value::StringValue(string_val) => Value::String(string_val.into()),
        ast::Value::FloatValue(float) => Value::Float(Float::new(float.into())),
        ast::Value::IntValue(int) => Value::Int(int.into()),
        ast::Value::BooleanValue(bool) => Value::Boolean(bool.into()),
        ast::Value::NullValue(_) => Value::Null,
        ast::Value::EnumValue(enum_) => Value::Enum(name(enum_.name())),
        ast::Value::ListValue(list) => {
            let list: Vec<Value> = list.values().map(value).collect();
            Value::List(list)
        }
        ast::Value::ObjectValue(object) => {
            let object_values: Vec<(String, Value)> = object
                .object_fields()
                .map(|o| {
                    let name = name(o.name());
                    let value = value(o.value().expect("Object Value must have a value"));
                    (name, value)
                })
                .collect();
            Value::Object(object_values)
        }
    }
}

fn selection_set(
    db: &dyn SourceDatabase,
    selections: Option<ast::SelectionSet>,
    object_id: Option<Uuid>,
) -> SelectionSet {
    let selection_set = match selections {
        Some(sel) => sel
            .selections()
            .into_iter()
            .map(|sel| selection(db, sel, object_id))
            .collect(),
        None => Vec::new(),
    };

    SelectionSet {
        selection: Arc::new(selection_set),
    }
}

fn selection(
    db: &dyn SourceDatabase,
    selection: ast::Selection,
    object_id: Option<Uuid>,
) -> Selection {
    match selection {
        ast::Selection::Field(sel_field) => {
            let field = field(db, sel_field, object_id);
            Selection::Field(field)
        }
        ast::Selection::FragmentSpread(fragment) => {
            let fragment_spread = fragment_spread(fragment);
            Selection::FragmentSpread(fragment_spread)
        }
        ast::Selection::InlineFragment(fragment) => {
            let inline_fragment = inline_fragment(db, fragment, object_id);
            Selection::InlineFragment(inline_fragment)
        }
    }
}

fn inline_fragment(
    db: &dyn SourceDatabase,
    fragment: ast::InlineFragment,
    object_id: Option<Uuid>,
) -> Arc<InlineFragment> {
    let type_condition = fragment.type_condition().map(|tc| {
        tc.named_type()
            .expect("Type Condition must have a name")
            .name()
            .expect("Name must have text")
            .text()
            .to_string()
    });
    let reference_ty_id = if let Some(type_condition) = type_condition.clone() {
        db.find_type_system_definition_by_name(type_condition)
            .and_then(|def| def.id().cloned())
    } else {
        object_id
    };
    let directives = directives(fragment.directives());
    let selection_set: SelectionSet = selection_set(db, fragment.selection_set(), reference_ty_id);
    let ast_ptr = SyntaxNodePtr::new(fragment.syntax());

    let fragment_data = InlineFragment {
        type_condition,
        directives,
        selection_set,
        ast_ptr,
    };
    Arc::new(fragment_data)
}

fn fragment_spread(fragment: ast::FragmentSpread) -> Arc<FragmentSpread> {
    let name = name(
        fragment
            .fragment_name()
            .expect("Fragment Spread must have a name")
            .name(),
    );
    let directives = directives(fragment.directives());
    let ast_ptr = SyntaxNodePtr::new(fragment.syntax());

    let fragment_data = FragmentSpread {
        name,
        directives,
        ast_ptr,
    };
    Arc::new(fragment_data)
}

fn field(db: &dyn SourceDatabase, field: ast::Field, ty_id: Option<Uuid>) -> Arc<Field> {
    let name = name(field.name());
    let alias = alias(field.alias());
    let ty = field_ty(db, &name, ty_id);
    let new_ty_id = field_ty_id(&field, db, ty.as_ref());
    let selection_set = selection_set(db, field.selection_set(), new_ty_id);
    let directives = directives(field.directives());
    let arguments = arguments(field.arguments());
    let ast_ptr = SyntaxNodePtr::new(field.syntax());

    let field_data = Field {
        name,
        alias,
        selection_set,
        ty,
        reference_ty_id: ty_id,
        directives,
        arguments,
        ast_ptr,
    };
    Arc::new(field_data)
}

fn field_ty(db: &dyn SourceDatabase, field_name: &str, ty_id: Option<Uuid>) -> Option<Type> {
    if let Some(id) = ty_id {
        Some(
            db.find_type_system_definition(id)?
                .field(field_name)?
                .ty()
                .clone(),
        )
    } else {
        None
    }
}

fn field_ty_id(
    field: &ast::Field,
    db: &dyn SourceDatabase,
    field_ty: Option<&Type>,
) -> Option<Uuid> {
    match field_ty {
        Some(ty) => {
            if field.selection_set().is_some() {
                db.find_type_system_definition_by_name(ty.name())
                    .and_then(|def| def.id().cloned())
            } else {
                None
            }
        }
        None => None,
    }
}

fn name(name: Option<ast::Name>) -> String {
    name.expect("Field must have a name").text().to_string()
}

fn enum_value(enum_value: Option<ast::EnumValue>) -> String {
    enum_value
        .expect("Enum value must have a name")
        .name()
        .expect("Name must have text")
        .text()
        .to_string()
}

fn description(description: Option<ast::Description>) -> Option<String> {
    description.map(|desc| {
        desc.string_value()
            .expect("Description must have text")
            .into()
    })
}

fn alias(alias: Option<ast::Alias>) -> Option<Arc<Alias>> {
    alias.map(|alias| {
        let name = alias.name().expect("Alias must have a name").to_string();
        let alias_data = Alias(name);
        Arc::new(alias_data)
    })
}

fn object_type_uuid(db: &dyn SourceDatabase, ty: OperationType) -> Option<Uuid> {
    match ty {
        OperationType::Query => db.schema().query(db).map(|query| *query.id()),
        OperationType::Mutation => db.schema().mutation(db).map(|mutation| *mutation.id()),
        OperationType::Subscription => db.schema().subscription(db).map(|sub| *sub.id()),
    }
}

//  Int, Float, String, Boolean, and ID
fn built_in_scalars(mut scalars: Vec<ScalarTypeDefinition>) -> Vec<ScalarTypeDefinition> {
    scalars.push(int_scalar());
    scalars.push(float_scalar());
    scalars.push(string_scalar());
    scalars.push(boolean_scalar());
    scalars.push(id_scalar());

    scalars
}

fn int_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.".into()),
        name: "Int".into(),
        directives: Arc::new(Vec::new()),
        ast_ptr: None,
        built_in: true
    }
}

fn float_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).".into()),
        name: "Float".into(),
        directives: Arc::new(Vec::new()),
        ast_ptr: None,
        built_in: true
    }
}

fn string_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.".into()),
        name: "String".into(),
        directives: Arc::new(Vec::new()),
        ast_ptr: None,
        built_in: true
    }
}

fn boolean_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Boolean` scalar type represents `true` or `false`.".into()),
        name: "Boolean".into(),
        directives: Arc::new(Vec::new()),
        ast_ptr: None,
        built_in: true,
    }
}

fn id_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `ID` scalar type represents a unique identifier, often used to refetch an object or as key for a cache. The ID type appears in a JSON response as a String; however, it is not intended to be human-readable. When expected as an input type, any string (such as `\"4\"`) or integer (such as `4`) input value will be accepted as an ID.".into()),
        name: "ID".into(),
        directives: Arc::new(Vec::new()),
        ast_ptr: None,
        built_in: true
    }
}

fn built_in_directives(mut directives: Vec<DirectiveDefinition>) -> Vec<DirectiveDefinition> {
    if !directives.iter().any(|dir| dir.name == "skip") {
        directives.push(skip_directive());
    }

    if !directives.iter().any(|dir| dir.name == "specifiedBy") {
        directives.push(specified_by_directive());
    }

    if !directives.iter().any(|dir| dir.name == "deprecated") {
        directives.push(deprecated_directive());
    }

    if !directives.iter().any(|dir| dir.name == "include") {
        directives.push(include_directive());
    }

    directives
}

fn skip_directive() -> DirectiveDefinition {
    // "Directs the executor to skip this field or fragment when the `if` argument is true."
    // directive @skip(
    //   "Skipped when true."
    //   if: Boolean!
    // ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some(
            "Directs the executor to skip this field or fragment when the `if` argument is true."
                .into(),
        ),
        name: "skip".into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some("Skipped when true.".into()),
                name: "if".into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "Boolean".into(),
                        ast_ptr: None,
                    }),
                    ast_ptr: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                ast_ptr: None,
            }]),
            ast_ptr: None,
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::Field,
            DirectiveLocation::FragmentSpread,
            DirectiveLocation::InlineFragment,
        ]),
        ast_ptr: None,
    }
}

fn specified_by_directive() -> DirectiveDefinition {
    // "Exposes a URL that specifies the behaviour of this scalar."
    // directive @specifiedBy(
    //     "The URL that specifies the behaviour of this scalar."
    //     url: String!
    // ) on SCALAR
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Exposes a URL that specifies the behaviour of this scalar.".into()),
        name: "specifiedBy".into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some("The URL that specifies the behaviour of this scalar.".into()),
                name: "url".into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "String".into(),
                        ast_ptr: None,
                    }),
                    ast_ptr: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                ast_ptr: None,
            }]),
            ast_ptr: None,
        },
        repeatable: false,
        directive_locations: Arc::new(vec![DirectiveLocation::Scalar]),
        ast_ptr: None,
    }
}

fn deprecated_directive() -> DirectiveDefinition {
    // "Marks an element of a GraphQL schema as no longer supported."
    // directive @deprecated(
    //   """
    //   Explains why this element was deprecated, usually also including a
    //   suggestion for how to access supported similar data. Formatted using
    //   the Markdown syntax, as specified by
    //   [CommonMark](https://commonmark.org/).
    //   """
    //   reason: String = "No longer supported"
    // ) on FIELD_DEFINITION | ENUM_VALUE
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Marks an element of a GraphQL schema as no longer supported.".into()),
        name: "deprecated".into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some(
                    "Explains why this element was deprecated, usually also including a suggestion for how to access supported similar data. Formatted using the Markdown syntax, as specified by [CommonMark](https://commonmark.org/).".into(),
                ),
                name: "reason".into(),
                ty: Type::Named {
                    name: "String".into(),
                    ast_ptr: None,
                },
                default_value: Some(DefaultValue::String("No longer supported".into())),
                directives: Arc::new(Vec::new()),
                ast_ptr: None
            }]),
            ast_ptr: None
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::FieldDefinition,
            DirectiveLocation::EnumValue
        ]),
        ast_ptr: None
    }
}

fn include_directive() -> DirectiveDefinition {
    // "Directs the executor to include this field or fragment only when the `if` argument is true."
    // directive @include(
    //   "Included when true."
    //   if: Boolean!
    // ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Directs the executor to include this field or fragment only when the `if` argument is true.".into()),
        name: "include".into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some(
                    "Included when true.".into(),
                ),
                name: "if".into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "Boolean".into(),
                        ast_ptr: None,
                    }),
                    ast_ptr: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                ast_ptr: None
            }]),
            ast_ptr: None
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::Field,
            DirectiveLocation::FragmentDefinition,
            DirectiveLocation::InlineFragment,
        ]),
        ast_ptr: None
    }
}
