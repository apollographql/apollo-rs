// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use std::collections::HashSet;
use std::sync::Arc;

use apollo_parser::{ast, Parser, SyntaxTree};

use crate::diagnostics::ApolloDiagnostic;
use crate::values::*;

#[salsa::database(ASTDatabase)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}

#[salsa::query_group(ASTDatabase)]
pub trait SourceDatabase {
    #[salsa::input]
    fn input_string(&self, key: ()) -> Arc<String>;

    fn parse(&self) -> Arc<SyntaxTree>;

    fn document(&self) -> Arc<ast::Document>;

    fn syntax_errors(&self) -> Arc<Vec<ApolloDiagnostic>>;

    fn definitions(&self) -> Arc<Vec<ast::Definition>>;

    fn operations(&self) -> Operations;

    fn find_operation(&self, name: String) -> Option<Arc<OperationDefinition>>;

    fn operation_definition_defined_variables(&self, name: String) -> Option<Arc<HashSet<String>>>;

    fn operation_definition_in_use_variables(&self, name: String) -> Option<Arc<HashSet<String>>>;

    fn operation_fields(&self, name: String) -> Option<Arc<Vec<Field>>>;

    fn operation_definitions_names(&self) -> Arc<Vec<String>>;

    fn fragments(&self) -> Fragments;
}

// this is top level entry to the source db
fn parse(db: &dyn SourceDatabase) -> Arc<SyntaxTree> {
    let input = db.input_string(());
    let parser = Parser::new(&input);
    Arc::new(parser.parse())
}

// NOTE: a very expensive clone - should more tightly couple the parser and the
// source database for a cleaner solution
fn document(db: &dyn SourceDatabase) -> Arc<ast::Document> {
    Arc::new(db.parse().as_ref().clone().document())
}

fn syntax_errors(db: &dyn SourceDatabase) -> Arc<Vec<ApolloDiagnostic>> {
    let errors = db
        .parse()
        .errors()
        .into_iter()
        .map(|err| ApolloDiagnostic::SyntaxError {
            message: err.message().to_string(),
            data: err.data().to_string(),
            index: err.index(),
        })
        .collect();
    Arc::new(errors)
}

fn definitions(db: &dyn SourceDatabase) -> Arc<Vec<ast::Definition>> {
    Arc::new(db.document().definitions().into_iter().collect())
}

// NOTE: we might want to the values::OperationDefinition creation even further.
// At the moment all fields in this struct are created here, instead individual
// queries for selection_set, variables, directives etc can be created.
fn operations(db: &dyn SourceDatabase) -> Operations {
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
    Operations::new(Arc::new(operations))
}

fn fragments(db: &dyn SourceDatabase) -> Fragments {
    let operations: Vec<FragmentDefinition> = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::FragmentDefinition(fragment_def) => {
                Some(fragment_definition(db, fragment_def.clone()))
            }
            _ => None,
        })
        .collect();
    Fragments::new(Arc::new(operations))
}

fn find_operation(db: &dyn SourceDatabase, name: String) -> Option<Arc<OperationDefinition>> {
    db.operations().iter().find_map(|op| {
        if let Some(n) = op.name() {
            if n == name {
                return Some(Arc::new(op.clone()));
            }
        }
        None
    })
}

fn operation_definitions_names(db: &dyn SourceDatabase) -> Arc<Vec<String>> {
    Arc::new(db.operations().iter().filter_map(|n| n.name()).collect())
}

fn operation_definition_defined_variables(
    db: &dyn SourceDatabase,
    op_name: String,
) -> Option<Arc<HashSet<String>>> {
    let vars: HashSet<String> = db
        .find_operation(op_name)?
        .variables()?
        .iter()
        .map(|v| v.name())
        .collect();
    Some(Arc::new(vars))
}

