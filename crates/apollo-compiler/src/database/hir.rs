use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::Arc,
};

use apollo_parser::{ast, SyntaxNode};
use ordered_float::{self, OrderedFloat};

use crate::{HirDatabase, Source};

use super::FileId;
use indexmap::IndexMap;

pub type ByName<T> = Arc<IndexMap<String, Arc<T>>>;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TypeSystemDefinitions {
    pub schema: Arc<SchemaDefinition>,
    pub scalars: ByName<ScalarTypeDefinition>,
    pub objects: ByName<ObjectTypeDefinition>,
    pub interfaces: ByName<InterfaceTypeDefinition>,
    pub unions: ByName<UnionTypeDefinition>,
    pub enums: ByName<EnumTypeDefinition>,
    pub input_objects: ByName<InputObjectTypeDefinition>,
    pub directives: ByName<DirectiveDefinition>,
}

/// Contains `TypeSystemDefinitions` together with:
///
/// * Other data that can be derived from it, computed eagerly
/// * Relevant inputs, so that diagnostics can print context
///
/// This can be used with [`set_type_system_hir`][crate::ApolloCompiler::set_type_system_hir]
/// on another compiler.
#[derive(PartialEq, Eq, Debug)]
pub struct TypeSystem {
    pub definitions: Arc<TypeSystemDefinitions>,
    pub inputs: IndexMap<FileId, Source>,
    pub type_definitions_by_name: Arc<IndexMap<String, TypeDefinition>>,
    pub subtype_map: Arc<HashMap<String, HashSet<String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeDefinition {
    ScalarTypeDefinition(Arc<ScalarTypeDefinition>),
    ObjectTypeDefinition(Arc<ObjectTypeDefinition>),
    InterfaceTypeDefinition(Arc<InterfaceTypeDefinition>),
    UnionTypeDefinition(Arc<UnionTypeDefinition>),
    EnumTypeDefinition(Arc<EnumTypeDefinition>),
    InputObjectTypeDefinition(Arc<InputObjectTypeDefinition>),
}

impl TypeDefinition {
    pub fn name(&self) -> &str {
        match self {
            Self::ScalarTypeDefinition(def) => def.name(),
            Self::ObjectTypeDefinition(def) => def.name(),
            Self::InterfaceTypeDefinition(def) => def.name(),
            Self::UnionTypeDefinition(def) => def.name(),
            Self::EnumTypeDefinition(def) => def.name(),
            Self::InputObjectTypeDefinition(def) => def.name(),
        }
    }

    pub fn name_src(&self) -> &Name {
        match self {
            Self::ScalarTypeDefinition(def) => def.name_src(),
            Self::ObjectTypeDefinition(def) => def.name_src(),
            Self::InterfaceTypeDefinition(def) => def.name_src(),
            Self::UnionTypeDefinition(def) => def.name_src(),
            Self::EnumTypeDefinition(def) => def.name_src(),
            Self::InputObjectTypeDefinition(def) => def.name_src(),
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ScalarTypeDefinition(_) => "ScalarTypeDefinition",
            Self::ObjectTypeDefinition(_) => "ObjectTypeDefinition",
            Self::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition",
            Self::UnionTypeDefinition(_) => "UnionTypeDefinition",
            Self::EnumTypeDefinition(_) => "EnumTypeDefinition",
            Self::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition",
        }
    }

    /// Returns whether this definition is a composite definition (union, interface, or object).
    #[must_use]
    pub fn is_composite_definition(&self) -> bool {
        matches!(
            self,
            Self::ObjectTypeDefinition(_)
                | Self::InterfaceTypeDefinition(_)
                | Self::UnionTypeDefinition(_)
        )
    }

    /// Returns whether this definition is a scalar, object, interface, union, or enum.
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

    /// Returns whether this definition is an input object, scalar, or enum.
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

