use std::{
    collections::{HashMap, HashSet},
    fmt, hash,
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

    pub fn field(&self, db: &dyn HirDatabase, name: &str) -> Option<&FieldDefinition> {
        match self {
            Self::ObjectTypeDefinition(def) => def.field(db, name),
            Self::InterfaceTypeDefinition(def) => def.field(name),
            Self::UnionTypeDefinition(def) => {
                def.implicit_fields().iter().find(|f| f.name() == name)
            }
            _ => None,
        }
    }

    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Self::ObjectTypeDefinition(def) => def.loc(),
            Self::InterfaceTypeDefinition(def) => def.loc(),
            Self::UnionTypeDefinition(def) => def.loc(),
            Self::EnumTypeDefinition(def) => def.loc(),
            Self::InputObjectTypeDefinition(def) => def.loc(),
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

    /// Get the location information for the "head" of the fragment definition, namely the
    /// `fragment` keyword and the name.
    pub(crate) fn head_loc(&self) -> HirNodeLocation {
        self.name_src()
            .loc()
            .map(|name_loc| HirNodeLocation {
                // Adjust the node length to include the name
                node_len: name_loc.end_offset() - self.loc.offset(),
                ..self.loc
            })
            .unwrap_or(self.loc)
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
        let schema = db.schema();
        let name = match self.operation_ty {
            OperationType::Query => schema.query()?,
            OperationType::Mutation => schema.mutation()?,
            OperationType::Subscription => schema.subscription()?,
        };
        db.object_types_with_built_ins().get(name).cloned()
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

    /// Get all fragment definitions referenced by the operation.
    pub fn fragment_references(&self, db: &dyn HirDatabase) -> Arc<Vec<Arc<FragmentDefinition>>> {
        db.operation_fragment_references(self.selection_set.clone())
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

impl From<OperationType> for &'static str {
    fn from(ty: OperationType) -> &'static str {
        match ty {
            OperationType::Query => "Query",
            OperationType::Mutation => "Mutation",
            OperationType::Subscription => "Subscription",
        }
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).into())
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
    pub(crate) loc: HirNodeLocation,
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
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Get the location information for the "head" of the directive definition, namely the
    /// `directive` keyword and the name.
    pub(crate) fn head_loc(&self) -> HirNodeLocation {
        self.name_src()
            .loc()
            .map(|name_loc| HirNodeLocation {
                // Adjust the node length to include the name
                node_len: name_loc.end_offset() - self.loc.offset(),
                ..self.loc
            })
            .unwrap_or(self.loc)
    }

    /// Checks if current directive is one of built-in directives - `@skip`,
    /// `@include`, `@deprecated`, `@specifiedBy`.
    pub fn is_built_in(&self) -> bool {
        matches!(
            self.name(),
            "skip" | "include" | "deprecated" | "specifiedBy"
        )
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
    Int {
        value: Float,
        loc: HirNodeLocation,
    },
    Float {
        value: Float,
        loc: HirNodeLocation,
    },
    String {
        value: String,
        loc: HirNodeLocation,
    },
    Boolean {
        value: bool,
        loc: HirNodeLocation,
    },
    Null {
        loc: HirNodeLocation,
    },
    Enum {
        value: Name,
        loc: HirNodeLocation,
    },
    List {
        value: Vec<Value>,
        loc: HirNodeLocation,
    },
    Object {
        value: Vec<(Name, Value)>,
        loc: HirNodeLocation,
    },
}

impl Value {
    /// Returns `true` if `other` represents the same value as `self`. This is different from the
    /// `Eq` implementation as it ignores location information.
    pub fn is_same_value(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Variable(left), Value::Variable(right)) => left.name() == right.name(),
            (
                Value::Int { value: left, .. } | Value::Float { value: left, .. },
                Value::Int { value: right, .. } | Value::Float { value: right, .. },
            ) => left == right,
            (Value::String { value: left, .. }, Value::String { value: right, .. }) => {
                left == right
            }
            (Value::Boolean { value: left, .. }, Value::Boolean { value: right, .. }) => {
                left == right
            }
            (Value::Null { .. }, Value::Null { .. }) => true,
            (Value::Enum { value: left, .. }, Value::Enum { value: right, .. }) => {
                left.src() == right.src()
            }
            (Value::List { value: left, .. }, Value::List { value: right, .. })
                if left.len() == right.len() =>
            {
                left.iter()
                    .zip(right)
                    .all(|(left, right)| left.is_same_value(right))
            }
            (Value::Object { value: left, .. }, Value::Object { value: right, .. })
                if left.len() == right.len() =>
            {
                left.iter().zip(right).all(|(left, right)| {
                    left.0.src() == left.0.src() && left.1.is_same_value(&right.1)
                })
            }
            _ => false,
        }
    }

    /// Get current value's location.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Value::Variable(var) => var.loc(),
            Value::Int { value: _, loc } => *loc,
            Value::Float { value: _, loc } => *loc,
            Value::String { value: _, loc } => *loc,
            Value::Boolean { value: _, loc } => *loc,
            Value::Null { loc } => *loc,
            Value::Enum { value: _, loc } => *loc,
            Value::List { value: _, loc } => *loc,
            Value::Object { value: _, loc } => *loc,
        }
    }

    pub fn variables(&self) -> Vec<Variable> {
        match self {
            Value::Variable(var) => vec![var.clone()],
            Value::List {
                value: values,
                loc: _loc,
            } => values.iter().flat_map(|v| v.variables()).collect(),
            Value::Object {
                value: obj,
                loc: _loc,
            } => obj.iter().flat_map(|o| o.1.variables()).collect(),
            _ => Vec::new(),
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Value::Variable { .. } => "Variable",
            Value::Int { .. } => "Int",
            Value::Float { .. } => "Float",
            Value::String { .. } => "String",
            Value::Boolean { .. } => "Boolean",
            Value::Null { .. } => "Null",
            Value::Enum { .. } => "Enum",
            Value::List { .. } => "List",
            Value::Object { .. } => "Object",
        }
    }

    /// Returns `true` if the value is [`Variable`].
    ///
    /// [`Variable`]: Value::Variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable { .. })
    }

    /// Returns `true` if the value is [`Null`].
    ///
    /// [`Null`]: Value::Null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null { .. })
    }

    /// Returns an `i32` if the value is a number and can be represented as an i32.
    #[must_use]
    pub fn as_i32(&self) -> Option<i32> {
        i32::try_from(self).ok()
    }

    /// Returns an `f64` if the value is a number and can be represented as an f64.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        f64::try_from(self).ok()
    }

    /// Returns a `str` if the value is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&'_ str> {
        match self {
            Value::String { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns true/false if the value is a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Returns the inner list if the value is a List type.
    #[must_use]
    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns a keys/values list if the value is an input object.
    #[must_use]
    pub fn as_object(&self) -> Option<&Vec<(Name, Value)>> {
        match self {
            Value::Object { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns the [`hir::Variable`] if the value is a variable reference.
    ///
    /// [`hir::Variable`]: Variable
    #[must_use]
    pub fn as_variable(&self) -> Option<&Variable> {
        match self {
            Value::Variable(var) => Some(var),
            _ => None,
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
        if let Value::Int { value: float, .. } | Value::Float { value: float, .. } = value {
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
        if let Value::Int { value: float, .. } = value {
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
                        output.extend(spread.self_used_variables());

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
                        output.extend(inline.self_used_variables());
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
        fn is_introspection_impl(
            db: &dyn HirDatabase,
            set: &SelectionSet,
            seen_fragments: &mut HashSet<Arc<FragmentDefinition>>,
        ) -> bool {
            set.selection().iter().all(|selection| match selection {
                Selection::Field(field) => field.is_introspection(),
                Selection::FragmentSpread(spread) => {
                    let maybe_fragment = spread.fragment(db);
                    maybe_fragment.map_or(false, |fragment| {
                        if seen_fragments.contains(&fragment) {
                            false
                        } else {
                            seen_fragments.insert(Arc::clone(&fragment));
                            is_introspection_impl(db, &fragment.selection_set, seen_fragments)
                        }
                    })
                }
                Selection::InlineFragment(inline) => {
                    is_introspection_impl(db, &inline.selection_set, seen_fragments)
                }
            })
        }

        is_introspection_impl(db, self, &mut HashSet::new())
    }

    /// Create a selection set for the concatenation of two selection sets' fields.
    ///
    /// This does not deduplicate fields: if the two selection sets both select a field `a`, the
    /// merged set will select field `a` twice.
    pub fn concat(&self, other: &SelectionSet) -> SelectionSet {
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

/// Represent both kinds of fragment selections: named and inline fragments.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FragmentSelection {
    FragmentSpread(Arc<FragmentSpread>),
    InlineFragment(Arc<InlineFragment>),
}

impl FragmentSelection {
    /// Get the name of this fragment's type condition.
    ///
    /// This returns `None` on the following invalid inputs:
    /// - `self` is a named fragment spread, but the fragment it refers to is not defined
    /// - `self` is an inline fragment without an explicit type condition, used in a selection set
    ///   with a declared parent type that is not defined in the schema
    pub fn type_condition(&self, db: &dyn HirDatabase) -> Option<String> {
        match self {
            FragmentSelection::FragmentSpread(spread) => spread
                .fragment(db)
                .map(|frag| frag.type_condition().to_string()),
            FragmentSelection::InlineFragment(inline) => inline
                .type_condition()
                .or(inline.parent_obj.as_deref())
                .map(ToString::to_string),
        }
    }

    /// Get this fragment's selection set. This may be `None` if the fragment spread refers to an
    /// undefined fragment.
    pub fn selection_set(&self, db: &dyn HirDatabase) -> Option<SelectionSet> {
        match self {
            FragmentSelection::FragmentSpread(spread) => {
                spread.fragment(db).map(|frag| frag.selection_set().clone())
            }
            FragmentSelection::InlineFragment(inline) => Some(inline.selection_set().clone()),
        }
    }

    /// Get the type that this fragment is being spread onto.
    ///
    /// Returns `None` if the fragment is spread into a selection of an undefined field or type,
    /// like in:
    /// ```graphql
    /// type Query {
    ///   field: Int
    /// }
    /// query {
    ///   nonExistentField {
    ///     ... spreadToUnknownType
    ///   }
    /// }
    /// ```
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        match self {
            FragmentSelection::FragmentSpread(spread) => spread.parent_type(db),
            FragmentSelection::InlineFragment(inline) => inline.parent_type(db),
        }
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            FragmentSelection::FragmentSpread(fragment_spread) => fragment_spread.loc(),
            FragmentSelection::InlineFragment(inline_fragment) => inline_fragment.loc(),
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
            .field(db, self.name())?
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
        let type_name = self.parent_obj.as_ref()?.to_string();
        let type_def = db.find_type_definition_by_name(type_name)?;

        match type_def {
            TypeDefinition::ObjectTypeDefinition(obj) => obj.field(db, self.name()).cloned(),
            TypeDefinition::InterfaceTypeDefinition(iface) => iface.field(self.name()).cloned(),
            _ => None,
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

    /// Return an iterator over the variables used in arguments to this field and its directives.
    fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.arguments
            .iter()
            .chain(
                self.directives()
                    .iter()
                    .flat_map(|directive| directive.arguments()),
            )
            .flat_map(|arg| arg.value().variables())
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

    /// Returns true if this is an introspection field (i.e. it's
    /// [`Self::name()`] is one of __type, or __schema).
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
    pub(crate) parent_obj: Option<String>,
    pub(crate) loc: HirNodeLocation,
}

impl InlineFragment {
    /// Get a reference to inline fragment's type condition.
    pub fn type_condition(&self) -> Option<&str> {
        self.type_condition.as_ref().map(|t| t.src())
    }

    /// Get the type this fragment is spread onto.
    ///
    /// ## Examples
    /// ```graphql
    /// type Query {
    ///     field: X
    /// }
    /// query {
    ///     ... on Query { field } # spread A
    ///     field {
    ///         ... on X { subField } # spread B
    ///     }
    /// }
    /// ```
    /// `A.parent_type()` is `Query`.
    /// `B.parent_type()` is `X`.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Get inline fragments's type definition.
    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.type_condition()?.to_string())
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

    /// Return an iterator over the variables used in directives on this spread.
    ///
    /// Variables used *inside* the fragment are not included. For that, use
    /// [`variables()`][Self::variables].
    pub fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.directives()
            .iter()
            .flat_map(Directive::arguments)
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(var.clone()),
                _ => None,
            })
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
    pub(crate) parent_obj: Option<String>,
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

    /// Get the type this fragment is spread onto.
    ///
    /// ## Examples
    /// ```graphql
    /// type Query {
    ///     field: X
    /// }
    /// query {
    ///     ...fragment
    ///     field { ...subFragment }
    /// }
    /// ```
    /// `fragment.parent_type()` is `Query`.
    /// `subFragment.parent_type()` is `X`.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Return an iterator over the variables used in directives on this spread.
    ///
    /// Variables used by the fragment definition are not included. For that, use
    /// [`variables()`][Self::variables].
    pub fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.directives()
            .iter()
            .flat_map(Directive::arguments)
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(var.clone()),
                _ => None,
            })
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

    /// Returns true if the fragment referenced by this spread exists and its
    /// [`SelectionSet`] is an introspection.
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

/// This pre-computes where to find items such as fields of an object type on a
/// type extension based on the item's name.
#[derive(Clone, Debug, Eq)]
pub(crate) struct ByNameWithExtensions {
    /// `(None, i)` designates `def.example[i]`.
    /// `(Some(j), i)` designates `def.extensions[j].example[i]`.
    indices: IndexMap<String, (Option<usize>, usize)>,
}

/// Equivalent to ignoring a `ByNameWithExtensions` field in `PartialEq` for its parent struct,
/// since it is determined by other fields.
impl PartialEq for ByNameWithExtensions {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

/// Equivalent to ignoring a `ByNameWithExtensions` field in `Hash` for its parent struct,
/// since it is determined by other fields.
impl hash::Hash for ByNameWithExtensions {
    fn hash<H: hash::Hasher>(&self, _state: &mut H) {
        // do nothing
    }
}

impl ByNameWithExtensions {
    pub(crate) fn new<Item>(self_items: &[Item], name: impl Fn(&Item) -> &str) -> Self {
        let mut indices = IndexMap::new();
        for (i, item) in self_items.iter().enumerate() {
            indices.entry(name(item).to_owned()).or_insert((None, i));
        }
        ByNameWithExtensions { indices }
    }

    pub(crate) fn add_extension<Item>(
        &mut self,
        extension_index: usize,
        extension_items: &[Item],
        name: impl Fn(&Item) -> &str,
    ) {
        for (i, item) in extension_items.iter().enumerate() {
            self.indices
                .entry(name(item).to_owned())
                .or_insert((Some(extension_index), i));
        }
    }

    fn get_by_index<'a, Item, Ext>(
        &self,
        (ext, i): (Option<usize>, usize),
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item],
    ) -> &'a Item {
        let items = if let Some(j) = ext {
            extension_items(&extensions[j])
        } else {
            self_items
        };
        &items[i]
    }

    pub(crate) fn get<'a, Item, Ext>(
        &self,
        name: &str,
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item],
    ) -> Option<&'a Item> {
        let index = *self.indices.get(name)?;
        Some(self.get_by_index(index, self_items, extensions, extension_items))
    }

    pub(crate) fn iter<'a, Item, Ext>(
        &'a self,
        self_items: &'a [Item],
        extensions: &'a [Arc<Ext>],
        extension_items: impl Fn(&'a Ext) -> &'a [Item] + Copy + 'a,
    ) -> impl Iterator<Item = &'a Item> + ExactSizeIterator + DoubleEndedIterator {
        self.indices
            .values()
            .map(move |&index| self.get_by_index(index, self_items, extensions, extension_items))
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Default, Eq)]
pub struct SchemaDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
    pub(crate) loc: Option<HirNodeLocation>,
    pub(crate) extensions: Vec<Arc<SchemaExtension>>,
    pub(crate) root_operation_names: RootOperationNames,
}

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct RootOperationNames {
    pub(crate) query: Option<String>,
    pub(crate) mutation: Option<String>,
    pub(crate) subscription: Option<String>,
}

