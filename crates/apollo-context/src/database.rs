use std::sync::Arc;

use apollo_parser::{ast, Parser};

#[salsa::database(InternerDatabase, ASTDatabase)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Database>,
}

impl salsa::Database for Database {}

#[salsa::query_group(ASTDatabase)]
pub trait SourceDatabase: Interner {
    #[salsa::invoke(parse_query)]
    fn parse(&self) -> ast::Document;

    #[salsa::input]
    fn input_string(&self, key: ()) -> Arc<String>;

    fn document(&self) -> Arc<Document>;

    fn definitions(&self, doc: ast::Document) -> Arc<Vec<Definition>>;
}

fn parse_query(db: &dyn SourceDatabase) -> ast::Document {
    let input = db.input_string(());

    let parser = Parser::new(&input);
    parser.parse().document()
}

// this is top level entry to the source db
fn document(db: &dyn SourceDatabase) -> Arc<Document> {
    let document = db.parse();
    let definitions = db.definitions(document);
    let document_data = DocumentData { definitions };

    Arc::new(db.intern_document(document_data))
}

fn definitions(db: &dyn SourceDatabase, document: ast::Document) -> Arc<Vec<Definition>> {
    let definitions: Vec<_> = document
        .definitions()
        .map(|def| match def {
            ast::Definition::OperationDefinition(def) => {
                let op_def = operation_definition(db, def);
                let definition_data = DefinitionData::OperationDefinition(op_def);
                db.intern_definition(definition_data)
            }
            _ => todo!(),
        })
        .collect();

    Arc::new(definitions)
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

    let ty: OperationType = match op_def.operation_type() {
        Some(op_type) => {
            if op_type.query_token().is_some() {
                OperationType::Query
            } else if op_type.mutation_token().is_some() {
                OperationType::Mutation
            } else if op_type.subscription_token().is_some() {
                OperationType::Subscription
            } else {
                OperationType::Query
            }
        }
        None => OperationType::Query,
    };

    let operation_def_data = OperationDefinitionData { ty, name };
    let operation_def = db.intern_operation_definition(operation_def_data);

    Arc::new(operation_def)
}

#[salsa::query_group(InternerDatabase)]
pub trait Interner {
    #[salsa::interned]
    fn intern_document(&self, document: DocumentData) -> Document;
    #[salsa::interned]
    fn intern_definition(&self, definition: DefinitionData) -> Definition;
    #[salsa::interned]
    fn intern_operation_definition(
        &self,
        operation: OperationDefinitionData,
    ) -> OperationDefinition;
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DocumentData {
    definitions: Arc<Vec<Definition>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Document(salsa::InternId);

impl salsa::InternKey for Document {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DefinitionData {
    OperationDefinition(Arc<OperationDefinition>),
    FragmentDefinition,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Definition(salsa::InternId);

impl salsa::InternKey for Definition {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

// NOTE @lrlna: is there a way to query this straight up from definitions so we
// don't have to keep a separate struct?
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Operations(Arc<Vec<OperationDefinitionData>>);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinitionData {
    ty: OperationType,
    name: String,
    // variables: Arc<Vec<VariableDefinition>>,
    // directives: Arc<Vec<Directive>>,
    // selection_set: Arc<Vec<Selection>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition(salsa::InternId);

impl salsa::InternKey for OperationDefinition {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}