    /// Returns directives of this type definition (excluding those on its extensions)
    pub fn self_directives(&self) -> &[Directive] {
        match self {
            Self::ScalarTypeDefinition(def) => def.self_directives(),
            Self::ObjectTypeDefinition(def) => def.self_directives(),
            Self::InterfaceTypeDefinition(def) => def.self_directives(),
            Self::UnionTypeDefinition(def) => def.self_directives(),
            Self::EnumTypeDefinition(def) => def.self_directives(),
            Self::InputObjectTypeDefinition(def) => def.self_directives(),
        }
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        match self {
            Self::ScalarTypeDefinition(def) => {
                // Use `Box<dyn _>` since each inner method returns a different iterator type.
                // https://crates.io/crates/enum_dispatch could be used instead
                // but is it worth the trouble?
                Box::new(def.directives()) as Box<dyn Iterator<Item = &Directive>>
            }
            Self::ObjectTypeDefinition(def) => Box::new(def.directives()),
            Self::InterfaceTypeDefinition(def) => Box::new(def.directives()),
            Self::UnionTypeDefinition(def) => Box::new(def.directives()),
            Self::EnumTypeDefinition(def) => Box::new(def.directives()),
            Self::InputObjectTypeDefinition(def) => Box::new(def.directives()),
        }
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the type definition or its type extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the type definition or its type extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        match self {
            Self::ObjectTypeDefinition(def) => def.field(name),
            Self::InterfaceTypeDefinition(def) => def.field(name),
            _ => None,
        }
    }

    pub fn loc(&self) -> Option<HirNodeLocation> {
        match self {
            Self::ObjectTypeDefinition(def) => Some(def.loc()),
            Self::InterfaceTypeDefinition(def) => Some(def.loc()),
            Self::UnionTypeDefinition(def) => Some(def.loc()),
            Self::EnumTypeDefinition(def) => Some(def.loc()),
            Self::InputObjectTypeDefinition(def) => Some(def.loc()),
            Self::ScalarTypeDefinition(def) => def.loc(),
        }
    }
    /// Returns `true` if the type definition is [`ScalarTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: TypeDefinition::ScalarTypeDefinition
    #[must_use]
    pub fn is_scalar_type_definition(&self) -> bool {
        matches!(self, Self::ScalarTypeDefinition(..))
    }

    /// Returns `true` if the type definition is [`ObjectTypeDefinition`].
    ///
    /// [`ObjectTypeDefinition`]: TypeDefinition::ObjectTypeDefinition
    #[must_use]
    pub fn is_object_type_definition(&self) -> bool {
        matches!(self, Self::ObjectTypeDefinition(..))
    }

    /// Returns `true` if the type definition is [`InterfaceTypeDefinition`].
    ///
    /// [`InterfaceTypeDefinition`]: TypeDefinition::InterfaceTypeDefinition
    #[must_use]
    pub fn is_interface_type_definition(&self) -> bool {
        matches!(self, Self::InterfaceTypeDefinition(..))
    }

    /// Returns `true` if the type definition is [`UnionTypeDefinition`].
    ///
    /// [`UnionTypeDefinition`]: TypeDefinition::UnionTypeDefinition
    #[must_use]
    pub fn is_union_type_definition(&self) -> bool {
        matches!(self, Self::UnionTypeDefinition(..))
    }

    /// Returns `true` if the type definition is [`EnumTypeDefinition`].
    ///
    /// [`EnumTypeDefinition`]: TypeDefinition::EnumTypeDefinition
    #[must_use]
    pub fn is_enum_type_definition(&self) -> bool {
        matches!(self, Self::EnumTypeDefinition(..))
    }