impl SchemaDefinition {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

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

    /// Returns the root operations from this schema definition,
    /// excluding those from schema extensions.
    pub fn self_root_operations(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Returns an iterator of root operations, from either on this schema defintion or its extensions.
    pub fn root_operations(&self) -> impl Iterator<Item = &RootOperationTypeDefinition> {
        self.self_root_operations().iter().chain(
            self.extensions()
                .iter()
                .flat_map(|ext| ext.root_operations()),
        )
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<SchemaExtension>] {
        &self.extensions
    }

    /// Returns the name of the object type for the `query` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn query(&self) -> Option<&str> {
        self.root_operation_names.query.as_deref()
    }

    /// Returns the name of the object type for the `mutation` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn mutation(&self) -> Option<&str> {
        self.root_operation_names.mutation.as_deref()
    }

    /// Returns the name of the object type for the `subscription` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn subscription(&self) -> Option<&str> {
        self.root_operation_names.subscription.as_deref()
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
    pub(crate) fields_by_name: ByNameWithExtensions,
    pub(crate) implements_interfaces_by_name: ByNameWithExtensions,
    pub(crate) is_introspection: bool,
    pub(crate) implicit_fields: Arc<Vec<FieldDefinition>>,
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

    /// Get a reference to the object type definition's field definitions,
    /// excluding fields from extensions.
    pub fn self_fields(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Returns an iterator of fields of this object type,
    /// whether from its own definition or from extensions.
    pub fn fields(
        &self,
    ) -> impl Iterator<Item = &FieldDefinition> + ExactSizeIterator + DoubleEndedIterator {
        self.fields_by_name.iter(
            self.self_fields(),
            self.extensions(),
            ObjectTypeExtension::fields,
        )
    }

    /// Find a field by its name, either in this object type definition or its extensions.
    pub fn field(&self, db: &dyn HirDatabase, name: &str) -> Option<&FieldDefinition> {
        self.fields_by_name
            .get(
                name,
                self.self_fields(),
                self.extensions(),
                ObjectTypeExtension::fields,
            )
            .or_else(|| self.implicit_fields(db).iter().find(|f| f.name() == name))
    }

    /// Returns interfaces implemented by this object type definition,
    /// excluding those from extensions.
    pub fn self_implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    /// Returns an iterator of interfaces implemented by this object type,
    /// whether from its own definition or from extensions.
    pub fn implements_interfaces(
        &self,
    ) -> impl Iterator<Item = &ImplementsInterface> + ExactSizeIterator + DoubleEndedIterator {
        self.implements_interfaces_by_name.iter(
            self.self_implements_interfaces(),
            self.extensions(),
            ObjectTypeExtension::implements_interfaces,
        )
    }

    /// Returns whether this object type implements the interface of the given name,
    /// either in its own definition or its extensions.
    pub fn implements_interface(&self, name: &str) -> bool {
        self.implements_interfaces_by_name
            .get(
                name,
                self.self_implements_interfaces(),
                self.extensions(),
                ObjectTypeExtension::implements_interfaces,
            )
            .is_some()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<ObjectTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<ObjectTypeExtension>) {
        let next_index = self.extensions.len();
        self.fields_by_name
            .add_extension(next_index, ext.fields(), FieldDefinition::name);
        self.implements_interfaces_by_name.add_extension(
            next_index,
            ext.implements_interfaces(),
            ImplementsInterface::interface,
        );
        self.extensions.push(ext);
    }

    /// Returns `true` if this Object Type Definition is one of the
    /// introspection types:
    ///
    /// `__Schema`, `__Type`, `__Field`, `__InputValue`,
    /// `__EnumValue`, `__Directive`
    pub fn is_introspection(&self) -> bool {
        self.is_introspection
    }

    pub(crate) fn implicit_fields(&self, db: &dyn HirDatabase) -> &[FieldDefinition] {
        let is_root_query = db
            .schema()
            .root_operations()
            .any(|op| op.operation_ty().is_query() && op.named_type().name() == self.name());
        if is_root_query {
            self.implicit_fields.as_ref()
        } else {
            let position = self
                .implicit_fields
                .iter()
                .cloned()
                .position(|f| f.name() == "__typename")
                .unwrap();
            &self.implicit_fields[position..position + 1]
        }
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
    pub(crate) loc: Option<HirNodeLocation>,
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
    pub fn loc(&self) -> Option<HirNodeLocation> {
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

    /// Return the directives used on this input value definition.
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
    pub(crate) loc: HirNodeLocation,
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<ScalarTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<ScalarTypeExtension>) {
        self.extensions.push(ext);
    }

    /// Returns true if the current scalar is a GraphQL built in.
    pub fn is_built_in(&self) -> bool {
        self.built_in
    }

    /// Returns true if the current scalar is the built in Int type.
    pub fn is_int(&self) -> bool {
        self.name() == "Int" && self.built_in
    }

    /// Returns true if the current scalar is the built in Boolean type.
    pub fn is_boolean(&self) -> bool {
        self.name() == "Boolean" && self.built_in
    }

    /// Returns true if the current scalar is the built in String type.
    pub fn is_string(&self) -> bool {
        self.name() == "String" && self.built_in
    }

    /// Returns true if the current scalar is the built in Float type.
    pub fn is_float(&self) -> bool {
        self.name() == "Float" && self.built_in
    }

    /// Returns true if the current scalar is the built in ID type.
    pub fn is_id(&self) -> bool {
        self.name() == "ID" && self.built_in
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
    pub(crate) values_by_name: ByNameWithExtensions,
    pub(crate) is_introspection: bool,
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

    /// Returns the values of this enum definition, excluding those from extensions.
    pub fn self_values(&self) -> &[EnumValueDefinition] {
        self.enum_values_definition.as_ref()
    }

    /// Returns an iterator of values of this enum type,
    /// whether from its own definition or from extensions.
    pub fn values(
        &self,
    ) -> impl Iterator<Item = &EnumValueDefinition> + ExactSizeIterator + DoubleEndedIterator {
        self.values_by_name.iter(
            self.self_values(),
            self.extensions(),
            EnumTypeExtension::values,
        )
    }

    /// Find an enum value by its name, either in this enum type definition or its extensions.
    pub fn value(&self, name: &str) -> Option<&EnumValueDefinition> {
        self.values_by_name.get(
            name,
            self.self_values(),
            self.extensions(),
            EnumTypeExtension::values,
        )
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<EnumTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<EnumTypeExtension>) {
        let next_index = self.extensions.len();
        self.values_by_name.add_extension(
            next_index,
            ext.values(),
            EnumValueDefinition::enum_value,
        );
        self.extensions.push(ext);
    }

    /// Returns `true` if this Object Type Definition is one of the
    /// introspection types:
    ///
    /// `__TypeKind`, `__DirectiveLocation`
    pub fn is_introspection(&self) -> bool {
        self.is_introspection
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
    /// Get a reference to enum value description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }
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
    pub(crate) members_by_name: ByNameWithExtensions,
    pub(crate) implicit_fields: Arc<Vec<FieldDefinition>>,
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

    /// Get a reference to union definition's union members,
    /// excluding those from extensions.
    pub fn self_members(&self) -> &[UnionMember] {
        self.union_members.as_ref()
    }

    /// Returns an iterator of members of this union type,
    /// whether from its own definition or from extensions.
    pub fn members(
        &self,
    ) -> impl Iterator<Item = &UnionMember> + ExactSizeIterator + DoubleEndedIterator {
        self.members_by_name.iter(
            self.self_members(),
            self.extensions(),
            UnionTypeExtension::members,
        )
    }

    /// Returns whether the type of the given name is a member of this union type,
    /// either from the union type definition or its extensions.
    pub fn has_member(&self, name: &str) -> bool {
        self.members_by_name
            .get(
                name,
                self.self_members(),
                self.extensions(),
                UnionTypeExtension::members,
            )
            .is_some()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<UnionTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<UnionTypeExtension>) {
        let next_index = self.extensions.len();
        self.members_by_name
            .add_extension(next_index, ext.members(), UnionMember::name);
        self.extensions.push(ext);
    }

    pub(crate) fn implicit_fields(&self) -> &[FieldDefinition] {
        self.implicit_fields.as_ref()
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
    pub(crate) fields_by_name: ByNameWithExtensions,
    pub(crate) implements_interfaces_by_name: ByNameWithExtensions,
    pub(crate) implicit_fields: Arc<Vec<FieldDefinition>>,
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

    /// Returns interfaces implemented by this interface type definition,
    /// excluding those from extensions.
    pub fn self_implements_interfaces(&self) -> &[ImplementsInterface] {
        self.implements_interfaces.as_ref()
    }

    /// Returns an iterator of interfaces implemented by this interface type,
    /// whether from its own definition or from extensions.
    pub fn implements_interfaces(
        &self,
    ) -> impl Iterator<Item = &ImplementsInterface> + ExactSizeIterator + DoubleEndedIterator {
        self.implements_interfaces_by_name.iter(
            self.self_implements_interfaces(),
            self.extensions(),
            InterfaceTypeExtension::implements_interfaces,
        )
    }

    /// Returns whether this interface type implements the interface of the given name,
    /// either in its own definition or its extensions.
    pub fn implements_interface(&self, name: &str) -> bool {
        self.implements_interfaces_by_name
            .get(
                name,
                self.self_implements_interfaces(),
                self.extensions(),
                InterfaceTypeExtension::implements_interfaces,
            )
            .is_some()
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

    /// Get a reference to interface definition's fields,
    /// excluding those from extensions.
    pub fn self_fields(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Returns an iterator of fields of this interface type,
    /// whether from its own definition or from extensions.
    pub fn fields(
        &self,
    ) -> impl Iterator<Item = &FieldDefinition> + ExactSizeIterator + DoubleEndedIterator {
        self.fields_by_name.iter(
            self.self_fields(),
            self.extensions(),
            InterfaceTypeExtension::fields,
        )
    }

    /// Find a field by its name, either in this interface type definition or its extensions.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields_by_name
            .get(
                name,
                self.self_fields(),
                self.extensions(),
                InterfaceTypeExtension::fields,
            )
            .or_else(|| self.implicit_fields().iter().find(|f| f.name() == name))
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<InterfaceTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<InterfaceTypeExtension>) {
        let next_index = self.extensions.len();
        self.fields_by_name
            .add_extension(next_index, ext.fields(), FieldDefinition::name);
        self.implements_interfaces_by_name.add_extension(
            next_index,
            ext.implements_interfaces(),
            ImplementsInterface::interface,
        );
        self.extensions.push(ext);
    }

    pub(crate) fn implicit_fields(&self) -> &[FieldDefinition] {
        self.implicit_fields.as_ref()
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
    pub(crate) input_fields_by_name: ByNameWithExtensions,
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

    /// Get a reference to input fields definitions,
    /// excluding those from extensions.
    pub fn self_fields(&self) -> &[InputValueDefinition] {
        self.input_fields_definition.as_ref()
    }

    /// Returns an iterator of fields of this input object type,
    /// whether from its own definition or from extensions.
    pub fn fields(
        &self,
    ) -> impl Iterator<Item = &InputValueDefinition> + ExactSizeIterator + DoubleEndedIterator {
        self.input_fields_by_name.iter(
            self.self_fields(),
            self.extensions(),
            InputObjectTypeExtension::fields,
        )
    }

    /// Find a field by its name, either in this input object type definition or its extensions.
    pub fn field(&self, name: &str) -> Option<&InputValueDefinition> {
        self.input_fields_by_name.get(
            name,
            self.self_fields(),
            self.extensions(),
            InputObjectTypeExtension::fields,
        )
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<InputObjectTypeExtension>] {
        &self.extensions
    }

    pub(crate) fn push_extension(&mut self, ext: Arc<InputObjectTypeExtension>) {
        let next_index = self.extensions.len();
        self.input_fields_by_name.add_extension(
            next_index,
            ext.fields(),
            InputValueDefinition::name,
        );
        self.extensions.push(ext);
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
    pub fn root_operations(&self) -> &[RootOperationTypeDefinition] {
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
    pub fn fields(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in object type definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields().iter().find(|f| f.name() == name)
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
    pub fn fields(&self) -> &[FieldDefinition] {
        self.fields_definition.as_ref()
    }

    /// Find a field in interface face definition.
    pub fn field(&self, name: &str) -> Option<&FieldDefinition> {
        self.fields().iter().find(|f| f.name() == name)
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
    pub(crate) members_by_name: ByNameWithExtensions,
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
    pub fn members(&self) -> &[UnionMember] {
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
    pub fn values(&self) -> &[EnumValueDefinition] {
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

    pub fn fields(&self) -> &[InputValueDefinition] {
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

    /// Get the source offset of the end of the current node.
    pub fn end_offset(&self) -> usize {
        self.offset + self.node_len
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
    fn root_operations() {
        let mut compiler = ApolloCompiler::new();
        let first = r#"
            schema @core(feature: "https://specs.apollo.dev/core/v0.1")
            type Query {
                field: Int
            }
            type Subscription {
                newsletter: [String]
            }
        "#;
        let second = r#"
            extend schema @core(feature: "https://specs.apollo.dev/join/v0.1") {
                query: MyQuery
            }
            type MyQuery {
                different_field: String
            }
        "#;
        compiler.add_type_system(first, "first.graphql");
        compiler.add_type_system(second, "second.graphql");

        let schema = compiler.db.schema();
        assert_eq!(
            schema
                .self_root_operations()
                .iter()
                .map(|op| op.named_type().name())
                .collect::<Vec<_>>(),
            ["Subscription"]
        );
        assert_eq!(
            schema
                .root_operations()
                .map(|op| op.named_type().name())
                .collect::<Vec<_>>(),
            ["Subscription", "MyQuery"]
        );
        assert!(schema.mutation().is_none());
        assert_eq!(schema.query().unwrap(), "MyQuery");
        assert_eq!(schema.subscription().unwrap(), "Subscription");
    }

    #[test]
    fn extensions() {
        let mut compiler = ApolloCompiler::new();
        let first = r#"
            scalar Scalar @specifiedBy(url: "https://apollographql.com")
            type Object implements Intf {
                field: Int,
            }
            type Object2 {
                field: String,
            }
            interface Intf {
                field: Int,
            }
            input Input {
                field: Enum,
            }
            enum Enum {
                VALUE,
            }
            union Union = Object | Object2;
        "#;
        let second = r#"
            extend scalar Scalar @deprecated(reason: "do something else")
            extend interface Intf implements Intf2 {
                field2: Scalar,
            }
            interface Intf2 {
                field3: String,
            }
            extend type Object implements Intf2 {
                field2: Scalar,
                field3: String,
            }
            extend enum Enum {
                "like VALUE, but more"
                VALUE_2,
            }
            extend input Input {
                field2: Int,
            }
            extend union Union = Query;
            type Query {
                object: Object,
            }
        "#;
        compiler.add_type_system(first, "first.graphql");
        compiler.add_type_system(second, "second.graphql");

        let scalar = &compiler.db.types_definitions_by_name()["Scalar"];
        let object = &compiler.db.object_types()["Object"];
        let interface = &compiler.db.interfaces()["Intf"];
        let input = &compiler.db.input_objects()["Input"];
        let enum_ = &compiler.db.enums()["Enum"];
        let union_ = &compiler.db.unions()["Union"];

        assert_eq!(
            scalar
                .self_directives()
                .iter()
                .map(|d| d.name())
                .collect::<Vec<_>>(),
            ["specifiedBy"]
        );
        assert_eq!(
            scalar.directives().map(|d| d.name()).collect::<Vec<_>>(),
            ["specifiedBy", "deprecated"]
        );
        // assert_eq!(
        //     *scalar
        //         .directive_by_name("deprecated")
        //         .unwrap()
        //         .argument_by_name("reason")
        //         .unwrap(),
        //     super::Value::String("do something else".to_owned())
        // );
        assert!(scalar.directive_by_name("haunted").is_none());

        assert_eq!(
            object
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            object.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2", "field3"]
        );
        assert_eq!(
            object.field(&compiler.db, "field").unwrap().ty().name(),
            "Int"
        );
        assert!(object.field(&compiler.db, "field4").is_none());

        assert_eq!(
            object
                .self_implements_interfaces()
                .iter()
                .map(|i| i.interface())
                .collect::<Vec<_>>(),
            ["Intf"]
        );
        assert_eq!(
            object
                .implements_interfaces()
                .map(|f| f.interface())
                .collect::<Vec<_>>(),
            ["Intf", "Intf2"]
        );
        assert!(object.implements_interface("Intf2"));
        assert!(!object.implements_interface("Intf3"));

        assert_eq!(
            interface
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            interface.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2"]
        );
        assert_eq!(interface.field("field").unwrap().ty().name(), "Int");
        assert!(interface.field("field4").is_none());

        assert!(interface.self_implements_interfaces().is_empty());
        assert_eq!(
            interface
                .implements_interfaces()
                .map(|f| f.interface())
                .collect::<Vec<_>>(),
            ["Intf2"]
        );
        assert!(interface.implements_interface("Intf2"));
        assert!(!interface.implements_interface("Intf3"));

        assert_eq!(
            input
                .self_fields()
                .iter()
                .map(|f| f.name())
                .collect::<Vec<_>>(),
            ["field"]
        );
        assert_eq!(
            input.fields().map(|f| f.name()).collect::<Vec<_>>(),
            ["field", "field2"]
        );
        assert_eq!(input.field("field").unwrap().ty().name(), "Enum");
        assert!(input.field("field3").is_none());

        assert_eq!(
            enum_
                .self_values()
                .iter()
                .map(|v| v.enum_value())
                .collect::<Vec<_>>(),
            ["VALUE"]
        );
        assert_eq!(
            enum_.values().map(|v| v.enum_value()).collect::<Vec<_>>(),
            ["VALUE", "VALUE_2"]
        );
        assert_eq!(
            enum_.value("VALUE_2").unwrap().description(),
            Some("like VALUE, but more")
        );
        assert!(enum_.value("VALUE_3").is_none());

        assert_eq!(
            union_
                .self_members()
                .iter()
                .map(|m| m.name())
                .collect::<Vec<_>>(),
            ["Object", "Object2"]
        );
        assert_eq!(
            union_.members().map(|m| m.name()).collect::<Vec<_>>(),
            ["Object", "Object2", "Query"]
        );
        assert!(union_.has_member("Object2"));
        assert!(!union_.has_member("Enum"));
    }

    #[test]
    fn query_extended_type() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system("type Query { foo: String }", "base.graphql");
        compiler.add_type_system("extend type Query { bar: Int }", "ext.graphql");
        compiler.add_executable("{ bar }", "query.graphql");
        let operations = compiler.db.all_operations();
        let fields = operations[0].fields(&compiler.db);
        // This unwrap failed before https://github.com/apollographql/apollo-rs/pull/482
        // changed the behavior of `ObjectTypeDefinition::field(name)` in `hir_db::parent_ty`
        let ty = fields[0].ty(&compiler.db).unwrap();
        assert_eq!(ty.name(), "Int");
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

    #[test]
    fn introspection_field_types() {
        let input = r#"
type Query {
  id: String
  name: String
  birthday: Date
}

scalar Date @specifiedBy(url: "datespec.com")

{
  __type(name: "User") {
    name
    fields {
      name
      type {
        name
      }
    }
  }
}
        "#;
        let mut compiler = ApolloCompiler::new();
        let file_id = compiler.add_type_system(input, "ts.graphql");

        let diagnostics = compiler.validate();
        assert!(diagnostics.is_empty());

        let db = compiler.db;
        let op = db.find_operation(file_id, None).unwrap();
        let ty_field = op
            .selection_set()
            .field("__type")
            .unwrap()
            .ty(&db)
            .unwrap()
            .name();

        assert_eq!(ty_field, "__Type");
    }

    #[test]
    fn built_in_types() {
        let input = r#"
type Query {
  id: String
  name: String
  birthday: Date
}
        "#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input, "ts.graphql");
        let db = compiler.db;

        // introspection types
        assert!(db
            .find_object_type_by_name("__Schema".to_string())
            .is_some());
        assert!(db.find_object_type_by_name("__Type".to_string()).is_some());
        assert!(db.find_enum_by_name("__TypeKind".to_string()).is_some());
        assert!(db.find_object_type_by_name("__Field".to_string()).is_some());
        assert!(db
            .find_object_type_by_name("__InputValue".to_string())
            .is_some());
        assert!(db
            .find_object_type_by_name("__EnumValue".to_string())
            .is_some());
        assert!(db
            .find_object_type_by_name("__Directive".to_string())
            .is_some());
        assert!(db
            .find_enum_by_name("__DirectiveLocation".to_string())
            .is_some());

        // scalar types
        assert!(db.find_scalar_by_name("Int".to_string()).is_some());
        assert!(db.find_scalar_by_name("Float".to_string()).is_some());
        assert!(db.find_scalar_by_name("Boolean".to_string()).is_some());
        assert!(db.find_scalar_by_name("String".to_string()).is_some());
        assert!(db.find_scalar_by_name("ID".to_string()).is_some());

        // directive definitions
        assert!(db
            .find_directive_definition_by_name("specifiedBy".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("skip".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("include".to_string())
            .is_some());
        assert!(db
            .find_directive_definition_by_name("deprecated".to_string())
            .is_some());
    }

    #[test]
    fn built_in_types_in_type_system_hir() {
        let mut compiler_1 = ApolloCompiler::new();
        compiler_1.add_type_system("type Query { unused: Int }", "unused.graphql");

        let mut compiler_2 = ApolloCompiler::new();
        compiler_2.set_type_system_hir(compiler_1.db.type_system());
        assert!(compiler_2
            .db
            .object_types_with_built_ins()
            .contains_key("__Schema"));
        assert!(compiler_2
            .db
            .enums_with_built_ins()
            .contains_key("__TypeKind"));
    }

    #[test]
    fn field_definition() {
        let input = r#"
schema {
  query Query
}

type Query {
  foo: String
  creature: Creature
}

interface Creature {
  name: String
}

query {
  foo
  creature {
    name
  }
}
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "test.graphql");
        let db = &compiler.db;
        let all_ops = db.all_operations();
        let default_query_op = all_ops
            .iter()
            .find(|op| op.name().is_none())
            .expect("default query not found");

        let sel_set = default_query_op.selection_set();
        let query_type = default_query_op
            .object_type(&compiler.db)
            .expect("query type not found");

        let sel_foo_field_def = sel_set
            .field("foo")
            .expect("query.foo selection field not found")
            .field_definition(db)
            .expect("field_definition returned none for query.foo");

        let query_foo_field_def = query_type
            .field(db, "foo")
            .expect("foo field not found on query type");

        // assert that field_definition() returns a field def for object types
        assert_eq!(&sel_foo_field_def, query_foo_field_def);

        let creature_type = db
            .find_interface_by_name("Creature".to_owned())
            .expect("creature type not found");

        let sel_creature_name_field_def = sel_set
            .field("creature")
            .expect("creature field not found on query selection")
            .selection_set()
            .field("name")
            .expect("name field not found on creature selection")
            .field_definition(db)
            .expect("field definition not found on creature.name selection");

        let hir_creature_field_def = creature_type
            .field("name")
            .expect("name field not found on creature type");

        // assert that field_definition() also returns a field def for interface types
        assert_eq!(hir_creature_field_def, &sel_creature_name_field_def)
    }

    #[test]
    fn values() {
        let mut compiler = ApolloCompiler::new();
        let input = r#"
            query ($arg: Int!) {
                field(
                    float: 1.234,
                    int: 1234,
                    string: "some text",
                    bool: true,
                    variable: $arg,
                )
            }
        "#;
        let id = compiler.add_executable(input, "test.graphql");
        let op = compiler.db.find_operation(id, None).unwrap();
        let field = &op.fields(&compiler.db)[0];

        let args = field.arguments();
        assert_eq!(args[0].value.as_f64(), Some(1.234));
        assert_eq!(args[0].value.as_i32(), None);
        assert_eq!(args[0].value.as_str(), None);
        assert_eq!(args[1].value.as_i32(), Some(1234));
        assert_eq!(args[1].value.as_f64(), Some(1234.0));
        assert_eq!(args[1].value.as_str(), None);
        assert_eq!(args[2].value.as_str(), Some("some text"));
        assert_eq!(args[2].value.as_bool(), None);
        assert_eq!(args[2].value.as_i32(), None);
        assert_eq!(args[3].value.as_bool(), Some(true));
        assert_eq!(args[3].value.as_f64(), None);
        assert_eq!(args[3].value.as_i32(), None);
        assert!(args[4].value.as_variable().is_some());
        assert!(args[4].value.as_bool().is_none());
        assert!(args[4].value.as_f64().is_none());
        assert!(args[4].value.as_i32().is_none());
    }
}
