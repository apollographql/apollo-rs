use std::{ops::Deref, sync::Arc};

use ordered_float::{self, OrderedFloat};
use uuid::Uuid;

use crate::SourceDatabase;

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
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) type_condition: String,
    pub(crate) directives: Option<Arc<Vec<Directive>>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

// NOTE @lrlna: all the getter methods here return the exact types that are
// stored in salsa's DB, Arc<>'s and all. In the long run, this should return
// the underlying values, as what's important is that the values are Arc<>'d in
// the database.
impl FragmentDefinition {
    // NOTE @lrlna: can this just be a getter for a reference? what are the
    // repercussions of this?
    /// Get fragment definition's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get fragment definition's id.
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get fragment definition's type condition.
    pub fn type_condition(&self) -> String {
        self.type_condition.clone()
    }

    /// Get fragment definition's directives.
    pub fn directives(&self) -> Option<Arc<Vec<Directive>>> {
        self.directives.clone()
    }

    /// Get fragment definition's selection set.
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

    // NOTE: this should only be a wrapper around a find_operation method on the
    // SourceDatabase so this function is also memoized.  How do we get access
    // to SourceDatabase from this struct impl gracefully here?
    pub fn find(&self, name: &str) -> Option<Arc<OperationDefinition>> {
        self.inner.iter().find_map(|op| {
            if let Some(n) = op.name() {
                if n == name {
                    return Some(Arc::new(op.clone()));
                }
            }
            None
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
    pub(crate) id: Uuid,
    pub(crate) ty: OperationType,
    pub(crate) name: Option<String>,
    pub(crate) variables: Option<Arc<Vec<VariableDefinition>>>,
    pub(crate) directives: Option<Arc<Vec<Directive>>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl OperationDefinition {
    /// Get a mutable reference to the operation definition's variables.
    pub fn variables(&self) -> Option<Arc<Vec<VariableDefinition>>> {
        self.variables.clone()
    }
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get a mutable reference to the operation definition's directives.
    pub fn directives(&self) -> Option<Arc<Vec<Directive>>> {
        self.directives.clone()
    }

    /// Get a mutable reference to the operation definition's name.
    pub fn name(&self) -> Option<String> {
        self.name.clone()
    }

    /// Get a reference to the operation definition's ty.
    pub fn ty(&self) -> OperationType {
        self.ty
    }

    pub fn selection_set(&self) -> Arc<Vec<Selection>> {
        self.selection_set.clone()
    }

    pub fn fields(&self, db: &dyn SourceDatabase) -> Option<Arc<Vec<Field>>> {
        db.operation_fields(self.id)
    }

    // NOTE @lrlna: this is quite messy. it should live under the
    // inline_fragment/fragment_spread impls, i.e. op.fragment_spread().fields(),
    // op.inline_fragments().fields()
    //
    // We will need to figure out how to store operation definition id on its
    // fragment spreads and inline fragments to do this
    pub fn fields_in_inline_fragments(&self, db: &dyn SourceDatabase) -> Option<Arc<Vec<Field>>> {
        db.operation_inline_fragment_fields(self.id)
    }

    pub fn fields_in_fragment_spread(&self, db: &dyn SourceDatabase) -> Option<Arc<Vec<Field>>> {
        db.operation_fragment_spread_fields(self.id)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    /// Returns `true` if the operation type is [`Query`].
    ///
    /// [`Query`]: OperationType::Query
    #[must_use]
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Returns `true` if the operation type is [`Mutation`].
    ///
    /// [`Mutation`]: OperationType::Mutation
    #[must_use]
    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation)
    }

    /// Returns `true` if the operation type is [`Subscription`].
    ///
    /// [`Subscription`]: OperationType::Subscription
    #[must_use]
    pub fn is_subscription(&self) -> bool {
        matches!(self, Self::Subscription)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDefinition {
    pub name: String,
    pub ty: Type,
    pub default_value: Option<Value>,
    pub directives: Option<Arc<Vec<Directive>>>,
}

impl VariableDefinition {
    /// Get a mutable reference to the variable definition's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    NonNull { ty: Box<Type> },
    List { ty: Box<Type> },
    Named { name: String },
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument {
    pub name: String,
    pub value: Value,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    Variable(String),
    Int(i32),
    Float(Float),
    String(String),
    Boolean(bool),
    Null,
    Enum(String),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Selection {
    Field(Arc<Field>),
    FragmentSpread(Arc<FragmentSpread>),
    InlineFragment(Arc<InlineFragment>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub alias: Option<Arc<Alias>>,
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Option<Arc<Vec<Selection>>>,
}

impl Field {
    /// Get a reference to the field's arguments.
    pub fn arguments(&self) -> Option<Arc<Vec<Argument>>> {
        self.arguments.clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InlineFragment {
    pub type_condition: Option<String>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Arc<Vec<Selection>>,
}

impl InlineFragment {
    /// Get inline fragment's type condition.
    pub fn type_condition(&self) -> Option<String> {
        self.type_condition.clone()
    }

    /// Get inline fragment's directives.
    pub fn directives(&self) -> Option<Arc<Vec<Directive>>> {
        self.directives.clone()
    }

    /// Get inline fragment's selection set.
    pub fn selection_set(&self) -> Arc<Vec<Selection>> {
        self.selection_set.clone()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub name: String,
    pub directives: Option<Arc<Vec<Directive>>>,
    // NOTE @lrlna: this should just be Uuid.  If we can't find the framgment we
    // are looking for when populating this field, we should throw a semantic
    // error.
    pub fragment_id: Option<Uuid>,
}

impl FragmentSpread {
    /// Get the fragment spread's fragment id.
    pub fn fragment_id(&self) -> Option<Uuid> {
        self.fragment_id
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Float {
    inner: ordered_float::OrderedFloat<f64>,
}

impl Float {
    pub fn new(float: f64) -> Self {
        Self {
            inner: OrderedFloat(float),
        }
    }
}
