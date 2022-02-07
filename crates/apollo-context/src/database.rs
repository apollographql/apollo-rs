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

    fn all_definitions(&self, doc: ast::Document) -> Arc<Vec<Definition>>;
}

fn parse_query(db: &dyn SourceDatabase) -> ast::Document {
    let input = db.input_string(());

    let parser = Parser::new(&input);
    parser.parse().document()
}

fn document(db: &dyn SourceDatabase) -> Arc<Document> {
    let document = db.parse();
    let definitions = db.all_definitions(document);
    let document_data = DocumentData { definitions };

    Arc::new(db.intern_document(document_data))
}

fn all_definitions(db: &dyn SourceDatabase, document: ast::Document) -> Arc<Vec<Definition>> {
    let definitions: Vec<_> = document
        .definitions()
        .map(|def| match def {
            ast::Definition::OperationDefinition(def) => {
                let name = def.name().expect("not optional").text().to_string();
                let definition_data = DefinitionData { name };
                db.intern_definition(definition_data)
            }
            ast::Definition::FragmentDefinition(def) => {
                let name = def
                    .fragment_name()
                    .expect("not optional")
                    .name()
                    .expect("not optional")
                    .text()
                    .to_string();
                let definition_data = DefinitionData { name };
                db.intern_definition(definition_data)
            }
            _ => todo!(),
        })
        .collect();

    Arc::new(definitions)
}

#[salsa::query_group(InternerDatabase)]
pub trait Interner {
    #[salsa::interned]
    fn intern_definition(&self, definition: DefinitionData) -> Definition;
    #[salsa::interned]
    fn intern_document(&self, document: DocumentData) -> Document;
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
    OperationDefinition(OperationDefinition),
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

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinitionData {
    ty: OperationType,
    name: Option<String>,
    // variables: Arc<Vec<VariableDefinition>>,
    // directives: Arc<Vec<Directive>>,
    // selection_set: Arc<Vec<Selection>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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
pub enum OperationTypeData {
    Query,
    Mutation,
    Subscription,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationType(salsa::InternId);

impl salsa::InternKey for OperationType {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}