fn operation_fields(db: &dyn SourceDatabase, op_name: String) -> Option<Arc<Vec<Field>>> {
    let fields: Vec<Field> = db
        .find_operation(op_name)?
        .selection_set()
        .iter()
        .map(|sel| match sel {
            Selection::Field(field) => field.as_ref().clone(),
        })
        .collect();
    Some(Arc::new(fields))
}

// NOTE: potentially want to return a hashmap of variables and their types?
fn operation_definition_in_use_variables(
    db: &dyn SourceDatabase,
    op_name: String,
) -> Option<Arc<HashSet<String>>> {
    // TODO: once FragmentSpread and InlineFragment are added, get their fields
    // and combine all variable usage.
    let vars = db
        .operation_fields(op_name)?
        .iter()
        .flat_map(|field| {
            if let Some(args) = field.arguments() {
                let vars: Vec<String> = args
                    .iter()
                    .flat_map(|arg| get_field_variable_value(arg.value.clone()))
                    .collect();
                return vars;
            }
            Vec::new()
        })
        .collect();
    Some(Arc::new(vars))
}

fn get_field_variable_value(val: Value) -> Vec<String> {
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

fn operation_definition(
    db: &dyn SourceDatabase,
    op_def: ast::OperationDefinition,
) -> OperationDefinition {
    // check if there are already operations
    // if there are operations, they must have names
    // if there are no names, an error must be raised that all operations must have a name
    let name = op_def.name().map(|name| name.to_string());
    let ty = operation_type(op_def.operation_type());
    let variables = variable_definitions(op_def.variable_definitions());
    let selections = op_def
        .selection_set()
        .expect("Operation Definition must have a Selection Set")
        .selections();
    let selection_set = selection_set(db, selections);
    let directives = directives(op_def.directives());

    OperationDefinition {
        ty,
        name,
        variables,
        selection_set,
        directives,
    }
}

fn fragment_definition(
    db: &dyn SourceDatabase,
    fragment_def: ast::FragmentDefinition,
) -> FragmentDefinition {
    let name = fragment_def
        .fragment_name()
        .expect("Fragment Definition must have a name")
        .name()
        .expect("Name must have text")
        .text()
        .to_string();
    let type_condition = fragment_def
        .type_condition()
        .expect("Fragment Definition must have a type condition")
        .named_type()
        .expect("Type Condition must have a name")
        .name()
        .expect("Name must have text")
        .text()
        .to_string();
    let selections = fragment_def
        .selection_set()
        .expect("Operation Definition must have a Selection Set")
        .selections();
    let selection_set = selection_set(db, selections);
    let directives = directives(fragment_def.directives());

    FragmentDefinition {
        name,
        type_condition,
        selection_set,
        directives,
    }
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
) -> Option<Arc<Vec<VariableDefinition>>> {
    match variable_definitions {
        Some(vars) => {
            let variable_definitions = vars
                .variable_definitions()
                .into_iter()
                .map(variable_definition)
                .collect();
            Some(Arc::new(variable_definitions))
        }
        None => None,
    }
}

fn variable_definition(var: ast::VariableDefinition) -> VariableDefinition {
    let name = var
        .variable()
        .expect("Variable Definition must have a variable")
        .name()
        .expect("Variable must have a name")
        .to_string();
    let directives = directives(var.directives());
    let default_value = var
        .default_value()
        .map(|val| value(val.value().expect("Default Value must have a value")));
    let ty = ty(var.ty().expect("Variable Definition must have a type"));

    VariableDefinition {
        name,
        directives,
        ty,
        default_value,
    }
}

fn ty(ty_: ast::Type) -> Type {
    match ty_ {
        ast::Type::NamedType(name) => Type::Named {
            name: name
                .name()
                .expect("NamedType must have text")
                .text()
                .to_string(),
        },
        ast::Type::ListType(list) => Type::List {
            ty: Box::new(ty(list.ty().expect("List Type must have a type"))),
        },
        ast::Type::NonNullType(non_null) => {
            if let Some(name) = non_null.named_type() {
                let named_type = Type::Named {
                    name: name
                        .name()
                        .expect("NamedType must have text")
                        .text()
                        .to_string(),
                };
                Type::NonNull {
                    ty: Box::new(named_type),
                }
            } else if let Some(list) = non_null.list_type() {
                let list_type = Type::List {
                    ty: Box::new(ty(list.ty().expect("List Type must have a type"))),
                };
                Type::NonNull {
                    ty: Box::new(list_type),
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

fn directives(directives: Option<ast::Directives>) -> Option<Arc<Vec<Directive>>> {
    match directives {
        Some(directives) => {
            let directives = directives.directives().into_iter().map(directive).collect();
            Some(Arc::new(directives))
        }
        None => None,
    }
}

fn directive(directive: ast::Directive) -> Directive {
    let name = directive
        .name()
        .expect("Directive must have a name")
        .to_string();
    let arguments = arguments(directive.arguments());
    Directive { name, arguments }
}

fn arguments(arguments: Option<ast::Arguments>) -> Option<Arc<Vec<Argument>>> {
    match arguments {
        Some(arguments) => {
            let arguments = arguments.arguments().into_iter().map(argument).collect();
            Some(Arc::new(arguments))
        }
        None => None,
    }
}

fn argument(argument: ast::Argument) -> Argument {
    let name = argument
        .name()
        .expect("Argument must have a name")
        .to_string();
    let value = value(argument.value().expect("Argument must have a value"));
    Argument { name, value }
}

fn value(val: ast::Value) -> Value {
    match val {
        ast::Value::Variable(var) => Value::Variable(
            var.name()
                .expect("Variable must have a name")
                .text()
                .to_string(),
        ),
        ast::Value::StringValue(string_val) => Value::Variable(string_val.into()),
        ast::Value::FloatValue(float) => Value::Float(Float::new(float.into())),
        ast::Value::IntValue(int) => Value::Int(int.into()),
        ast::Value::BooleanValue(bool) => Value::Boolean(bool.into()),
        ast::Value::NullValue(_) => Value::Null,
        ast::Value::EnumValue(enum_) => Value::Enum(
            enum_
                .name()
                .expect("Enum Value must have a name")
                .text()
                .to_string(),
        ),
        ast::Value::ListValue(list) => {
            let list: Vec<Value> = list.values().map(value).collect();
            Value::List(list)
        }
        ast::Value::ObjectValue(object) => {
            let object_values: Vec<(String, Value)> = object
                .object_fields()
                .map(|o| {
                    let name = o
                        .name()
                        .expect("Object Value must have a name")
                        .text()
                        .to_string();
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
    selections: ast::AstChildren<ast::Selection>,
) -> Arc<Vec<Selection>> {
    let selections = selections
        .into_iter()
        .map(|sel| selection(db, sel))
        .collect();
    Arc::new(selections)
}

fn selection(db: &dyn SourceDatabase, selection: ast::Selection) -> Selection {
    match selection {
        ast::Selection::Field(sel_field) => {
            let field = field(db, sel_field);
            Selection::Field(field)
        }
        ast::Selection::FragmentSpread(_) => unimplemented!(),
        ast::Selection::InlineFragment(_) => unimplemented!(),
    }
}

fn field(db: &dyn SourceDatabase, field: ast::Field) -> Arc<Field> {
    let name = field.name().expect("Field must have a name").to_string();
    let alias = alias(field.alias());
    let selection_set = field
        .selection_set()
        .map(|sel_set| selection_set(db, sel_set.selections()));
    let directives = directives(field.directives());
    let arguments = arguments(field.arguments());

    let field_data = Field {
        name,
        alias,
        selection_set,
        directives,
        arguments,
    };
    Arc::new(field_data)
}

fn alias(alias: Option<ast::Alias>) -> Option<Arc<Alias>> {
    alias.map(|alias| {
        let name = alias.name().expect("Alias must have a name").to_string();
        let alias_data = Alias(name);
        Arc::new(alias_data)
    })
}
