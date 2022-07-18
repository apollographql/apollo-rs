use std::{ops::Deref, sync::Arc};

use apollo_parser::{
    ast::{self, AstNode, SyntaxNodePtr},
    SyntaxNode,
};
use ordered_float::{self, OrderedFloat};
use uuid::Uuid;

use crate::SourceDatabase;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    FragmentDefinition(FragmentDefinition),
    DirectiveDefinition(DirectiveDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    ObjectTypeDefinition(ObjectTypeDefinition),
    InterfaceTypeDefinition(InterfaceTypeDefinition),
    UnionTypeDefinition(UnionTypeDefinition),
    EnumTypeDefinition(EnumTypeDefinition),
    InputObjectTypeDefinition(InputObjectTypeDefinition),
    SchemaDefinition(SchemaDefinition),
}

impl Definition {
    // Get a reference to definition's name.
    pub fn name(&self) -> Option<&str> {
        match self {
            Definition::OperationDefinition(def) => def.name(),
            Definition::FragmentDefinition(def) => Some(def.name()),
            Definition::DirectiveDefinition(def) => Some(def.name()),
            Definition::ScalarTypeDefinition(def) => Some(def.name()),
            Definition::ObjectTypeDefinition(def) => Some(def.name()),
            Definition::InterfaceTypeDefinition(def) => Some(def.name()),
            Definition::UnionTypeDefinition(def) => Some(def.name()),
            Definition::EnumTypeDefinition(def) => Some(def.name()),
            Definition::InputObjectTypeDefinition(def) => Some(def.name()),
            Definition::SchemaDefinition(_) => None,
        }
    }

    // Get the current definition type, e..g OperationDefinition,
    // EnumTypeDefinition, ObjectTypeDefinition etc.
    pub fn ty(&self) -> String {
        match self {
            Definition::OperationDefinition(_) => "OperationDefinition".to_string(),
            Definition::FragmentDefinition(_) => "FragmentDefinition".to_string(),
            Definition::DirectiveDefinition(_) => "DirectiveDefinition".to_string(),
            Definition::ScalarTypeDefinition(_) => "ScalarTypeDefinition".to_string(),
            Definition::ObjectTypeDefinition(_) => "ObjectTypeDefinition".to_string(),
            Definition::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition".to_string(),
            Definition::UnionTypeDefinition(_) => "UnionTypeDefinition".to_string(),
            Definition::EnumTypeDefinition(_) => "EnumTypeDefinition".to_string(),
            Definition::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition".to_string(),
            Definition::SchemaDefinition(_) => "SchemaDefinition".to_string(),
        }
    }

    pub fn id(&self) -> Option<&Uuid> {
        match self {
            Definition::OperationDefinition(def) => Some(def.id()),
            Definition::FragmentDefinition(def) => Some(def.id()),
            Definition::DirectiveDefinition(def) => Some(def.id()),
            Definition::ScalarTypeDefinition(def) => Some(def.id()),
            Definition::ObjectTypeDefinition(def) => Some(def.id()),
            Definition::InterfaceTypeDefinition(def) => Some(def.id()),
            Definition::UnionTypeDefinition(def) => Some(def.id()),
            Definition::EnumTypeDefinition(def) => Some(def.id()),
            Definition::InputObjectTypeDefinition(def) => Some(def.id()),
            Definition::SchemaDefinition(_) => None,
        }
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        match self {
            Definition::ObjectTypeDefinition(def) => def.field(name),
            Definition::InterfaceTypeDefinition(def) => def.field(name),
            _ => None,
        }
    }

    /// Returns `true` if the definition is either a [`ScalarTypeDefinition`],
    /// [`ObjectTypeDefinition`], [`InterfaceTypeDefinition`],
    /// [`UnionTypeDefinition`], [`EnumTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`ObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    /// [`InterfaceTypeDefinition`]: Definition::InterfaceTypeDefinition
    /// [`UnionTypeDefinition`]: Definition::UnionTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    #[must_use]
    pub fn is_output_definition(&self) -> bool {
        matches!(
            self,
            Self::ScalarTypeDefinition(..)
                | Self::ObjectTypeDefinition(..)
                | Self::InterfaceTypeDefinition(..)
                | Self::UnionTypeDefinition(..)
                | Self::EnumTypeDefinition(..)
        )
    }

