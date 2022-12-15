use std::sync::Arc;

use apollo_parser::ast;
use ordered_float::{self, OrderedFloat};
use uuid::Uuid;

use crate::DocumentDatabase;

use super::FileId;

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
    SchemaExtension(SchemaExtension),
    ScalarTypeExtension(ScalarTypeExtension),
    ObjectTypeExtension(ObjectTypeExtension),
    InterfaceTypeExtension(InterfaceTypeExtension),
    UnionTypeExtension(UnionTypeExtension),
    EnumTypeExtension(EnumTypeExtension),
    InputObjectTypeExtension(InputObjectTypeExtension),
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
            Definition::SchemaExtension(_) => None,
            Definition::ScalarTypeExtension(def) => Some(def.name()),
            Definition::ObjectTypeExtension(def) => Some(def.name()),
            Definition::InterfaceTypeExtension(def) => Some(def.name()),
            Definition::UnionTypeExtension(def) => Some(def.name()),
            Definition::EnumTypeExtension(def) => Some(def.name()),
            Definition::InputObjectTypeExtension(def) => Some(def.name()),
        }
    }

    pub fn name_src(&self) -> Option<&Name> {
        match self {
            Definition::OperationDefinition(def) => def.name_src(),
            Definition::FragmentDefinition(def) => Some(def.name_src()),
            Definition::DirectiveDefinition(def) => Some(def.name_src()),
            Definition::ScalarTypeDefinition(def) => Some(def.name_src()),
            Definition::ObjectTypeDefinition(def) => Some(def.name_src()),
            Definition::InterfaceTypeDefinition(def) => Some(def.name_src()),
            Definition::UnionTypeDefinition(def) => Some(def.name_src()),
            Definition::EnumTypeDefinition(def) => Some(def.name_src()),
            Definition::InputObjectTypeDefinition(def) => Some(def.name_src()),
            Definition::SchemaDefinition(_) => None,
            Definition::SchemaExtension(_) => None,
            Definition::ScalarTypeExtension(def) => Some(def.name_src()),
            Definition::ObjectTypeExtension(def) => Some(def.name_src()),
            Definition::InterfaceTypeExtension(def) => Some(def.name_src()),
            Definition::UnionTypeExtension(def) => Some(def.name_src()),
            Definition::EnumTypeExtension(def) => Some(def.name_src()),
            Definition::InputObjectTypeExtension(def) => Some(def.name_src()),
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
            Definition::SchemaExtension(_) => "SchemaExtension".to_string(),
            Definition::ScalarTypeExtension(_) => "ScalarTypeExtension".to_string(),
            Definition::ObjectTypeExtension(_) => "ObjectTypeExtension".to_string(),
            Definition::InterfaceTypeExtension(_) => "InterfaceTypeExtension".to_string(),
            Definition::UnionTypeExtension(_) => "UnionTypeExtension".to_string(),
            Definition::EnumTypeExtension(_) => "EnumTypeExtension".to_string(),
            Definition::InputObjectTypeExtension(_) => "InputObjectTypeExtension".to_string(),
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
            Definition::SchemaExtension(_) => None,
            Definition::ScalarTypeExtension(_) => None,
            Definition::ObjectTypeExtension(_) => None,
            Definition::InterfaceTypeExtension(_) => None,
            Definition::UnionTypeExtension(_) => None,
            Definition::EnumTypeExtension(_) => None,
            Definition::InputObjectTypeExtension(_) => None,
        }
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        match self {
            Definition::ObjectTypeDefinition(def) => def.field(name),
            Definition::InterfaceTypeDefinition(def) => def.field(name),
            _ => None,
        }
    }

    pub fn directives(&self) -> &[Directive] {
        match self {
            Definition::OperationDefinition(def) => def.directives(),
            Definition::FragmentDefinition(def) => def.directives(),
            Definition::DirectiveDefinition(_) => &[],
            Definition::ScalarTypeDefinition(def) => def.directives(),
            Definition::ObjectTypeDefinition(def) => def.directives(),
            Definition::InterfaceTypeDefinition(def) => def.directives(),
            Definition::UnionTypeDefinition(def) => def.directives(),
            Definition::EnumTypeDefinition(def) => def.directives(),
            Definition::InputObjectTypeDefinition(def) => def.directives(),
            Definition::SchemaDefinition(def) => def.directives(),
            Definition::SchemaExtension(def) => def.directives(),
            Definition::ScalarTypeExtension(def) => def.directives(),
            Definition::ObjectTypeExtension(def) => def.directives(),
            Definition::InterfaceTypeExtension(def) => def.directives(),
            Definition::UnionTypeExtension(def) => def.directives(),
            Definition::EnumTypeExtension(def) => def.directives(),
            Definition::InputObjectTypeExtension(def) => def.directives(),
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
    pub(crate) name: Name,
    pub(crate) type_condition: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
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
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
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
    pub fn variables(&self, db: &dyn DocumentDatabase) -> Vec<Variable> {
        self.selection_set
            .selection()
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect()
    }

    pub fn ty(&self, db: &dyn DocumentDatabase) -> Option<Arc<Definition>> {
        db.find_type_system_definition_by_name(self.name().to_string())
    }

    /// Get fragment definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub(crate) id: Uuid,
    pub(crate) operation_ty: OperationType,
    pub(crate) name: Option<Name>,
    pub(crate) variables: Arc<Vec<VariableDefinition>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

impl OperationDefinition {
    /// Get a reference to operation definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the operation definition's ty.
    pub fn operation_ty(&self) -> &OperationType {
        &self.operation_ty
    }

    /// Get a mutable reference to the operation definition's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.src())
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// Get operation's definition object type.
    pub fn object_type(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        match self.operation_ty {
            OperationType::Query => db.schema().query(db),
            OperationType::Mutation => db.schema().mutation(db),
            OperationType::Subscription => db.schema().subscription(db),
        }
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

    /// Get fields in the operation definition (excluding inline fragments and
    /// fragment spreads).
    pub fn fields(&self, db: &dyn DocumentDatabase) -> Arc<Vec<Field>> {
        db.operation_fields(self.id)
    }

    // NOTE @lrlna: this is quite messy. it should live under the
    // inline_fragment/fragment_spread impls, i.e. op.fragment_spread().fields(),
    // op.inline_fragments().fields()
    //
    // We will need to figure out how to store operation definition id on its
    // fragment spreads and inline fragments to do this

    /// Get all fields in an inline fragment.
    pub fn fields_in_inline_fragments(&self, db: &dyn DocumentDatabase) -> Arc<Vec<Field>> {
        db.operation_inline_fragment_fields(self.id)
    }

    /// Get all fields in a fragment spread
    pub fn fields_in_fragment_spread(&self, db: &dyn DocumentDatabase) -> Arc<Vec<Field>> {
        db.operation_fragment_spread_fields(self.id)
    }

    /// Get operation definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
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
            OperationType::Query => write!(f, "Query"),
            OperationType::Mutation => write!(f, "Mutation"),
            OperationType::Subscription => write!(f, "Subscription"),
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
    pub(crate) name: Name,
    pub(crate) ty: Type,
    pub(crate) default_value: Option<DefaultValue>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl VariableDefinition {
    /// Get a reference to the variable definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to the variable definition's ty.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a reference to the variable definition's default value.
    pub fn default_value(&self) -> Option<&DefaultValue> {
        self.default_value.as_ref()
    }

    /// Get a reference to the variable definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get variable definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    NonNull {
        ty: Box<Type>,
        loc: Option<HirNodeLocation>,
    },
    List {
        ty: Box<Type>,
        loc: Option<HirNodeLocation>,
    },
    Named {
        name: String,
        loc: Option<HirNodeLocation>,
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
    pub fn is_output_type(&self, db: &dyn DocumentDatabase) -> bool {
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
    pub fn is_input_type(&self, db: &dyn DocumentDatabase) -> bool {
        if let Some(ty) = self.ty(db) {
            ty.as_ref().is_input_definition()
        } else {
            false
        }
    }

    /// Get a reference to location information of the current HIR node.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        match self {
            Type::NonNull { ty: _, loc } => loc.as_ref(),
            Type::List { ty: _, loc } => loc.as_ref(),
            Type::Named { name: _, loc } => loc.as_ref(),
        }
    }

    pub fn ty(&self, db: &dyn DocumentDatabase) -> Option<Arc<Definition>> {
        db.find_definition_by_name(self.name())
    }

    pub fn name(&self) -> String {
        match self {
            Type::NonNull { ty, loc: _ } => get_name(*ty.clone()),
            Type::List { ty, loc: _ } => get_name(*ty.clone()),
            Type::Named { name, loc: _ } => name.to_owned(),
        }
    }
}

fn get_name(ty: Type) -> String {
    match ty {
        Type::NonNull { ty, loc: _ } => match *ty {
            Type::NonNull { ty, loc: _ } => get_name(*ty),
            Type::List { ty, loc: _ } => get_name(*ty),
            Type::Named { name, loc: _ } => name,
        },
        Type::List { ty, loc: _ } => match *ty {
            Type::NonNull { ty, loc: _ } => get_name(*ty),
            Type::List { ty, loc: _ } => get_name(*ty),
            Type::Named { name, loc: _ } => name,
        },
        Type::Named { name, loc: _ } => name,
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub(crate) name: Name,
    pub(crate) arguments: Arc<Vec<Argument>>,
    pub(crate) loc: HirNodeLocation,
}

impl Directive {
    /// Get a reference to the directive's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to the directive's arguments.
    pub fn arguments(&self) -> &[Argument] {
        self.arguments.as_ref()
    }

    /// Get a reference to the value of the directive argument with the given name, if it exists.
    pub fn argument_by_name(&self, name: &str) -> Option<&Value> {
        Some(
            self.arguments
                .iter()
                .find(|arg| arg.name() == name)?
                .value(),
        )
    }

    // Get directive definition of the currently used directive
    pub fn directive(&self, db: &dyn DocumentDatabase) -> Option<Arc<DirectiveDefinition>> {
        db.find_directive_definition_by_name(self.name().to_string())
    }

    /// Get directive's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DirectiveDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) repeatable: bool,
    pub(crate) directive_locations: Arc<Vec<DirectiveLocation>>,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl DirectiveDefinition {
    /// Get a reference to the directive definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the directive definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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

    /// Indicates whether a directive may be used multiple times in a single location.
    pub fn repeatable(&self) -> bool {
        self.repeatable
    }

    /// Get directive definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
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
    pub(crate) name: Name,
    pub(crate) value: Value,
    pub(crate) loc: HirNodeLocation,
}

impl Argument {
    /// Get a reference to the argument's value.
    pub fn value(&self) -> &Value {
        &self.value
    }

    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get argument's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

pub type DefaultValue = Value;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    Variable(Variable),

    // A value of integer syntax may be coerced to a Float input value:
    // https://spec.graphql.org/draft/#sec-Float.Input-Coercion
    // Keep a f64 here instead of i32 in order to support
    // the full range of f64 integer values for that case.
    //
    // All i32 values can be represented exactly in f64,
    // so conversion to an Int input value is still exact:
    // https://spec.graphql.org/draft/#sec-Int.Input-Coercion
    Int(Float),
    Float(Float),
    String(String),
    Boolean(bool),
    Null,
    Enum(Name),
    List(Vec<Value>),
    Object(Vec<(Name, Value)>),
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

/// Coerce to a `Float` input type (from either `Float` or `Int` syntax)
///
/// <https://spec.graphql.org/draft/#sec-Float.Input-Coercion>
impl TryFrom<Value> for f64 {
    type Error = FloatCoercionError;

    #[inline]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        f64::try_from(&value)
    }
}

/// Coerce to a `Float` input type (from either `Float` or `Int` syntax)
///
/// <https://spec.graphql.org/draft/#sec-Float.Input-Coercion>
impl TryFrom<&'_ Value> for f64 {
    type Error = FloatCoercionError;

    fn try_from(value: &'_ Value) -> Result<Self, Self::Error> {
        if let Value::Int(float) | Value::Float(float) = value {
            // FIXME: what does "a value outside the available precision" mean?
            // Should coercion fail when f64 does not have enough mantissa bits
            // to represent the source token exactly?
            Ok(float.inner.0)
        } else {
            Err(FloatCoercionError(()))
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("coercing a non-numeric value to a `Float` input value")]
pub struct FloatCoercionError(());

/// Coerce to an `Int` input type
///
/// <https://spec.graphql.org/draft/#sec-Int.Input-Coercion>
impl TryFrom<Value> for i32 {
    type Error = IntCoercionError;

    #[inline]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        i32::try_from(&value)
    }
}

/// Coerce to an `Int` input type
///
/// <https://spec.graphql.org/draft/#sec-Int.Input-Coercion>
impl TryFrom<&'_ Value> for i32 {
    type Error = IntCoercionError;

    fn try_from(value: &'_ Value) -> Result<Self, Self::Error> {
        if let Value::Int(float) = value {
            // The parser emitted an `ast::IntValue` instead of `ast::FloatValue`
            // so we already know `float` does not have a frational part.
            float
                .to_i32_checked()
                .ok_or(IntCoercionError::RangeOverflow)
        } else {
            Err(IntCoercionError::NotAnInteger)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IntCoercionError {
    #[error("coercing a non-integer value to an `Int` input value")]
    NotAnInteger,
    #[error("integer input value overflows the signed 32-bit range")]
    RangeOverflow,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) loc: HirNodeLocation,
}

impl Variable {
    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get variable's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
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

    /// Get a reference to selection set's fragment spread.
    pub fn fragment_spreads(&self) -> Vec<FragmentSpread> {
        let fragment_spread: Vec<FragmentSpread> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::FragmentSpread(fragment_spread) => {
                    return Some(fragment_spread.as_ref().clone())
                }
                _ => None,
            })
            .collect();

        fragment_spread
    }

    /// Get a reference to selection set's inline fragments.
    pub fn inline_fragments(&self) -> Vec<InlineFragment> {
        let inline_fragments: Vec<InlineFragment> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::InlineFragment(inline) => return Some(inline.as_ref().clone()),
                _ => None,
            })
            .collect();

        inline_fragments
    }

    /// Find a field a selection set.
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
    pub fn variables(&self, db: &dyn DocumentDatabase) -> Vec<Variable> {
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
    pub(crate) name: Name,
    pub(crate) arguments: Arc<Vec<Argument>>,
    pub(crate) parent_obj: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
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
        self.name.src()
    }

    /// Get a reference to field's type.
    pub fn ty(&self, db: &dyn DocumentDatabase) -> Option<Type> {
        let def = db
            .find_type_system_definition_by_name(self.parent_obj.as_ref()?.to_string())?
            .field(self.name())?
            .ty()
            .to_owned();
        Some(def)
    }

    /// Get field's original field definition.
    pub fn field_definition(&self, db: &dyn DocumentDatabase) -> Option<FieldDefinition> {
        db.find_object_type_by_name(self.parent_obj.as_ref()?.to_string())?
            .fields_definition()
            .iter()
            .find(|field| field.name() == self.name())
            .cloned()
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
    pub fn variables(&self, db: &dyn DocumentDatabase) -> Vec<Variable> {
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

    /// Get field's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<Name>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

impl InlineFragment {
    /// Get a reference to inline fragment's type condition.
    pub fn type_condition(&self) -> Option<&str> {
        self.type_condition.as_ref().map(|t| t.src())
    }

    /// Get a reference to inline fragment's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference inline fragment's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Get variables in use in the inline fragment.
    pub fn variables(&self, db: &dyn DocumentDatabase) -> Vec<Variable> {
        let vars = self
            .selection_set
            .selection()
            .iter()
            .flat_map(|sel| sel.variables(db))
            .collect();
        vars
    }

    /// Get inline fragment's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl FragmentSpread {
    /// Get a reference to the fragmen spread's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the fragment definition this fragment spread is referencing.
    pub fn fragment(&self, db: &dyn DocumentDatabase) -> Option<Arc<FragmentDefinition>> {
        db.find_fragment_by_name(self.loc.file_id(), self.name().to_string())
    }

    /// Get framgent spread's defined variables.
    pub fn variables(&self, db: &dyn DocumentDatabase) -> Vec<Variable> {
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

    /// Get a reference to fragment spread directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get fragment spread's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Float {
    inner: ordered_float::OrderedFloat<f64>,
}

impl Float {
    pub fn new(float: f64) -> Self {
        Self {
            inner: OrderedFloat(float),
        }
    }

    pub fn get(self) -> f64 {
        self.inner.0
    }

    /// If the value is in the `i32` range, convert by rounding towards zero.
    ///
    /// (This is mostly useful when matching on [`Value::Int`]
    /// where the value is known not to have a fractional part
    ///  so the rounding mode doesn’t affect the result.)
    pub fn to_i32_checked(self) -> Option<i32> {
        let float = self.inner.0;
        if float <= (i32::MAX as f64) && float >= (i32::MIN as f64) {
            Some(float as i32)
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Default, Eq)]
pub struct SchemaDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl SchemaDefinition {
    /// Get a reference to the schema definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

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

    /// Get schema definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }

    // NOTE(@lrlna): potentially have the following fns on the database itself
    // so they are memoised as well

    /// Get Schema's query object type definition.
    pub fn query(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_query() {
                op.object_type(db)
            } else {
                None
            }
        })
    }

    /// Get Schema's mutation object type definition.
    pub fn mutation(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_mutation() {
                op.object_type(db)
            } else {
                None
            }
        })
    }

    /// Get Schema's subscription object type definition.
    pub fn subscription(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_type.is_subscription() {
                op.object_type(db)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootOperationTypeDefinition {
    pub(crate) operation_type: OperationType,
    pub(crate) named_type: Type,
    pub(crate) loc: Option<HirNodeLocation>,
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

    pub fn object_type(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type_by_name(self.named_type().name())
    }

    pub fn object_type_id(&self, db: &dyn DocumentDatabase) -> Option<Uuid> {
        self.object_type(db).map(|object_type| *object_type.id())
    }

    /// Get root operation type definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }
}

impl Default for RootOperationTypeDefinition {
    fn default() -> Self {
        Self {
            operation_type: OperationType::Query,
            named_type: Type::Named {
                name: "Query".to_string(),
                loc: None,
            },
            loc: None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ObjectTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl ObjectTypeDefinition {
    /// Get the object type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the object type definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the object type definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to the object type definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
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

    /// Get object type definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ImplementsInterface {
    pub(crate) interface: Name,
    pub(crate) loc: HirNodeLocation,
}

impl ImplementsInterface {
    /// Get the interface this implements interface is referencing.
    pub fn interface_definition(
        &self,
        db: &dyn DocumentDatabase,
    ) -> Option<Arc<InterfaceTypeDefinition>> {
        db.find_interface_by_name(self.interface().to_string())
    }

    /// Get implements interfaces' interface name.
    pub fn interface(&self) -> &str {
        self.interface.src()
    }

    /// Get implements interfaces's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) ty: Type,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl FieldDefinition {
    /// Get a reference to the field definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to the field definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to the field's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get field definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }

    /// Get a reference to field definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a reference to field definition's arguments
    pub fn arguments(&self) -> &ArgumentsDefinition {
        &self.arguments
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ArgumentsDefinition {
    pub(crate) input_values: Arc<Vec<InputValueDefinition>>,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl ArgumentsDefinition {
    /// Get a reference to arguments definition's input values.
    pub fn input_values(&self) -> &[InputValueDefinition] {
        self.input_values.as_ref()
    }

    /// Get arguments definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) ty: Type,
    pub(crate) default_value: Option<DefaultValue>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl InputValueDefinition {
    /// Get a reference to input value definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to input value definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to input value definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get input value definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }

    /// Get a reference to input value definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a refernce to inpul value definition's default_value.
    pub fn default_value(&self) -> Option<&DefaultValue> {
        self.default_value.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) built_in: bool,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl ScalarTypeDefinition {
    /// Get the scalar type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the scalar definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the scalar definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to scalar definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns true if the current scalar is a GraphQL built in.
    pub fn is_built_in(&self) -> bool {
        self.built_in
    }

    /// Get scalar type definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) enum_values_definition: Arc<Vec<EnumValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl EnumTypeDefinition {
    /// Get the scalar type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the enum definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the enum definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to enum definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to enum definition's enum values definition vector.
    pub fn enum_values_definition(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }

    /// Get enum type definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumValueDefinition {
    pub(crate) description: Option<String>,
    pub(crate) enum_value: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl EnumValueDefinition {
    /// Get a reference to enum value definition's enum value
    pub fn enum_value(&self) -> &str {
        self.enum_value.src()
    }

    /// Get a reference to enum value definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get enum value definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) union_members: Arc<Vec<UnionMember>>,
    pub(crate) loc: HirNodeLocation,
}

impl UnionTypeDefinition {
    /// Get the union type definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the union definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the union definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to union definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to union definition's union members.
    pub fn union_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }

    /// Get union type definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionMember {
    pub(crate) name: Name,
    pub(crate) loc: HirNodeLocation,
}

impl UnionMember {
    /// Get a reference to the union member's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the object definition this union member is referencing.
    pub fn object(&self, db: &dyn DocumentDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type_by_name(self.name().to_string())
    }

    /// Get union member's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InterfaceTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl InterfaceTypeDefinition {
    /// Get the interface definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the interface definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the interface definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to interface definition's implements interfaces vector.
    pub fn implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    /// Get a reference to the interface definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to interface definition's fields.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    /// Get interface type definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputObjectTypeDefinition {
    pub(crate) id: Uuid,
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) input_fields_definition: Arc<Vec<InputValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl InputObjectTypeDefinition {
    /// Get the input object definition's id.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a reference to the input object definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to the input object definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to input object definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to input fields definitions.
    pub fn input_fields_definition(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    /// Get input object type definition's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Name {
    pub(crate) src: String,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl Name {
    /// Get a reference to the name itself.
    pub fn src(&self) -> &str {
        self.src.as_ref()
    }

    /// Get input object type definition's hir node location.
    pub fn loc(&self) -> Option<&HirNodeLocation> {
        self.loc.as_ref()
    }
}

impl From<Name> for String {
    fn from(name: Name) -> String {
        name.src().to_owned()
    }
}

impl From<String> for Name {
    fn from(name: String) -> Name {
        Name {
            src: name,
            loc: None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SchemaExtension {
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl SchemaExtension {
    /// Get a reference to the schema definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to the schema definition's root operation type definition.
    pub fn root_operation_type_definition(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Get schema extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarTypeExtension {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl ScalarTypeExtension {
    /// Get a reference to the scalar definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to scalar definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get scalar type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ObjectTypeExtension {
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl ObjectTypeExtension {
    /// Get a reference to the object type definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }
    /// Get a reference to the object type definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
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

    /// Get object type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InterfaceTypeExtension {
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl InterfaceTypeExtension {
    /// Get a reference to the interface definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to interface definition's implements interfaces vector.
    pub fn implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    /// Get a reference to the interface definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to interface definition's fields.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    /// Get interface type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionTypeExtension {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) union_members: Arc<Vec<UnionMember>>,
    pub(crate) loc: HirNodeLocation,
}

impl UnionTypeExtension {
    /// Get a reference to the union definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to union definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to union definition's union members.
    pub fn union_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }

    /// Get union type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumTypeExtension {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) enum_values_definition: Arc<Vec<EnumValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl EnumTypeExtension {
    /// Get a reference to the enum definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to enum definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Get a reference to enum definition's enum values definition vector.
    pub fn enum_values_definition(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }

    /// Get enum type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputObjectTypeExtension {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) input_fields_definition: Arc<Vec<InputValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl InputObjectTypeExtension {
    /// Get a reference to the input object definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to input object definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    pub fn input_fields_definition(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    /// Get input object type extension's hir node location.
    pub fn loc(&self) -> &HirNodeLocation {
        &self.loc
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HirNodeLocation {
    pub(crate) offset: usize,
    pub(crate) node_len: usize,
    pub(crate) file_id: FileId,
}

impl HirNodeLocation {
    /// Get file id of the current node.
    pub fn file_id(&self) -> FileId {
        self.file_id
    }

    /// Get source offset of the current node.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Get node length.
    pub fn node_len(&self) -> usize {
        self.node_len
    }
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::DocumentDatabase;

    #[test]
    fn huge_floats() {
        let mut compiler = ApolloCompiler::new();
        compiler.create_schema(
            "input HugeFloats {
                a: Float = 9876543210
                b: Float = 9876543210.0
                c: Float = 98765432109876543210
                d: Float = 98765432109876543210.0
            }",
            "huge_floats.graphql",
        );

        let default_values: Vec<_> = compiler
            .db
            .find_input_object_by_name("HugeFloats".into())
            .unwrap()
            .input_fields_definition
            .iter()
            .map(|field| {
                f64::try_from(field.default_value().unwrap())
                    .unwrap()
                    .to_string()
            })
            .collect();
        // The exact value is preserved, even outside of the range of i32
        assert_eq!(default_values[0], "9876543210");
        assert_eq!(default_values[1], "9876543210");
        // Beyond ~53 bits of mantissa we may lose precision,
        // but this is approximation is still in the range of finite f64 values.
        assert_eq!(default_values[2], "98765432109876540000");
        assert_eq!(default_values[3], "98765432109876540000");
    }

    #[test]
    fn syntax_errors() {
        let mut compiler = ApolloCompiler::new();
        compiler.create_schema(
            "type Person {
                id: ID!
                name: String
                appearedIn: [Film]s
                directed: [Film]
            }",
            "person.graphql",
        );
        let person = compiler
            .db
            .find_object_type_by_name("Person".into())
            .unwrap();
        let hir_field_names: Vec<_> = person
            .fields_definition
            .iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(hir_field_names, ["id", "name", "appearedIn", "directed"]);
    }
}