    /// Returns `true` if the type definition is [`InputObjectTypeDefinition`].
    ///
    /// [`InputObjectTypeDefinition`]: TypeDefinition::InputObjectTypeDefinition
    #[must_use]
    pub fn is_input_object_type_definition(&self) -> bool {
        matches!(self, Self::InputObjectTypeDefinition(..))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TypeExtension {
    ScalarTypeExtension(Arc<ScalarTypeExtension>),
    ObjectTypeExtension(Arc<ObjectTypeExtension>),
    InterfaceTypeExtension(Arc<InterfaceTypeExtension>),
    UnionTypeExtension(Arc<UnionTypeExtension>),
    EnumTypeExtension(Arc<EnumTypeExtension>),
    InputObjectTypeExtension(Arc<InputObjectTypeExtension>),
}

impl TypeExtension {
    pub fn name(&self) -> &str {
        match self {
            Self::ScalarTypeExtension(def) => def.name(),
            Self::ObjectTypeExtension(def) => def.name(),
            Self::InterfaceTypeExtension(def) => def.name(),
            Self::UnionTypeExtension(def) => def.name(),
            Self::EnumTypeExtension(def) => def.name(),
            Self::InputObjectTypeExtension(def) => def.name(),
        }
    }

    pub fn name_src(&self) -> &Name {
        match self {
            Self::ScalarTypeExtension(def) => def.name_src(),
            Self::ObjectTypeExtension(def) => def.name_src(),
            Self::InterfaceTypeExtension(def) => def.name_src(),
            Self::UnionTypeExtension(def) => def.name_src(),
            Self::EnumTypeExtension(def) => def.name_src(),
            Self::InputObjectTypeExtension(def) => def.name_src(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::ScalarTypeExtension(_) => "ScalarTypeExtension",
            Self::ObjectTypeExtension(_) => "ObjectTypeExtension",
            Self::InterfaceTypeExtension(_) => "InterfaceTypeExtension",
            Self::UnionTypeExtension(_) => "UnionTypeExtension",
            Self::EnumTypeExtension(_) => "EnumTypeExtension",
            Self::InputObjectTypeExtension(_) => "InputObjectTypeExtension",
        }
    }

    pub fn directives(&self) -> &[Directive] {
        match self {
            Self::ScalarTypeExtension(def) => def.directives(),
            Self::ObjectTypeExtension(def) => def.directives(),
            Self::InterfaceTypeExtension(def) => def.directives(),
            Self::UnionTypeExtension(def) => def.directives(),
            Self::EnumTypeExtension(def) => def.directives(),
            Self::InputObjectTypeExtension(def) => def.directives(),
        }
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        match self {
            Self::ObjectTypeExtension(def) => def.field(name),
            Self::InterfaceTypeExtension(def) => def.field(name),
            _ => None,
        }
    }

    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Self::ObjectTypeExtension(def) => def.loc(),
            Self::InterfaceTypeExtension(def) => def.loc(),
            Self::UnionTypeExtension(def) => def.loc(),
            Self::EnumTypeExtension(def) => def.loc(),
            Self::InputObjectTypeExtension(def) => def.loc(),
            Self::ScalarTypeExtension(def) => def.loc(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentDefinition {
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
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
    ///
    /// TODO(@goto-bus-stop): Maybe rename this to used_variables
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.selection_set.variables(db)
    }

    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.name().to_string())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub(crate) operation_ty: OperationType,
    pub(crate) name: Option<Name>,
    pub(crate) variables: Arc<Vec<VariableDefinition>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

impl OperationDefinition {
    /// Get the kind of the operation: `query`, `mutation`, or `subscription`
    pub fn operation_ty(&self) -> OperationType {
        self.operation_ty
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
    pub fn object_type(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to the operation definition's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Get fields in the operation definition (excluding inline fragments and
    /// fragment spreads).
    pub fn fields(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_fields(self.selection_set.clone())
    }

    // NOTE @lrlna: this is quite messy. it should live under the
    // inline_fragment/fragment_spread impls, i.e. op.fragment_spread().fields(),
    // op.inline_fragments().fields()
    //
    // We will need to figure out how to store operation definition id on its
    // fragment spreads and inline fragments to do this

    /// Get all fields in an inline fragment.
    pub fn fields_in_inline_fragments(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_inline_fragment_fields(self.selection_set.clone())
    }

    /// Get all fields in a fragment spread
    pub fn fields_in_fragment_spread(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_fragment_spread_fields(self.selection_set.clone())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if this is a query operation and its [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        self.operation_ty().is_query() && self.selection_set().is_introspection(db)
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

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Query => write!(f, "Query"),
            OperationType::Mutation => write!(f, "Mutation"),
            OperationType::Subscription => write!(f, "Subscription"),
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

impl From<OperationType> for DirectiveLocation {
    fn from(op_type: OperationType) -> Self {
        if op_type.is_subscription() {
            DirectiveLocation::Subscription
        } else if op_type.is_mutation() {
            DirectiveLocation::Mutation
        } else {
            DirectiveLocation::Query
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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
    pub fn is_output_type(&self, db: &dyn HirDatabase) -> bool {
        if let Some(ty) = self.type_def(db) {
            ty.is_output_definition()
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
    pub fn is_input_type(&self, db: &dyn HirDatabase) -> bool {
        if let Some(ty) = self.type_def(db) {
            ty.is_input_definition()
        } else {
            false
        }
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        match self {
            Type::NonNull { loc, .. } | Type::List { loc, .. } | Type::Named { loc, .. } => *loc,
        }
    }

    /// Get current Type's Type Definition.
    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.name())
    }

    /// Get current Type's name.
    pub fn name(&self) -> String {
        match self {
            Type::NonNull { ty, .. } | Type::List { ty, .. } => ty.name(),
            Type::Named { name, .. } => name.clone(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::NonNull { ty, .. } => write!(f, "{ty}!"),
            Type::List { ty, .. } => write!(f, "[{ty}]"),
            Type::Named { name, .. } => write!(f, "{name}"),
        }
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
    pub fn directive(&self, db: &dyn HirDatabase) -> Option<Arc<DirectiveDefinition>> {
        db.find_directive_definition_by_name(self.name().to_string())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct DirectiveDefinition {
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
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

impl DirectiveLocation {
    /// Get the name of this directive location as it would appear in GraphQL source code.
    pub fn name(self) -> &'static str {
        match self {
            DirectiveLocation::Query => "QUERY",
            DirectiveLocation::Mutation => "MUTATION",
            DirectiveLocation::Subscription => "SUBSCRIPTION",
            DirectiveLocation::Field => "FIELD",
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION",
            DirectiveLocation::Schema => "SCHEMA",
            DirectiveLocation::Scalar => "SCALAR",
            DirectiveLocation::Object => "OBJECT",
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocation::Interface => "INTERFACE",
            DirectiveLocation::Union => "UNION",
            DirectiveLocation::Enum => "ENUM",
            DirectiveLocation::EnumValue => "ENUM_VALUE",
            DirectiveLocation::InputObject => "INPUT_OBJECT",
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
        }
    }
}

impl fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns `true` if `other` represents the same value as `self`. This is different from the
    /// `Eq` implementation as it ignores location information.
    pub fn is_same_value(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Variable(left), Value::Variable(right)) => left.name() == right.name(),
            (Value::Int(left) | Value::Float(left), Value::Int(right) | Value::Float(right)) => {
                left == right
            }
            (Value::String(left), Value::String(right)) => left == right,
            (Value::Boolean(left), Value::Boolean(right)) => left == right,
            (Value::Null, Value::Null) => true,
            (Value::Enum(left), Value::Enum(right)) => left.src() == right.src(),
            (Value::List(left), Value::List(right)) if left.len() == right.len() => left
                .iter()
                .zip(right)
                .all(|(left, right)| left.is_same_value(right)),
            (Value::Object(left), Value::Object(right)) if left.len() == right.len() => {
                left.iter().zip(right).all(|(left, right)| {
                    left.0.src() == left.0.src() && left.1.is_same_value(&right.1)
                })
            }
            _ => false,
        }
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Get all variables used in this selection set.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        /// Recursively collect used variables. Accounts for self-referential fragments.
        fn collect_used_variables(
            db: &dyn HirDatabase,
            set: &SelectionSet,
            seen_fragments: &mut HashSet<Arc<FragmentDefinition>>,
            output: &mut Vec<Variable>,
        ) {
            for selection in set.selection() {
                match selection {
                    Selection::Field(field) => {
                        output.extend(field.self_used_variables());
                        collect_used_variables(db, field.selection_set(), seen_fragments, output);
                    }
                    Selection::FragmentSpread(spread) => {
                        let Some(fragment) = spread.fragment(db) else {
                            return;
                        };
                        if seen_fragments.contains(&fragment) {
                            return; // prevent recursion loop
                        }
                        seen_fragments.insert(Arc::clone(&fragment));
                        collect_used_variables(
                            db,
                            fragment.selection_set(),
                            seen_fragments,
                            output,
                        );
                    }
                    Selection::InlineFragment(inline) => {
                        collect_used_variables(db, inline.selection_set(), seen_fragments, output);
                    }
                }
            }
        }

        let mut output = vec![];
        collect_used_variables(db, self, &mut HashSet::new(), &mut output);
        output
    }

    /// Returns true if all the [`Selection`]s in this selection set are themselves introspections.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        self.selection().iter().all(|selection| match selection {
            Selection::Field(field) => field.is_introspection(),
            Selection::FragmentSpread(spread) => spread.is_introspection(db),
            Selection::InlineFragment(inline) => inline.is_introspection(db),
        })
    }

    /// Create a selection set for the concatenation of two selection sets' fields.
    ///
    /// This does not deduplicate fields: if the two selection sets both select a field `a`, the
    /// merged set will select field `a` twice.
    pub fn merge(&self, other: &SelectionSet) -> SelectionSet {
        let mut merged: Vec<Selection> =
            Vec::with_capacity(self.selection.len() + other.selection.len());
        merged.append(&mut self.selection.as_ref().clone());
        merged.append(&mut other.selection.as_ref().clone());

        SelectionSet {
            selection: Arc::new(merged),
        }
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
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Selection::Field(field) => field.loc(),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.loc(),
            Selection::InlineFragment(inline_fragment) => inline_fragment.loc(),
        }
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

    /// Get the field's name, corresponding to the definition it looks up.
    ///
    /// For example, in this operation, the `.name()` is "sourceField":
    /// ```graphql
    /// query GetField { alias: sourceField }
    /// ```
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the name that will be used for this field selection in response formatting.
    ///
    /// For example, in this operation, the `.response_name()` is "sourceField":
    /// ```graphql
    /// query GetField { sourceField }
    /// ```
    ///
    /// But in this operation that uses an alias, the `.response_name()` is "responseField":
    /// ```graphql
    /// query GetField { responseField: sourceField }
    /// ```
    pub fn response_name(&self) -> &str {
        self.alias().map(Alias::name).unwrap_or_else(|| self.name())
    }

    /// Get a reference to field's type.
    pub fn ty(&self, db: &dyn HirDatabase) -> Option<Type> {
        let def = db
            .find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())?
            .field(self.name())?
            .ty()
            .to_owned();
        Some(def)
    }

    /// Get the field's parent type definition.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Get field's original field definition.
    pub fn field_definition(&self, db: &dyn HirDatabase) -> Option<FieldDefinition> {
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to the field's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Return an iterator over the variables used in arguments to this field.
    fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.arguments.iter().filter_map(|arg| match arg.value() {
            Value::Variable(var) => Some(var.clone()),
            _ => None,
        })
    }

    /// Get variables used in the field, including in sub-selections.
    ///
    /// For example, with this field:
    /// ```graphql
    /// {
    ///   field(arg: $arg) {
    ///     number(formatAs: $format)
    ///   }
    /// }
    /// ```
    /// the used variables are `$arg` and `$format`.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        let mut vars = self.self_used_variables().collect::<Vec<_>>();
        vars.extend(self.selection_set.variables(db));
        vars
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// returns true if this is an introspection field (i.e. it's [`Self::name()`] is one of __type, or __schema).
    pub fn is_introspection(&self) -> bool {
        let field_name = self.name();
        field_name == "__type" || field_name == "__schema" || field_name == "__typename"
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference inline fragment's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Get variables in use in the inline fragment.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.selection_set.variables(db)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if the inline fragment's [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        self.selection_set().is_introspection(db)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: HirNodeLocation,
}

impl FragmentSpread {
    /// Get a reference to the fragment spread's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the fragment definition this fragment spread is referencing.
    pub fn fragment(&self, db: &dyn HirDatabase) -> Option<Arc<FragmentDefinition>> {
        db.find_fragment_by_name(self.loc.file_id(), self.name().to_string())
    }

    /// Get fragment spread's defined variables.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.fragment(db)
            .map(|fragment| fragment.variables(db))
            .unwrap_or_default()
    }

    /// Get a reference to fragment spread directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if the fragment referenced by this spread exists and its [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        let maybe_fragment = self.fragment(db);
        maybe_fragment.map_or(false, |fragment| {
            fragment.selection_set.is_introspection(db)
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);
impl Alias {
    pub fn name(&self) -> &str {
        &self.0
    }
}

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
    pub(crate) extensions: Vec<Arc<SchemaExtension>>,
}

impl SchemaDefinition {
    /// Get a reference to the schema definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the `schema` definition or its extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<SchemaExtension>] {
        &self.extensions
    }

    // NOTE(@lrlna): potentially have the following fns on the database itself
    // so they are memoised as well

    /// Get Schema's query object type definition.
    pub fn query(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_ty.is_query() {
                op.object_type(db)
            } else {
                None
            }
        })
    }

    /// Get Schema's mutation object type definition.
    pub fn mutation(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_ty.is_mutation() {
                op.object_type(db)
            } else {
                None
            }
        })
    }

    /// Get Schema's subscription object type definition.
    pub fn subscription(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        self.root_operation_type_definition().iter().find_map(|op| {
            if op.operation_ty.is_subscription() {
                op.object_type(db)
            } else {
                None
            }
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootOperationTypeDefinition {
    pub(crate) operation_ty: OperationType,
    pub(crate) named_type: Type,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl RootOperationTypeDefinition {
    /// Get a reference to the root operation type definition's named type.
    pub fn named_type(&self) -> &Type {
        &self.named_type
    }

    /// Get the kind of the root operation type definition: `query`, `mutation`, or `subscription`
    pub fn operation_ty(&self) -> OperationType {
        self.operation_ty
    }

    /// Get the object type this root operation is referencing.
    pub fn object_type(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type_by_name(self.named_type().name())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }
}

impl Default for RootOperationTypeDefinition {
    fn default() -> Self {
        Self {
            operation_ty: OperationType::Query,
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
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
    pub(crate) extensions: Vec<Arc<ObjectTypeExtension>>,
}

impl ObjectTypeDefinition {
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

    /// Get a reference to the object type definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<ObjectTypeExtension>] {
        &self.extensions
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
        db: &dyn HirDatabase,
    ) -> Option<Arc<InterfaceTypeDefinition>> {
        db.find_interface_by_name(self.interface().to_string())
    }

    /// Get implements interfaces' interface name.
    pub fn interface(&self) -> &str {
        self.interface.src()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Get a reference to input value definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a refernce to inpul value definition's default_value.
    pub fn default_value(&self) -> Option<&DefaultValue> {
        self.default_value.as_ref()
    }

    /// If the argument does not have a default value and has a non-null type,
    /// a value must be provided by users.
    pub fn is_required(&self) -> bool {
        self.ty().is_non_null() && self.default_value.is_none()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarTypeDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) built_in: bool,
    pub(crate) loc: Option<HirNodeLocation>,
    pub(crate) extensions: Vec<Arc<ScalarTypeExtension>>,
}

impl ScalarTypeDefinition {
    /// Get the scalar type definition's id.

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

    /// Get a reference to scalar definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Returns true if the current scalar is a GraphQL built in.
    pub fn is_built_in(&self) -> bool {
        self.built_in
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<ScalarTypeExtension>] {
        &self.extensions
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct EnumTypeDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) enum_values_definition: Arc<Vec<EnumValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
    pub(crate) extensions: Vec<Arc<EnumTypeExtension>>,
}

impl EnumTypeDefinition {
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

    /// Get a reference to enum definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to enum definition's enum values definition vector.
    pub fn enum_values_definition(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<EnumTypeExtension>] {
        &self.extensions
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct UnionTypeDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) union_members: Arc<Vec<UnionMember>>,
    pub(crate) loc: HirNodeLocation,
    pub(crate) extensions: Vec<Arc<UnionTypeExtension>>,
}

impl UnionTypeDefinition {
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

    /// Get a reference to union definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to union definition's union members.
    pub fn union_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<UnionTypeExtension>] {
        &self.extensions
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
    pub fn object(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type_by_name(self.name().to_string())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InterfaceTypeDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: Arc<Vec<ImplementsInterface>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) fields_definition: Arc<Vec<FieldDefinition>>,
    pub(crate) loc: HirNodeLocation,
    pub(crate) extensions: Vec<Arc<InterfaceTypeExtension>>,
}

impl InterfaceTypeDefinition {
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

    /// Get a reference to the interface definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to interface definition's fields.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<InterfaceTypeExtension>] {
        &self.extensions
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InputObjectTypeDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) input_fields_definition: Arc<Vec<InputValueDefinition>>,
    pub(crate) loc: HirNodeLocation,
    pub(crate) extensions: Vec<Arc<InputObjectTypeExtension>>,
}

impl InputObjectTypeDefinition {
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

    /// Get a reference to input object definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the type definition or its type extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to input fields definitions.
    pub fn input_fields_definition(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<InputObjectTypeExtension>] {
        &self.extensions
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }
}

impl From<Name> for String {
    fn from(name: Name) -> String {
        name.src
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to the schema definition's root operation type definition.
    pub fn root_operation_type_definition(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to interface definition's fields.
    pub fn fields_definition(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_definition().iter().find(|f| f.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to union definition's union members.
    pub fn union_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to enum definition's enum values definition vector.
    pub fn enum_values_definition(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
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

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    pub fn input_fields_definition(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct HirNodeLocation {
    pub(crate) offset: usize,
    pub(crate) node_len: usize,
    pub(crate) file_id: FileId,
}

impl HirNodeLocation {
    pub(crate) fn new(file_id: FileId, node: &'_ SyntaxNode) -> Self {
        let text_range = node.text_range();
        Self {
            offset: text_range.start().into(),
            node_len: text_range.len().into(),
            file_id,
        }
    }

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

impl<Ast: ast::AstNode> From<(FileId, &'_ Ast)> for HirNodeLocation {
    fn from((file_id, node): (FileId, &'_ Ast)) -> Self {
        Self::new(file_id, node.syntax())
    }
}

#[cfg(test)]
mod tests {
    use crate::hir::OperationDefinition;
    use crate::ApolloCompiler;
    use crate::HirDatabase;
    use std::sync::Arc;

    #[test]
    fn huge_floats() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(
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
    fn extensions() {
        let mut compiler = ApolloCompiler::new();
        let first = "
            scalar URL
        ";
        let second = "
            extend scalar URL @example(arg: true)
        ";
        compiler.add_type_system(first, "first.graphql");
        compiler.add_type_system(second, "second.graphql");

        assert!(matches!(
            compiler
                .db
                .find_type_definition_by_name("URL".into())
                .unwrap()
                .directive_by_name("example")
                .unwrap()
                .argument_by_name("arg")
                .unwrap(),
            super::Value::Boolean(true)
        ));
    }

    #[test]
    fn syntax_errors() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(
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

    #[test]
    fn is_introspection_operation() {
        let query_input = r#"
            query TypeIntrospect {
              __type(name: "User") {
                name
                fields {
                  name
                  type {
                    name
                  }
                }
              }
              __schema {
                types {
                  fields {
                    name
                  }
                }
              }
            }
        "#;

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");

        let db = compiler.db;
        let type_introspect: Arc<OperationDefinition> = db
            .find_operation(query_id, Some(String::from("TypeIntrospect")))
            .expect("TypeIntrospect operation does not exist");

        assert!(type_introspect.is_introspection(&db));
    }

    #[test]
    fn is_not_introspection_operation() {
        let mutation_input = r#"
            mutation PurchaseBasket {
              buyA5Wagyu(pounds: 15) {
                submitted
              }
            }
        "#;

        let query_input = r#"
            query CheckStock {
              isKagoshimaWagyuInstock

              __schema {
                types {
                  fields {
                    name
                  }
                }
              }
            }
        "#;

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");
        let mutation_id = compiler.add_executable(mutation_input, "mutation.graphql");

        let db = compiler.db;
        let check_stock: Arc<OperationDefinition> = db
            .find_operation(query_id, Some("CheckStock".into()))
            .expect("CheckStock operation does not exist");

        let purchase_operation: Arc<OperationDefinition> = db
            .find_operation(mutation_id, Some("PurchaseBasket".into()))
            .expect("CheckStock operation does not exist");

        assert!(!check_stock.is_introspection(&db));
        assert!(!purchase_operation.is_introspection(&db));
    }

    #[test]
    fn is_introspection_deep() {
        let query_input = r#"
          query IntrospectDeepFragments {
            ...onRootTrippy
          }

          fragment onRootTrippy on Root {
             ...onRooten
          }

          fragment onRooten on Root {
            ...onRooten2

            ... on Root {
              __schema {
                types {
                  name
                }
              }
            }
          }

          fragment onRooten2 on Root {
             __type(name: "Root") {
              ...onType
            }
            ... on Root {
              __schema {
                directives {
                  name
                }
              }
            }
          }
          fragment onType on __Type {
            fields {
              name
            }
          }

          fragment onRooten2_not_intro on Root {
            species(id: "Ewok") {
              name
            }

            ... on Root {
              __schema {
                directives {
                  name
                }
              }
            }
         }
        "#;

        let query_input_not_introspect =
            query_input.replace("...onRooten2", "...onRooten2_not_intro");

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");
        let query_id_not_introspect =
            compiler.add_executable(query_input_not_introspect.as_str(), "query2.graphql");

        let db = compiler.db;
        let deep_introspect: Arc<OperationDefinition> = db
            .find_operation(query_id, Some("IntrospectDeepFragments".into()))
            .expect("IntrospectDeepFragments operation does not exist");

        assert!(deep_introspect.is_introspection(&db));

        let deep_introspect: Arc<OperationDefinition> = db
            .find_operation(
                query_id_not_introspect,
                Some("IntrospectDeepFragments".into()),
            )
            .expect("IntrospectDeepFragments operation does not exist");
        assert!(!deep_introspect.is_introspection(&db));
    }
}
