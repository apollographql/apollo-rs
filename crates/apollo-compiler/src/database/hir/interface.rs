use std::sync::Arc;

use crate::{
    hir::{ByNameWithExtensions, Directive, FieldDefinition, HirNodeLocation, Name},
    HirDatabase,
};

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
