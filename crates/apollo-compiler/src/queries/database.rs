// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use std::collections::HashSet;
use std::sync::Arc;

use apollo_parser::ast::AstChildren;
use apollo_parser::{ast, Parser, SyntaxTree};
use uuid::Uuid;

use crate::diagnostics::{ApolloDiagnostic, ErrorDiagnostic};
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

    fn operations(&self) -> Operations;

    fn fragments(&self) -> Fragments;

    fn schema(&self) -> Arc<SchemaDefinition>;

    fn object_types(&self) -> Arc<Vec<ObjectTypeDefinition>>;

    fn query_operations(&self) -> Operations;

    fn mutation_operations(&self) -> Operations;

    fn subscription_operations(&self) -> Operations;

    fn find_operation(&self, id: Uuid) -> Option<Arc<OperationDefinition>>;

    fn find_fragment(&self, id: Uuid) -> Option<Arc<FragmentDefinition>>;

    fn find_fragment_by_name(&self, name: String) -> Option<Arc<FragmentDefinition>>;

    fn find_object_type(&self, id: Uuid) -> Option<Arc<ObjectTypeDefinition>>;

    fn find_object_type_by_name(&self, name: String) -> Option<Arc<ObjectTypeDefinition>>;

    fn operation_definition_variables(&self, id: Uuid) -> Arc<HashSet<String>>;

    fn selection_variables(&self, id: Uuid) -> Arc<HashSet<String>>;

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

// NOTE: a very expensive clone - should more tightly couple the parser and the
// source database for a cleaner solution
fn document(db: &dyn SourceDatabase) -> Arc<ast::Document> {
    Arc::new(db.parse().as_ref().clone().document())
}

fn syntax_errors(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.parse()
        .errors()
        .into_iter()
        .map(|err| {
            ApolloDiagnostic::Error(ErrorDiagnostic::SyntaxError {
                message: err.message().to_string(),
                data: err.data().to_string(),
                index: err.index(),
            })
        })
        .collect()
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

fn query_operations(db: &dyn SourceDatabase) -> Operations {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.ty.is_query().then(|| op.clone()))
        .collect();
    Operations::new(Arc::new(operations))
}

fn subscription_operations(db: &dyn SourceDatabase) -> Operations {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.ty.is_subscription().then(|| op.clone()))
        .collect();
    Operations::new(Arc::new(operations))
}

fn mutation_operations(db: &dyn SourceDatabase) -> Operations {
    let operations = db
        .operations()
        .iter()
        .filter_map(|op| op.ty.is_mutation().then(|| op.clone()))
        .collect();
    Operations::new(Arc::new(operations))
}

fn find_operation(db: &dyn SourceDatabase, id: Uuid) -> Option<Arc<OperationDefinition>> {
    db.operations().iter().find_map(|op| {
        if &id == op.id() {
            return Some(Arc::new(op.clone()));
        }
        None
    })
}

// NOTE: potentially want to return a hashset of variables and their types?
fn operation_definition_variables(db: &dyn SourceDatabase, id: Uuid) -> Arc<HashSet<Variable>> {
    let vars: HashSet<String> = match db.find_operation(id) {
        Some(op) => op.variables().iter().map(|v| v.name()).collect(),
        None => HashSet::new(),
    };
    Arc::new(vars)
}

