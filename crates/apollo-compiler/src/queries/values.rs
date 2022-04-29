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

    pub fn find(&self, name: &str) -> Option<FragmentDefinition> {
        self.inner.iter().find_map(|op| {
            if op.name() == name {
                Some(op.clone())
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
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

// NOTE @lrlna: all the getter methods here return the exact types that are
// stored in salsa's DB, Arc<>'s and all. In the long run, this should return
// the underlying values, as what's important is that the values are Arc<>'d in
// the database.
impl FragmentDefinition {
    /// Get the fragment definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the fragment definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to fragment definition's type condition.
    pub fn type_condition(&self) -> &str {
        self.type_condition.as_ref()
    }

    /// Get fragment definition's directives.
    /// TODO: is this good??
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to fragment definition's selection set.
    /// TODO: is this good??
    pub fn selection_set(&self) -> &[Selection] {
        self.selection_set.as_ref()
    }

    // NOTE @lrlna: we will need to think and implement scope for fragment
    // definitions used/defined variables, as defined variables change based on
    // which operation definition the fragment is used in.

    /// Get variables used in a fragment definition.
    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        self.selection_set
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect()
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
    pub fn find(&self, name: &str) -> Option<OperationDefinition> {
        self.inner.iter().find_map(|op| {
            if let Some(n) = op.name() {
                if n == name {
                    return Some(op.clone());
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
    pub(crate) variables: Arc<Vec<VariableDefinition>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl OperationDefinition {
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the operation definition's ty.
    pub fn ty(&self) -> &OperationType {
        &self.ty
    }

    /// Get a mutable reference to the operation definition's name.
    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// Get a reference to the operation definition's variables.
    pub fn variables(&self) -> &[VariableDefinition] {
        self.variables.as_ref()
    }

    /// Get a mutable reference to the operation definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to the operation definition's selection set.
    pub fn selection_set(&self) -> &[Selection] {
        self.selection_set.as_ref()
    }

    /// Get fields in the operation definition.
    pub fn fields(&self, db: &dyn SourceDatabase) -> Arc<Vec<Field>> {
        db.operation_fields(self.id)
    }

    // NOTE @lrlna: this is quite messy. it should live under the
    // inline_fragment/fragment_spread impls, i.e. op.fragment_spread().fields(),
    // op.inline_fragments().fields()
    //
    // We will need to figure out how to store operation definition id on its
    // fragment spreads and inline fragments to do this
    pub fn fields_in_inline_fragments(&self, db: &dyn SourceDatabase) -> Arc<Vec<Field>> {
        db.operation_inline_fragment_fields(self.id)
    }

    pub fn fields_in_fragment_spread(&self, db: &dyn SourceDatabase) -> Arc<Vec<Field>> {
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
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) default_value: Option<Value>,
    pub(crate) directives: Arc<Vec<Directive>>,
}

impl VariableDefinition {
    /// Get a mutable reference to the variable definition's name.
    pub fn name(&self) -> String {
        self.name.clone()
    }

    /// Get a reference to the variable definition's ty.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a reference to the variable definition's default value.
    pub fn default_value(&self) -> Option<&Value> {
        self.default_value.as_ref()
    }

    /// Get a reference to the variable definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    NonNull { ty: Box<Type> },
    List { ty: Box<Type> },
    Named { name: String },
}

impl Type {
    /// Returns `true` if the type is [`NonNull`].
    ///
    /// [`NonNull`]: Type::NonNull
    #[must_use]
    pub fn is_non_null(&self) -> bool {
        matches!(self, Self::NonNull { .. })
    }

    /// Returns `true` if the type is [`Named`].
    ///
    /// [`Named`]: Type::Named
    #[must_use]
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }

    /// Returns `true` if the type is [`List`].
    ///
    /// [`List`]: Type::List
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List { .. })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub(crate) name: String,
    pub(crate) arguments: Arc<Vec<Argument>>,
}

impl Directive {
    /// Get a reference to the directive's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the directive's arguments.
    pub fn arguments(&self) -> &[Argument] {
        self.arguments.as_ref()
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: Value,
}

impl Argument {
    /// Get a reference to the argument's value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

pub type Variable = String;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    Variable(Variable),
    Int(i32),
    Float(Float),
    String(String),
    Boolean(bool),
    Null,
    Enum(String),
    List(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    /// Returns `true` if the value is [`Variable`].
    ///
    /// [`Variable`]: Value::Variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable(..))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Selection {
    Field(Arc<Field>),
    FragmentSpread(Arc<FragmentSpread>),
    InlineFragment(Arc<InlineFragment>),
}
impl Selection {
    /// Get variables used in the selection set.
    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        match self {
            Selection::Field(field) => field.variables(db),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.variables(db),
            Selection::InlineFragment(inline_fragment) => inline_fragment.variables(db),
        }
    }

    /// Returns `true` if the selection is [`Field`].
    ///
    /// [`Field`]: Selection::Field
    #[must_use]
    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(..))
    }

    /// Returns `true` if the selection is [`FragmentSpread`].
    ///
    /// [`FragmentSpread`]: Selection::FragmentSpread
    #[must_use]
    pub fn is_fragment_spread(&self) -> bool {
        matches!(self, Self::FragmentSpread(..))
    }

    /// Returns `true` if the selection is [`InlineFragment`].
    ///
    /// [`InlineFragment`]: Selection::InlineFragment
    #[must_use]
    pub fn is_inline_fragment(&self) -> bool {
        matches!(self, Self::InlineFragment(..))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub(crate) alias: Option<Arc<Alias>>,
    pub(crate) name: String,
    pub(crate) arguments: Arc<Vec<Argument>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl Field {
    /// Get a reference to the field's alias.
    pub fn alias(&self) -> Option<&Alias> {
        match &self.alias {
            Some(alias) => Some(alias.as_ref()),
            None => None,
        }
    }

    /// Get a reference to the field's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
    /// Get a reference to the field's arguments.
    pub fn arguments(&self) -> &[Argument] {
        self.arguments.as_ref()
    }

    /// Get a reference to the field's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to the field's selection set.
    pub fn selection_set(&self) -> &[Selection] {
        self.selection_set.as_ref()
    }

    /// Get variables used in the field.
    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let mut vars: Vec<_> = self
            .arguments
            .iter()
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(String::from(var)),
                _ => None,
            })
            .collect();
        let iter = self.selection_set.iter().flat_map(|sel| sel.variables(db));
        vars.extend(iter);
        vars
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: Arc<Vec<Selection>>,
}

impl InlineFragment {
    /// Get a reference to inline fragment's type condition.
    pub fn type_condition(&self) -> Option<&String> {
        self.type_condition.as_ref()
    }

    /// Get a reference to inline fragment's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference inline fragment's selection set.
    pub fn selection_set(&self) -> &[Selection] {
        self.selection_set.as_ref()
    }

    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let vars = self
            .selection_set
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect();
        vars
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub name: String,
    pub directives: Arc<Vec<Directive>>,
    // NOTE @lrlna: this should just be Uuid.  If we can't find the framgment we
    // are looking for when populating this field, we should throw a semantic
    // error.
    pub fragment_id: Option<Uuid>,
}

impl FragmentSpread {
    pub fn fragment(&self, db: &dyn SourceDatabase) -> Option<Arc<FragmentDefinition>> {
        db.find_fragment(self.fragment_id?)
    }

    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let vars = match self.fragment(db) {
            Some(fragment) => fragment
                .selection_set
                .iter()
                .flat_map(|sel| sel.variables(db))
                .collect(),
            None => Vec::new(),
        };
        vars
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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SchemaDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootOperationTypeDefinition {
    pub(crate) operation_type: OperationType,
    pub(crate) named_type: Type,
}
