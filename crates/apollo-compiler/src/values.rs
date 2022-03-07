use std::{ops::Deref, sync::Arc};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub data: String,
    pub index: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Fragments {
    inner: Arc<Vec<FragmentDefinition>>,
}

impl Fragments {
    pub fn new(inner: Arc<Vec<FragmentDefinition>>) -> Self {
        Self { inner }
    }

    pub fn find(&self, name: &str) -> Option<Arc<FragmentDefinition>> {
        self.inner.iter().find_map(|op| {
            if op.name() == name {
                Some(Arc::new(op.clone()))
            } else {
                None
            }
        })
    }
}
impl Deref for Fragments {
    type Target = Arc<Vec<FragmentDefinition>>;

    fn deref(&self) -> &Arc<Vec<FragmentDefinition>> {
        &self.inner
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentDefinition {
    pub(crate) name: String,
    pub(crate) type_condition: String,
    pub(crate) directives: Option<Arc<Vec<Directive>>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl FragmentDefinition {
    /// Get a reference to the fragment definition's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get a reference to the fragment definition's type condition.
    pub fn type_condition(&self) -> String {
        self.type_condition.clone()
    }

    /// Get a reference to the fragment definition's directives.
    pub fn directives(&self) -> Option<Arc<Vec<Directive>>> {
        self.directives.clone()
    }

    /// Get a reference to the fragment definition's selection set.
    pub fn selection_set(&self) -> Arc<Vec<Selection>> {
        self.selection_set.clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Operations {
    inner: Arc<Vec<OperationDefinition>>,
}

impl Operations {
    pub fn new(inner: Arc<Vec<OperationDefinition>>) -> Self {
        Self { inner }
    }

    pub fn find(&self, name: &str) -> Option<Arc<OperationDefinition>> {
        self.inner.iter().find_map(|op| {
            if op.name() == name {
                Some(Arc::new(op.clone()))
            } else {
                None
            }
        })
    }
}

impl Deref for Operations {
    type Target = Arc<Vec<OperationDefinition>>;

    fn deref(&self) -> &Arc<Vec<OperationDefinition>> {
        &self.inner
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub(crate) ty: OperationType,
    pub(crate) name: String, // TODO @lrlna: Option<String>
    pub(crate) variables: Option<Arc<Vec<VariableDefinition>>>,
    pub(crate) directives: Option<Arc<Vec<Directive>>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl OperationDefinition {
    /// Get a mutable reference to the operation definition's variables.
    pub fn variables(&self) -> Option<Arc<Vec<VariableDefinition>>> {
        self.variables.clone()
    }

    /// Get a mutable reference to the operation definition's directives.
    pub fn directives(&self) -> Option<Arc<Vec<Directive>>> {
        self.directives.clone()
    }

    /// Get a mutable reference to the operation definition's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get a reference to the operation definition's ty.
    pub fn ty(&self) -> OperationType {
        self.ty.clone() // ?? should we clone?
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDefinition {
    pub name: String,
    // ty: Type_,
    // default_value: Option<Arc<Value>>,
    pub directives: Option<Arc<Vec<Directive>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument {
    pub name: String,
    // pub value: Arc<Value>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Selection {
    Field(Arc<Field>),
    // FragmentSpread(Arc<FragmentSpread>),
    // InlineFragment(Arc<InlineFragment>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub alias: Option<Arc<Alias>>,
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Option<Arc<Vec<Selection>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);
