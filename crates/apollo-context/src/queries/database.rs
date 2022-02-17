// All .expect() calls are used for parts of the GraphQL grammar that are
// non-optional and will have an error produced in the parser if they are missing.

use std::sync::Arc;

use apollo_parser::{ast, Parser};

use crate::{
    interner::{Interner, InternerDatabase},
    values::*,
};

#[salsa::database(InternerDatabase, ASTDatabase)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}

#[salsa::query_group(ASTDatabase)]
pub trait SourceDatabase: Interner {
    #[salsa::input]
    fn input_string(&self, key: ()) -> Arc<String>;

    fn document(&self) -> ast::Document;

    fn definitions(&self) -> Arc<Vec<Definition>>;

    fn operations(&self) -> Arc<Vec<OperationDefinitionData>>;

    fn find_operation(&self, name: String) -> Option<Arc<OperationDefinitionData>>;

    fn find_fragment(&self, name: String) -> Option<Arc<FragmentDefinitionData>>;

    fn fragments(&self) -> Arc<Vec<FragmentDefinitionData>>;
}

// this is top level entry to the source db
fn document(db: &dyn SourceDatabase) -> ast::Document {
    let input = db.input_string(());
    let parser = Parser::new(&input);

    parser.parse().document()
}

fn definitions(db: &dyn SourceDatabase) -> Arc<Vec<Definition>> {
    let definitions: Vec<_> = db
        .document()
        .definitions()
        .map(|def| match def {
            ast::Definition::OperationDefinition(def) => {
                let op_def = operation_definition(db, def);
                let definition_data = DefinitionData::OperationDefinition(op_def);
                db.intern_definition(definition_data)
            }
            ast::Definition::FragmentDefinition(def) => {
                let fragment_def = fragment_definition(db, def);
                let definition_data = DefinitionData::FragmentDefinition(fragment_def);
                db.intern_definition(definition_data)
            }
            _ => todo!(),
        })
        .collect();

    Arc::new(definitions)
}

fn operations(db: &dyn SourceDatabase) -> Arc<Vec<OperationDefinitionData>> {
    let operations = db
        .definitions()
        .iter()
        .filter_map(|class| {
            let class = db.lookup_intern_definition(*class);
            match class {
                DefinitionData::OperationDefinition(op_def) => {
                    Some(db.lookup_intern_operation_definition(*op_def))
                }
                _ => None,
            }
        })
        .collect();
    Arc::new(operations)
}

fn find_operation(db: &dyn SourceDatabase, name: String) -> Option<Arc<OperationDefinitionData>> {
    db.operations().iter().find_map(|op| {
        if op.name == name {
            Some(Arc::new(op.clone()))
        } else {
            None
        }
    })
}

fn fragments(db: &dyn SourceDatabase) -> Arc<Vec<FragmentDefinitionData>> {
    let operations = db
        .definitions()
        .iter()
        .filter_map(|class| {
            let class = db.lookup_intern_definition(*class);
            match class {
                DefinitionData::FragmentDefinition(fragment_def) => {
                    Some(db.lookup_intern_fragment_definition(*fragment_def))
                }
                _ => None,
            }
        })
        .collect();
    Arc::new(operations)
}

fn find_fragment(db: &dyn SourceDatabase, name: String) -> Option<Arc<FragmentDefinitionData>> {
    db.fragments().iter().find_map(|fragment| {
        if fragment.name == name {
            Some(Arc::new(fragment.clone()))
        } else {
            None
        }
    })
}

fn operation_definition(
    db: &dyn SourceDatabase,
    op_def: ast::OperationDefinition,
) -> Arc<OperationDefinition> {
    // check if there are already operations
    // if there are operations, they must have names
    // if there are no names, an error must be raised that all operations must have a name
    let name = match op_def.name() {
        Some(name) => name.to_string(),
        None => String::from("query"),
    };
    let ty = operation_type(op_def.operation_type());
    let variables = variable_definitions(db, op_def.variable_definitions());
    let selections = op_def
        .selection_set()
        .expect("Operation Definition must have a Selection Set")
        .selections();
    let selection_set = selection_set(db, selections);
    let directives = directives(db, op_def.directives());

    let operation_def_data = OperationDefinitionData {
        ty,
        name,
        variables,
        selection_set,
        directives,
    };
    let operation_def = db.intern_operation_definition(operation_def_data);

    Arc::new(operation_def)
}

