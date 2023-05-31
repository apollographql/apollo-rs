use std::sync::Arc;

use crate::hir::{Directive, HirNodeLocation, Name};

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
