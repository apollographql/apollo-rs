use std::sync::Arc;

use crate::hir::{ByNameWithExtensions, Directive, HirNodeLocation, Name};

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
