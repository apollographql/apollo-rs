use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use indexmap::IndexMap;

use crate::{hir::*, FileId, HirDatabase, Source};

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
