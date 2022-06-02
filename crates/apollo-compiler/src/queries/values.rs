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

impl std::fmt::Display for OperationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OperationType::Query => write!(f, "Subscription"),
            OperationType::Mutation => write!(f, "Mutation"),
            OperationType::Subscription => write!(f, "Query"),
        }
    }
}

impl From<OperationType> for String {
    fn from(op_type: OperationType) -> Self {
        if op_type.is_subscription() {
            "Subscription".to_string()
        } else if op_type.is_mutation() {
            "Mutation".to_string()
        } else {
            "Query".to_string()
        }
    }
}

impl<'a> From<&'a str> for OperationType {
    fn from(op_type: &str) -> Self {
        if op_type == "Query" {
            OperationType::Query
        } else if op_type == "Mutation" {
            OperationType::Mutation
        } else {
            OperationType::Subscription
        }
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

    pub fn name(&self) -> String {
        match self {
            Type::NonNull { ty } => get_name(*ty.clone()),
            Type::List { ty } => get_name(*ty.clone()),
            Type::Named { name } => name.to_owned(),
        }
    }
}

fn get_name(ty: Type) -> String {
    match ty {
        Type::NonNull { ty } => match *ty {
            Type::NonNull { ty } => get_name(*ty),
            Type::List { ty } => get_name(*ty),
            Type::Named { name } => name,
        },
        Type::List { ty } => match *ty {
            Type::NonNull { ty } => get_name(*ty),
            Type::List { ty } => get_name(*ty),
            Type::Named { name } => name,
        },
        Type::Named { name } => name,
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

pub type DefaultValue = Value;

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

#[derive(Clone, Debug, Hash, PartialEq, Default, Eq)]
pub struct SchemaDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
}

impl SchemaDefinition {
    /// Get a reference to the schema definition's root operation type definition.
    pub fn root_operation_type_definition(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Set the schema definition's root operation type definition.
    pub(crate) fn set_root_operation_type_definition(&mut self, op: RootOperationTypeDefinition) {
        Arc::get_mut(&mut self.root_operation_type_definition)
            .unwrap()
            .push(op)
    }
    // NOTE(@lrlna): potentially have the following fns on the database itself
    // so they are memoised as well

    /// Get Schema's query object type definition.
    pub fn query(&self, db: &dyn SourceDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_query() {
                db.find_object_type(op.object_type_id?)
            } else {
                None
            }
        })
    }

    /// Get Schema's mutation object type definition.
    pub fn mutation(&self, db: &dyn SourceDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_mutation() {
                db.find_object_type(op.object_type_id?)
            } else {
                None
            }
        })
    }

    /// Get Schema's subscription object type definition.
    pub fn subscription(&self, db: &dyn SourceDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_subscription() {
                db.find_object_type(op.object_type_id?)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootOperationTypeDefinition {
    pub(crate) object_type_id: Option<Uuid>,
    pub(crate) operation_type: OperationType,
    pub(crate) named_type: Type,
}

impl RootOperationTypeDefinition {
    /// Get a reference to the root operation type definition's named type.
    pub fn named_type(&self) -> &Type {
        &self.named_type
    }

    /// Get the root operation type definition's operation type.
    pub fn operation_type(&self) -> OperationType {
        self.operation_type
    }
}

impl Default for RootOperationTypeDefinition {
    fn default() -> Self {
        Self {
            object_type_id: None,
            operation_type: OperationType::Query,
            named_type: Type::Named {
                name: "Query".to_string(),
            },
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ObjectTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
}

impl ObjectTypeDefinition {
    /// Get the object type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the object type definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the object type definition's field definitions.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Get a reference to object type definition's implements interfaces vector.
    pub fn implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImplementsInterface {
    pub(crate) interface: String,
}

impl ImplementsInterface {
    pub fn interface_definition(
        &self,
        db: &dyn SourceDatabase,
    ) -> Option<Arc<InterfaceDefinition>> {
        db.find_interface_by_name(self.interface.clone())
    }

    pub fn interface(&self) -> &str {
        self.interface.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) ty: Type,
    pub(crate) directives: Arc<Vec<Directive>>,
}

impl FieldDefinition {
    /// Get a reference to the field definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArgumentsDefinition {
    pub(crate) input_values: Arc<Vec<InputValueDefinition>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) default_value: Option<DefaultValue>,
    pub(crate) directives: Arc<Vec<Directive>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
}

impl ScalarDefinition {
    /// Get a reference to the scalar definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to scalar definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) enum_values_definition: Arc<Vec<EnumValueDefinition>>,
}

impl EnumDefinition {
    /// Get a reference to the enum definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to enum definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to enum definition's enum values definition vector.
    pub fn enum_values_definition(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) enum_value: String,
    pub(crate) directives: Arc<Vec<Directive>>,
}

impl EnumValueDefinition {
    /// Get a reference to enum value definition's enum value
    pub fn enum_value(&self) -> &str {
        self.enum_value.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) union_members: Arc<Vec<UnionMember>>,
}

impl UnionDefinition {
    /// Get a reference to the union definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to union definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to union definition's union members.
    pub fn union_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionMember {
    pub(crate) name: String,
    pub(crate) object_id: Option<Uuid>,
}

impl UnionMember {
    /// Get a reference to the union member's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn object(&self, db: &dyn SourceDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type(self.object_id?)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InterfaceDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
}

impl InterfaceDefinition {
    /// Get the interface definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the interface definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to interface definition's implements interfaces vector.
    pub fn implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    /// Get a reference to interface definition's fields.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }
}
