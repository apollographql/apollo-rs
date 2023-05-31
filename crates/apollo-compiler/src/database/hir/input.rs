use std::sync::Arc;

use crate::hir::{ByNameWithExtensions, DefaultValue, Directive, HirNodeLocation, Name, Type};

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