fn operation_fields(db: &dyn SourceDatabase, id: Uuid) -> Arc<Vec<Field>> {
    let fields = match db.find_operation(id) {
        Some(op) => op
            .selection_set()
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
            .iter()
            .filter_map(|sel| match sel {
                Selection::InlineFragment(fragment) => {
                    let fields: Vec<Field> = fragment
                        .selection_set()
                        .iter()
                        .filter_map(|sel| match sel {
                            Selection::Field(field) => Some(field.as_ref().clone()),
                            _ => None,
                        })
                        .collect();
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
            .iter()
            .filter_map(|sel| match sel {
                Selection::FragmentSpread(fragment_spread) => {
                    let fields: Vec<Field> = fragment_spread
                        .fragment(db)?
                        .selection_set()
                        .iter()
                        .filter_map(|sel| match sel {
                            Selection::Field(field) => Some(field.as_ref().clone()),
                            _ => None,
                        })
                        .collect();
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
fn selection_variables(db: &dyn SourceDatabase, id: Uuid) -> Arc<HashSet<String>> {
    // TODO: once FragmentSpread and InlineFragment are added, get their fields
    // and combine all variable usage.
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

fn fragments(db: &dyn SourceDatabase) -> Fragments {
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
    Fragments::new(Arc::new(fragments))
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
    let schema_def = schema.map_or(SchemaDefinition::default(), |s| schema_definition(db, s));
    Arc::new(schema_def)
}

fn object_types(db: &dyn SourceDatabase) -> Arc<Vec<ObjectTypeDefinition>> {
    let objects = db
        .definitions()
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::ObjectTypeDefinition(obj_def) => {
                Some(object_type_definition(db, obj_def.clone()))
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

fn operation_definition(
    db: &dyn SourceDatabase,
    op_def: ast::OperationDefinition,
) -> OperationDefinition {
    // check if there are already operations
    // if there are operations, they must have names
    // if there are no names, an error must be raised that all operations must have a name
    let name = op_def.name().map(|name| name.text().to_string());
    let ty = operation_type(op_def.operation_type());
    let variables = variable_definitions(op_def.variable_definitions());
    let selection_set = selection_set(db, op_def.selection_set());
    let directives = directives(op_def.directives());

    OperationDefinition {
        id: Uuid::new_v4(),
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
    let selection_set = selection_set(db, fragment_def.selection_set());
    let directives = directives(fragment_def.directives());

    FragmentDefinition {
        id: Uuid::new_v4(),
        name,
        type_condition,
        selection_set,
        directives,
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

    SchemaDefinition {
        description,
        directives,
        root_operation_type_definition,
    }
}

fn object_type_definition(
    db: &dyn SourceDatabase,
    obj_def: ast::ObjectTypeDefinition,
) -> ObjectTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(obj_def.description());
    let name = name(obj_def.name());
    let implements_interfaces = implements_interfaces(obj_def.implements_interfaces());
    let directives = directives(obj_def.directives());
    let fields_definition = fields_definition(obj_def.fields_definition());

    add_object_type_id_to_schema(db, id, &name);

    ObjectTypeDefinition {
        id,
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
    }
}

fn add_object_type_id_to_schema(db: &dyn SourceDatabase, id: Uuid, name: &str) {
    // Schema Definition does not have to be present in the SDL if ObjectType name is
    // - Query
    // - Subscription
    // - Mutation
    //
    // Compiler's internal schema, however, should have a reference to these
    // object types if they are present
    if let Some(mut root_op) = db
        .schema()
        .root_operation_type_definition()
        .iter()
        .find(|op| op.named_type().name() == name)
        .cloned()
    {
        root_op.object_type_id = Some(id)
    } else if matches!(name, "Query" | "Subscription" | "Mutation") {
        let operation_type = match_operation_type(name);
        let root_op = RootOperationTypeDefinition {
            object_type_id: Some(id),
            operation_type,
            named_type: Type::Named {
                name: name.to_string(),
            },
        };
        db.schema()
            .as_ref()
            .clone()
            .set_root_operation_type_definition(root_op);
    }
}

fn implements_interfaces(
    implements_interfaces: Option<ast::ImplementsInterfaces>,
) -> ImplementsInterfaces {
    let interfaces: Vec<Type> = implements_interfaces
        .iter()
        .flat_map(|interfaces| {
            let types: Vec<Type> = interfaces
                .named_types()
                .map(|n| Type::Named {
                    name: n.name().expect("Name must have text").text().to_string(),
                })
                .collect();
            types
        })
        .collect();

    ImplementsInterfaces {
        interfaces: Arc::new(interfaces),
    }
}

fn fields_definition(
    fields_definition: Option<ast::FieldsDefinition>,
) -> Arc<Vec<FieldDefinition>> {
    match fields_definition {
        Some(fields_def) => {
            let fields: Vec<FieldDefinition> = fields_def
                .field_definitions()
                .map(|field| {
                    let description = description(field.description());
                    let name = name(field.name());
                    let arguments = arguments_definition(field.arguments_definition());
                    let ty = ty(field.ty().expect("Field must have a type"));
                    let directives = directives(field.directives());

                    FieldDefinition {
                        description,
                        name,
                        arguments,
                        ty,
                        directives,
                    }
                })
                .collect();
            Arc::new(fields)
        }
        None => Arc::new(Vec::new()),
    }
}

fn arguments_definition(
    arguments_definition: Option<ast::ArgumentsDefinition>,
) -> ArgumentsDefinition {
    match arguments_definition {
        Some(arguments) => {
            let input_values = input_value_definitions(arguments.input_value_definitions());
            ArgumentsDefinition { input_values }
        }
        None => ArgumentsDefinition {
            input_values: Arc::new(Vec::new()),
        },
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

            InputValueDefinition {
                description,
                name,
                ty,
                default_value,
                directives,
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

            RootOperationTypeDefinition {
                object_type_id,
                operation_type,
                named_type,
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
fn match_operation_type(name: &str) -> OperationType {
    if name == "Subscription" {
        OperationType::Subscription
    } else if name == "Mutation" {
        OperationType::Mutation
    } else {
        OperationType::Query
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

    VariableDefinition {
        name,
        directives,
        ty,
        default_value,
    }
}

fn ty(ty_: ast::Type) -> Type {
    match ty_ {
        ast::Type::NamedType(name) => named_type(name.name()),
        ast::Type::ListType(list) => Type::List {
            ty: Box::new(ty(list.ty().expect("List Type must have a type"))),
        },
        ast::Type::NonNullType(non_null) => {
            if let Some(n) = non_null.named_type() {
                let named_type = named_type(n.name());
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

fn named_type(n: Option<ast::Name>) -> Type {
    Type::Named { name: name(n) }
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
    Directive { name, arguments }
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
    Argument { name, value }
}

fn value(val: ast::Value) -> Value {
    match val {
        ast::Value::Variable(var) => Value::Variable(name(var.name())),
        ast::Value::StringValue(string_val) => Value::Variable(string_val.into()),
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
) -> Arc<Vec<Selection>> {
    let selection_set = match selections {
        Some(sel) => sel
            .selections()
            .into_iter()
            .map(|sel| selection(db, sel))
            .collect(),
        None => Vec::new(),
    };
    Arc::new(selection_set)
}

fn selection(db: &dyn SourceDatabase, selection: ast::Selection) -> Selection {
    match selection {
        ast::Selection::Field(sel_field) => {
            let field = field(db, sel_field);
            Selection::Field(field)
        }
        ast::Selection::FragmentSpread(fragment) => {
            let fragment_spread = fragment_spread(db, fragment);
            Selection::FragmentSpread(fragment_spread)
        }
        ast::Selection::InlineFragment(fragment) => {
            let inline_fragment = inline_fragment(db, fragment);
            Selection::InlineFragment(inline_fragment)
        }
    }
}

fn inline_fragment(db: &dyn SourceDatabase, fragment: ast::InlineFragment) -> Arc<InlineFragment> {
    let type_condition = fragment.type_condition().map(|tc| {
        tc.named_type()
            .expect("Type Condition must have a name")
            .name()
            .expect("Name must have text")
            .text()
            .to_string()
    });
    let directives = directives(fragment.directives());
    let selection_set: Arc<Vec<Selection>> = selection_set(db, fragment.selection_set());

    let fragment_data = InlineFragment {
        type_condition,
        directives,
        selection_set,
    };
    Arc::new(fragment_data)
}

fn fragment_spread(db: &dyn SourceDatabase, fragment: ast::FragmentSpread) -> Arc<FragmentSpread> {
    let name = name(
        fragment
            .fragment_name()
            .expect("Fragment Spread must have a name")
            .name(),
    );
    let directives = directives(fragment.directives());
    // NOTE @lrlna: this should just be Uuid.  If we can't find the framgment we
    // are looking for when populating this field, we should throw a semantic
    // error.
    let fragment_id = db
        .find_fragment_by_name(name.clone())
        .map(|fragment| *fragment.id());
    let fragment_data = FragmentSpread {
        name,
        directives,
        fragment_id,
    };
    Arc::new(fragment_data)
}

fn field(db: &dyn SourceDatabase, field: ast::Field) -> Arc<Field> {
    let name = name(field.name());
    let alias = alias(field.alias());
    let selection_set = selection_set(db, field.selection_set());
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

fn name(name: Option<ast::Name>) -> String {
    name.expect("Field must have a name").text().to_string()
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