    /// Returns `true` if the definition is either a [`ScalarTypeDefinition`],
    /// [`EnumTypeDefinition`], [`InputObjectTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    /// [`InputObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    #[must_use]
    pub fn is_input_definition(&self) -> bool {
        matches!(
            self,
            Self::ScalarTypeDefinition(..)
                | Self::EnumTypeDefinition(..)
                | Self::InputObjectTypeDefinition(..)
        )
    }

    /// Returns `true` if the definition is [`OperationDefinition`].
    ///
    /// [`OperationDefinition`]: Definition::OperationDefinition
    #[must_use]
    pub fn is_operation_definition(&self) -> bool {
        matches!(self, Self::OperationDefinition(..))
    }

    /// Returns `true` if the definition is [`FragmentDefinition`].
    ///
    /// [`FragmentDefinition`]: Definition::FragmentDefinition
    #[must_use]
    pub fn is_fragment_definition(&self) -> bool {
        matches!(self, Self::FragmentDefinition(..))
    }

    /// Returns `true` if the definition is [`DirectiveDefinition`].
    ///
    /// [`DirectiveDefinition`]: Definition::DirectiveDefinition
    #[must_use]
    pub fn is_directive_definition(&self) -> bool {
        matches!(self, Self::DirectiveDefinition(..))
    }

    /// Returns `true` if the definition is [`ScalarTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    #[must_use]
    pub fn is_scalar_type_definition(&self) -> bool {
        matches!(self, Self::ScalarTypeDefinition(..))
    }

    /// Returns `true` if the definition is [`ObjectTypeDefinition`].
    ///
    /// [`ObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    #[must_use]
    pub fn is_object_type_definition(&self) -> bool {
        matches!(self, Self::ObjectTypeDefinition { .. })
    }

    /// Returns `true` if the definition is [`InterfaceTypeDefinition`].
    ///
    /// [`InterfaceTypeDefinition`]: Definition::InterfaceTypeDefinition
    #[must_use]
    pub fn is_interface_type_definition(&self) -> bool {
        matches!(self, Self::InterfaceTypeDefinition(..))
    }

    /// Returns `true` if the definition is [`UnionTypeDefinition`].
    ///
    /// [`UnionTypeDefinition`]: Definition::UnionTypeDefinition
    #[must_use]
    pub fn is_union_type_definition(&self) -> bool {
        matches!(self, Self::UnionTypeDefinition(..))
    }

    /// Returns `true` if the definition is [`EnumTypeDefinition`].
    ///
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    #[must_use]
    pub fn is_enum_type_definition(&self) -> bool {
        matches!(self, Self::EnumTypeDefinition(..))
    }

    /// Returns `true` if the definition is [`InputObjectTypeDefinition`].
    ///
    /// [`InputObjectTypeDefinition`]: Definition::InputObjectTypeDefinition
    #[must_use]
    pub fn is_input_object_type_definition(&self) -> bool {
        matches!(self, Self::InputObjectTypeDefinition(..))
    }

