use std::sync::Arc;

use crate::{
    hir::{
        ByNameWithExtensions, Directive, FieldDefinition, HirNodeLocation, Name,
        ObjectTypeDefinition,
    },
    HirDatabase,
};

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
