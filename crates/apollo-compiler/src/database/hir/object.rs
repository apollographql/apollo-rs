use std::sync::Arc;

use crate::{
    hir::{
        ByNameWithExtensions, Directive, FieldDefinition, HirNodeLocation, ImplementsInterface,
        Name,
    },
    HirDatabase,
};

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
