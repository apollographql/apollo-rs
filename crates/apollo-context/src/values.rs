use std::sync::Arc;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DocumentData {
    pub definitions: Arc<Vec<Definition>>,
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinitionData {
    pub ty: OperationType,
    pub name: String, // TODO @lrlna: Option<String>
    pub variables: Option<Arc<Vec<VariableDefinition>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Arc<Vec<Selection>>,
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDefinitionData {
    pub name: String,
    // ty: Type_,
    // default_value: Option<Arc<Value>>,
    pub directives: Option<Arc<Vec<Directive>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDefinition(salsa::InternId);

impl salsa::InternKey for VariableDefinition {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DirectiveData {
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive(salsa::InternId);

impl salsa::InternKey for Directive {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArgumentData {
    pub name: String,
    // pub value: Arc<Value>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument(salsa::InternId);

impl salsa::InternKey for Argument {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum SelectionData {
    Field(Arc<Field>),
    // FragmentSpread(Arc<FragmentSpread>),
    // InlineFragment(Arc<InlineFragment>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Selection(salsa::InternId);

impl salsa::InternKey for Selection {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldData {
    pub alias: Option<Arc<Alias>>,
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Option<Arc<Vec<Selection>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field(salsa::InternId);

impl salsa::InternKey for Field {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AliasData(pub String);
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(salsa::InternId);

impl salsa::InternKey for Alias {
    fn from_intern_id(id: salsa::InternId) -> Self {
        Self(id)
    }

    fn as_intern_id(&self) -> salsa::InternId {
        self.0
    }
}