    /// Returns `true` if the definition is [`SchemaDefinition`].
    ///
    /// [`SchemaDefinition`]: Definition::SchemaDefinition
    #[must_use]
    pub fn is_schema_definition(&self) -> bool {
        matches!(self, Self::SchemaDefinition(..))
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentDefinition {
    pub(crate) id: Uuid,
    pub(crate) name: String,
    pub(crate) type_condition: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) ast_ptr: SyntaxNodePtr,
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
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    // NOTE @lrlna: we will need to think and implement scope for fragment
    // definitions used/defined variables, as defined variables change based on
    // which operation definition the fragment is used in.

    /// Get variables used in a fragment definition.
    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        self.selection_set
            .selection()
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub(crate) id: Uuid,
    pub(crate) ty: OperationType,
    pub(crate) name: Option<String>,
    pub(crate) variables: Arc<Vec<VariableDefinition>>,
    pub(crate) object_id: Option<Uuid>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) ast_ptr: SyntaxNodePtr,
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
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
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
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
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

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
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
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl VariableDefinition {
    /// Get a reference to the variable definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
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

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    NonNull {
        ty: Box<Type>,
        ast_ptr: Option<SyntaxNodePtr>,
    },
    List {
        ty: Box<Type>,
        ast_ptr: Option<SyntaxNodePtr>,
    },
    Named {
        name: String,
        ast_ptr: Option<SyntaxNodePtr>,
    },
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

    /// Returns `true` if Type is either a [`ScalarTypeDefinition`],
    /// [`ObjectTypeDefinition`], [`InterfaceTypeDefinition`],
    /// [`UnionTypeDefinition`], [`EnumTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`ObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    /// [`InterfaceTypeDefinition`]: Definition::InterfaceTypeDefinition
    /// [`UnionTypeDefinition`]: Definition::UnionTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    #[must_use]
    pub fn is_output_type(&self, db: &dyn SourceDatabase) -> bool {
        if let Some(ty) = self.ty(db) {
            ty.as_ref().is_output_definition()
        } else {
            false
        }
    }

    /// Returns `true` if the Type is either a [`ScalarTypeDefinition`],
    /// [`EnumTypeDefinition`], [`InputObjectTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    /// [`InputObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    #[must_use]
    pub fn is_input_type(&self, db: &dyn SourceDatabase) -> bool {
        if let Some(ty) = self.ty(db) {
            ty.as_ref().is_input_definition()
        } else {
            false
        }
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        match self {
            Type::NonNull { ty: _, ast_ptr } => ast_ptr.as_ref(),
            Type::List { ty: _, ast_ptr } => ast_ptr.as_ref(),
            Type::Named { name: _, ast_ptr } => ast_ptr.as_ref(),
        }
    }

    pub fn ty(&self, db: &dyn SourceDatabase) -> Option<Arc<Definition>> {
        db.find_definition_by_name(self.name())
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }

    pub fn name(&self) -> String {
        match self {
            Type::NonNull { ty, ast_ptr: _ } => get_name(*ty.clone()),
            Type::List { ty, ast_ptr: _ } => get_name(*ty.clone()),
            Type::Named { name, ast_ptr: _ } => name.to_owned(),
        }
    }
}

fn get_name(ty: Type) -> String {
    match ty {
        Type::NonNull { ty, ast_ptr: _ } => match *ty {
            Type::NonNull { ty, ast_ptr: _ } => get_name(*ty),
            Type::List { ty, ast_ptr: _ } => get_name(*ty),
            Type::Named { name, ast_ptr: _ } => name,
        },
        Type::List { ty, ast_ptr: _ } => match *ty {
            Type::NonNull { ty, ast_ptr: _ } => get_name(*ty),
            Type::List { ty, ast_ptr: _ } => get_name(*ty),
            Type::Named { name, ast_ptr: _ } => name,
        },
        Type::Named { name, ast_ptr: _ } => name,
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub(crate) name: String,
    pub(crate) arguments: Arc<Vec<Argument>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
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

    // Get directive definition of the currently used directive
    pub fn directive(&self, db: &dyn SourceDatabase) -> Option<Arc<DirectiveDefinition>> {
        db.find_directive_definition_by_name(self.name().to_string())
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DirectiveDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) repeatable: bool,
    pub(crate) directive_locations: Arc<Vec<DirectiveLocation>>,
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
}

impl DirectiveDefinition {
    /// Get a reference to the directive definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to the directive definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    // Get a reference to argument definition's locations.
    pub fn arguments(&self) -> &ArgumentsDefinition {
        &self.arguments
    }

    // Get a reference to directive definition's locations.
    pub fn directive_locations(&self) -> &[DirectiveLocation] {
        self.directive_locations.as_ref()
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

impl From<ast::DirectiveLocation> for DirectiveLocation {
    fn from(directive_loc: ast::DirectiveLocation) -> Self {
        if directive_loc.query_token().is_some() {
            DirectiveLocation::Query
        } else if directive_loc.mutation_token().is_some() {
            DirectiveLocation::Mutation
        } else if directive_loc.subscription_token().is_some() {
            DirectiveLocation::Subscription
        } else if directive_loc.field_token().is_some() {
            DirectiveLocation::Field
        } else if directive_loc.fragment_definition_token().is_some() {
            DirectiveLocation::FragmentDefinition
        } else if directive_loc.fragment_spread_token().is_some() {
            DirectiveLocation::FragmentSpread
        } else if directive_loc.inline_fragment_token().is_some() {
            DirectiveLocation::InlineFragment
        } else if directive_loc.variable_definition_token().is_some() {
            DirectiveLocation::VariableDefinition
        } else if directive_loc.schema_token().is_some() {
            DirectiveLocation::Schema
        } else if directive_loc.scalar_token().is_some() {
            DirectiveLocation::Scalar
        } else if directive_loc.object_token().is_some() {
            DirectiveLocation::Object
        } else if directive_loc.field_definition_token().is_some() {
            DirectiveLocation::FieldDefinition
        } else if directive_loc.argument_definition_token().is_some() {
            DirectiveLocation::ArgumentDefinition
        } else if directive_loc.interface_token().is_some() {
            DirectiveLocation::Interface
        } else if directive_loc.union_token().is_some() {
            DirectiveLocation::Union
        } else if directive_loc.enum_token().is_some() {
            DirectiveLocation::Enum
        } else if directive_loc.enum_value_token().is_some() {
            DirectiveLocation::EnumValue
        } else if directive_loc.input_object_token().is_some() {
            DirectiveLocation::InputObject
        } else {
            DirectiveLocation::InputFieldDefinition
        }
    }
}

impl From<DirectiveLocation> for String {
    fn from(dir_loc: DirectiveLocation) -> Self {
        match dir_loc {
            DirectiveLocation::Query => "QUERY".to_string(),
            DirectiveLocation::Mutation => "MUTATION".to_string(),
            DirectiveLocation::Subscription => "SUBSCRIPTION".to_string(),
            DirectiveLocation::Field => "FIELD".to_string(),
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION".to_string(),
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD".to_string(),
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT".to_string(),
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION".to_string(),
            DirectiveLocation::Schema => "SCHEMA".to_string(),
            DirectiveLocation::Scalar => "SCALAR".to_string(),
            DirectiveLocation::Object => "OBJECT".to_string(),
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION".to_string(),
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION".to_string(),
            DirectiveLocation::Interface => "INTERFACE".to_string(),
            DirectiveLocation::Union => "UNION".to_string(),
            DirectiveLocation::Enum => "ENUM".to_string(),
            DirectiveLocation::EnumValue => "ENUM_VALUE".to_string(),
            DirectiveLocation::InputObject => "INPUT_OBJECT".to_string(),
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION".to_string(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument {
    pub(crate) name: String,
    pub(crate) value: Value,
    pub(crate) ast_ptr: SyntaxNodePtr,
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

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

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
pub struct Variable {
    pub(crate) name: String,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl Variable {
    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SelectionSet {
    pub(crate) selection: Arc<Vec<Selection>>,
}

impl SelectionSet {
    /// Get a reference to the selection set's selection.
    pub fn selection(&self) -> &[Selection] {
        self.selection.as_ref()
    }

    /// Get a refernce to the selection set's fields (not inline fragments, or
    /// fragment spreads).
    pub fn fields(&self) -> Vec<Field> {
        let fields: Vec<Field> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::Field(field) => return Some(field.as_ref().clone()),
                _ => None,
            })
            .collect();

        fields
    }

    pub fn field(&self, name: &str) -> Option<&Field> {
        self.selection().iter().find_map(|sel| {
            if let Selection::Field(field) = sel {
                if field.name() == name {
                    return Some(field.as_ref());
                }
                None
            } else {
                None
            }
        })
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
    pub(crate) ty: Option<Type>,
    pub(crate) reference_ty_id: Option<Uuid>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) ast_ptr: SyntaxNodePtr,
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

    // Get a reference to field's type.
    pub fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }

    // Get field's original field definition.
    pub fn field_definition(&self, db: &dyn SourceDatabase) -> Option<FieldDefinition> {
        if let Some(object_id) = self.reference_ty_id {
            db.find_object_type(object_id)?
                .fields_definition()
                .iter()
                .find(|field| field.name() == self.name)
                .cloned()
        } else {
            None
        }
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
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Get variables used in the field.
    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let mut vars: Vec<_> = self
            .arguments
            .iter()
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(var.clone()),
                _ => None,
            })
            .collect();
        let iter = self
            .selection_set
            .selection()
            .iter()
            .flat_map(|sel| sel.variables(db));
        vars.extend(iter);
        vars
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }

    // pub fn field_definition(&self, db: &dyn SourceDatabase) -> Option<Arc<FieldDefinition>> {
    //     db.get
    // }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) ast_ptr: SyntaxNodePtr,
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
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let vars = self
            .selection_set
            .selection()
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect();
        vars
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    // NOTE @lrlna: this should just be Uuid.  If we can't find the framgment we
    // are looking for when populating this field, we should throw a semantic
    // error.
    pub(crate) fragment_id: Option<Uuid>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl FragmentSpread {
    pub fn fragment(&self, db: &dyn SourceDatabase) -> Option<Arc<FragmentDefinition>> {
        db.find_fragment(self.fragment_id?)
    }

    pub fn variables(&self, db: &dyn SourceDatabase) -> Vec<Variable> {
        let vars = match self.fragment(db) {
            Some(fragment) => fragment
                .selection_set
                .selection()
                .iter()
                .flat_map(|sel| sel.variables(db))
                .collect(),
            None => Vec::new(),
        };
        vars
    }

    /// Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
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
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
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

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
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
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
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

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }
}

impl Default for RootOperationTypeDefinition {
    fn default() -> Self {
        Self {
            object_type_id: None,
            operation_type: OperationType::Query,
            named_type: Type::Named {
                name: "Query".to_string(),
                ast_ptr: None,
            },
            ast_ptr: None,
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
    pub(crate) ast_ptr: SyntaxNodePtr,
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

    /// Find a field in object type definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    /// Get a reference to object type definition's implements interfaces vector.
    pub fn implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImplementsInterface {
    pub(crate) interface: String,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl ImplementsInterface {
    pub fn interface_definition(
        &self,
        db: &dyn SourceDatabase,
    ) -> Option<Arc<InterfaceTypeDefinition>> {
        db.find_interface_by_name(self.interface.clone())
    }

    pub fn interface(&self) -> &str {
        self.interface.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) ty: Type,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl FieldDefinition {
    /// Get a reference to the field definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    /// Get a reference to the field's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }

    // Get a reference to field definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    // Get a reference to field definition's arguments
    pub fn arguments(&self) -> &ArgumentsDefinition {
        &self.arguments
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArgumentsDefinition {
    pub(crate) input_values: Arc<Vec<InputValueDefinition>>,
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
}

impl ArgumentsDefinition {
    /// Get a reference to arguments definition's input values.
    pub fn input_values(&self) -> &[InputValueDefinition] {
        self.input_values.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) ty: Type,
    pub(crate) default_value: Option<DefaultValue>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
}

impl InputValueDefinition {
    // Get a reference to input value definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    // Get a reference to input value definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }

    // Get a reference to input value definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) ast_ptr: Option<SyntaxNodePtr>,
    pub(crate) built_in: bool,
}

impl ScalarTypeDefinition {
    /// Get the scalar type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the scalar definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to scalar definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> Option<&SyntaxNodePtr> {
        self.ast_ptr.as_ref()
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> Option<SyntaxNode> {
        self.ast_ptr()
            .map(|ptr| ptr.to_node(db.document().deref().syntax()))
    }

    // Returns true if the current scalar is a GraphQL built in.
    pub(crate) fn is_built_in(&self) -> bool {
        self.built_in
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) enum_values_definition: Arc<Vec<EnumValueDefinition>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl EnumTypeDefinition {
    /// Get the scalar type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

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

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) enum_value: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl EnumValueDefinition {
    /// Get a reference to enum value definition's enum value
    pub fn enum_value(&self) -> &str {
        self.enum_value.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) union_members: Arc<Vec<UnionMember>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl UnionTypeDefinition {
    /// Get the union type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

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

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionMember {
    pub(crate) name: String,
    pub(crate) object_id: Option<Uuid>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl UnionMember {
    /// Get a reference to the union member's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn object(&self, db: &dyn SourceDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type(self.object_id?)
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InterfaceTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl InterfaceTypeDefinition {
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

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputObjectTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) input_fields_definition: Arc<Vec<InputValueDefinition>>,
    pub(crate) ast_ptr: SyntaxNodePtr,
}

impl InputObjectTypeDefinition {
    /// Get the input object definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the input object definition's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get a reference to input object definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    pub fn input_fields_definition(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    // Get a reference to SyntaxNodePtr of the current HIR node.
    pub fn ast_ptr(&self) -> &SyntaxNodePtr {
        &self.ast_ptr
    }

    // Get current HIR node's AST node.
    pub fn ast_node(&self, db: &dyn SourceDatabase) -> SyntaxNode {
        let syntax_node_ptr = self.ast_ptr();
        syntax_node_ptr.to_node(db.document().deref().syntax())
    }
}