fn fragment_definition(
    db: &dyn SourceDatabase,
    fragment_def: ast::FragmentDefinition,
) -> Arc<FragmentDefinition> {
    let name = fragment_def
        .fragment_name()
        .expect("Fragment Definition must have a name")
        .to_string();
    let type_condition = fragment_def
        .type_condition()
        .expect("Fragment Definition must have a type condition")
        .named_type()
        .expect("Type Condition must have a name")
        .to_string();
    let selections = fragment_def
        .selection_set()
        .expect("Operation Definition must have a Selection Set")
        .selections();
    let selection_set = selection_set(db, selections);
    let directives = directives(db, fragment_def.directives());

    let fragment_def_data = FragmentDefinitionData {
        name,
        type_condition,
        selection_set,
        directives,
    };

    let fragment_def = db.intern_fragment_definition(fragment_def_data);

    Arc::new(fragment_def)
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
    db: &dyn SourceDatabase,
    variable_definitions: Option<ast::VariableDefinitions>,
) -> Option<Arc<Vec<VariableDefinition>>> {
    match variable_definitions {
        Some(vars) => {
            let variable_definitions = vars
                .variable_definitions()
                .into_iter()
                .map(|var| variable_definition(db, var))
                .collect();
            Some(Arc::new(variable_definitions))
        }
        None => None,
    }
}

fn variable_definition(
    db: &dyn SourceDatabase,
    var: ast::VariableDefinition,
) -> VariableDefinition {
    let name = var
        .variable()
        .expect("Variable Definition must have a variable")
        .name()
        .expect("Variable must have a name")
        .to_string();
    let directives = directives(db, var.directives());
    let var_data = VariableDefinitionData { name, directives };
    db.intern_variable_definition(var_data)
}

fn directives(
    db: &dyn SourceDatabase,
    directives: Option<ast::Directives>,
) -> Option<Arc<Vec<Directive>>> {
    match directives {
        Some(directives) => {
            let directives = directives
                .directives()
                .into_iter()
                .map(|dir| directive(db, dir))
                .collect();
            Some(Arc::new(directives))
        }
        None => None,
    }
}

fn directive(db: &dyn SourceDatabase, directive: ast::Directive) -> Directive {
    let name = directive
        .name()
        .expect("Directive must have a name")
        .to_string();
    let arguments = arguments(db, directive.arguments());
    let directive_data = DirectiveData { name, arguments };
    db.intern_directive(directive_data)
}

fn arguments(
    db: &dyn SourceDatabase,
    arguments: Option<ast::Arguments>,
) -> Option<Arc<Vec<Argument>>> {
    match arguments {
        Some(arguments) => {
            let arguments = arguments
                .arguments()
                .into_iter()
                .map(|arg| argument(db, arg))
                .collect();
            Some(Arc::new(arguments))
        }
        None => None,
    }
}

fn argument(db: &dyn SourceDatabase, argument: ast::Argument) -> Argument {
    let name = argument
        .name()
        .expect("Argument must have a name")
        .to_string();
    let argument_data = ArgumentData { name };
    db.intern_argument(argument_data)
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
            let selection_data = SelectionData::Field(field);
            db.intern_selection(selection_data)
        }
        ast::Selection::FragmentSpread(_) => unimplemented!(),
        ast::Selection::InlineFragment(_) => unimplemented!(),
    }
}

fn field(db: &dyn SourceDatabase, field: ast::Field) -> Arc<Field> {
    let name = field.name().expect("Field must have a name").to_string();
    let alias = alias(db, field.alias());
    let selection_set = field
        .selection_set()
        .map(|sel_set| selection_set(db, sel_set.selections()));
    let directives = directives(db, field.directives());
    let arguments = arguments(db, field.arguments());

    let field_data = FieldData {
        name,
        alias,
        selection_set,
        directives,
        arguments,
    };
    Arc::new(db.intern_field(field_data))
}

fn alias(db: &dyn SourceDatabase, alias: Option<ast::Alias>) -> Option<Arc<Alias>> {
    alias.map(|alias| {
        let name = alias.name().expect("Alias must have a name").to_string();
        let alias_data = AliasData(name);
        Arc::new(db.intern_alias(alias_data))
    })
}
