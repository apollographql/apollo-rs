use crate::hir2::Located;
use crate::hir2::LocatedBorrow;
use crate::FileId;
use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::Ranged;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::num::NonZeroU32;
use std::sync::OnceLock;

/// Results of analysis of type system definitions from any number of input files.
///
/// Information about a given type can come either from its “main” definition
/// or from an extension.
#[derive(Clone, Debug)]
pub struct TypeSystem {
    pub schema: Schema,
    pub directives: IndexMap<Name, Located<mir::DirectiveDefinition>>,
    pub types: IndexMap<mir::NamedType, Type>,
}

#[derive(Debug, Clone)]
enum ComponentIndex {
    InDefinition {
        index: u32,
    },
    InExtension {
        extension_index_plus_one: NonZeroU32,
        index: u32,
    },
}

const _: () = {
    assert!(std::mem::size_of::<ComponentIndex>() == 8);
};

#[derive(Clone, Debug)]
pub struct Schema {
    pub definition: Option<Located<mir::SchemaDefinition>>,
    pub extensions: Vec<Located<mir::SchemaExtension>>,

    /// Name of the object type for the `query` root operation
    pub query: Option<mir::NamedType>,
    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<mir::NamedType>,
    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<mir::NamedType>,
}

/// The definition of a named type, with all information from type extensions folded in
#[derive(Clone, Debug)]
pub enum Type {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}

#[derive(Clone, Debug)]
pub struct ScalarType {
    pub definition: Located<mir::ScalarTypeDefinition>,
    pub extensions: Vec<Located<mir::ScalarTypeExtension>>,
}

#[derive(Clone, Debug)]
pub struct ObjectType {
    pub definition: Located<mir::ObjectTypeDefinition>,
    pub extensions: Vec<Located<mir::ObjectTypeExtension>>,

    pub implements_interfaces: IndexSet<Name>,
    fields: IndexMap<Name, ComponentIndex>,
}

#[derive(Clone, Debug)]
pub struct InterfaceType {
    pub definition: Located<mir::InterfaceTypeDefinition>,
    pub extensions: Vec<Located<mir::InterfaceTypeExtension>>,

    pub implements_interfaces: IndexSet<Name>,
    fields: IndexMap<Name, ComponentIndex>,
}

#[derive(Clone, Debug)]
pub struct UnionType {
    pub definition: Located<mir::UnionTypeDefinition>,
    pub extensions: Vec<Located<mir::UnionTypeExtension>>,

    pub members: IndexSet<mir::NamedType>,
}

#[derive(Clone, Debug)]
pub struct EnumType {
    pub definition: Located<mir::EnumTypeDefinition>,
    pub extensions: Vec<Located<mir::EnumTypeExtension>>,

    values: IndexMap<Name, ComponentIndex>,
}

#[derive(Clone, Debug)]
pub struct InputObjectType {
    pub definition: Located<mir::InputObjectTypeDefinition>,
    pub extensions: Vec<Located<mir::InputObjectTypeExtension>>,

    values: IndexMap<Name, ComponentIndex>,
}

impl ComponentIndex {
    fn in_definition(index: usize) -> Self {
        Self::InDefinition {
            index: u32::try_from(index).unwrap(),
        }
    }

    fn in_extension(extension_index: usize, index: usize) -> Self {
        Self::InExtension {
            extension_index_plus_one: NonZeroU32::new(
                u32::try_from(extension_index)
                    .unwrap()
                    .checked_add(1)
                    .unwrap(),
            )
            .unwrap(),
            index: u32::try_from(index).unwrap(),
        }
    }

    fn get<'a, Definition, Extension: 'a, Component>(
        &self,
        definition: &'a Located<Definition>,
        definition_components: impl Fn() -> &'a [Harc<Ranged<Component>>],
        extensions: impl Fn() -> &'a [Located<Extension>],
        extension_components: impl Fn(&'a Extension) -> &'a [Harc<Ranged<Component>>],
    ) -> LocatedBorrow<'a, Component> {
        match *self {
            ComponentIndex::InDefinition { index } => {
                definition.same_file_id(&definition_components()[index as usize])
            }
            ComponentIndex::InExtension {
                extension_index_plus_one,
                index,
            } => {
                let extension_index = extension_index_plus_one.get() - 1;
                let extension = &extensions()[extension_index as usize];
                extension.same_file_id(&extension_components(extension)[index as usize])
            }
        }
    }
}

impl TypeSystem {
    pub fn new(input_files: &[(FileId, mir::Document)]) -> Self {
        static BUILT_IN_TYPES: std::sync::OnceLock<mir::Document> = std::sync::OnceLock::new();
        let built_in = BUILT_IN_TYPES.get_or_init(|| {
            let ast = apollo_parser::Parser::new(include_str!("../built_in_types.graphql")).parse();
            debug_assert_eq!(ast.errors().as_slice(), []);
            ast.into_mir()
        });
        let documents = std::iter::once((FileId::BUILT_IN, built_in))
            .chain(input_files.iter().map(|(id, doc)| (*id, doc)));
        let mut opt_schema = None;
        let mut directives = IndexMap::new();
        let mut types = IndexMap::new();
        // Clone the iterator so we can later iterate again from the start
        for (file_id, document) in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaDefinition(def) => {
                        opt_schema.get_or_insert_with(|| {
                            Schema::new(Located::with_file_id(def.clone(), file_id))
                        });
                    }
                    mir::Definition::DirectiveDefinition(def) => {
                        directives
                            .entry(def.name.clone())
                            .or_insert(Located::with_file_id(def.clone(), file_id));
                    }
                    mir::Definition::ScalarTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Scalar(ScalarType::new(Located::with_file_id(
                                def.clone(),
                                file_id,
                            ))));
                    }
                    mir::Definition::ObjectTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Object(ObjectType::new(Located::with_file_id(
                                def.clone(),
                                file_id,
                            ))));
                    }
                    mir::Definition::InterfaceTypeDefinition(def) => {
                        types.entry(def.name.clone()).or_insert(Type::Interface(
                            InterfaceType::new(Located::with_file_id(def.clone(), file_id)),
                        ));
                    }
                    mir::Definition::UnionTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Union(UnionType::new(Located::with_file_id(
                                def.clone(),
                                file_id,
                            ))));
                    }
                    mir::Definition::EnumTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Enum(EnumType::new(Located::with_file_id(
                                def.clone(),
                                file_id,
                            ))));
                    }
                    mir::Definition::InputObjectTypeDefinition(def) => {
                        types.entry(def.name.clone()).or_insert(Type::InputObject(
                            InputObjectType::new(Located::with_file_id(def.clone(), file_id)),
                        ));
                    }
                    mir::Definition::SchemaExtension(_)
                    | mir::Definition::ScalarTypeExtension(_)
                    | mir::Definition::ObjectTypeExtension(_)
                    | mir::Definition::InterfaceTypeExtension(_)
                    | mir::Definition::UnionTypeExtension(_)
                    | mir::Definition::EnumTypeExtension(_)
                    | mir::Definition::InputObjectTypeExtension(_)
                    | mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_) => todo!(),
                }
            }
        }
        for (file_id, document) in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaExtension(ext) => {
                        if let Some(schema) = &mut opt_schema {
                            schema.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::ScalarTypeExtension(ext) => {
                        if let Some(Type::Scalar(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::ObjectTypeExtension(ext) => {
                        if let Some(Type::Object(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::InterfaceTypeExtension(ext) => {
                        if let Some(Type::Interface(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::UnionTypeExtension(ext) => {
                        if let Some(Type::Union(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::EnumTypeExtension(ext) => {
                        if let Some(Type::Enum(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::InputObjectTypeExtension(ext) => {
                        if let Some(Type::InputObject(ty)) = types.get_mut(&ext.name) {
                            ty.extend(Located::with_file_id(ext.clone(), file_id))
                        }
                    }
                    mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_)
                    | mir::Definition::DirectiveDefinition(_)
                    | mir::Definition::SchemaDefinition(_)
                    | mir::Definition::ScalarTypeDefinition(_)
                    | mir::Definition::ObjectTypeDefinition(_)
                    | mir::Definition::InterfaceTypeDefinition(_)
                    | mir::Definition::UnionTypeDefinition(_)
                    | mir::Definition::EnumTypeDefinition(_)
                    | mir::Definition::InputObjectTypeDefinition(_) => todo!(),
                }
            }
        }
        Self {
            schema: opt_schema.unwrap_or_else(|| Schema::implicit(&types)),
            directives,
            types,
        }
    }

    /// Returns the type with the given name, if it is a scalar type
    pub fn get_scalar(&self, name: &str) -> Option<&ScalarType> {
        if let Some(Type::Scalar(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a object type
    pub fn get_object(&self, name: &str) -> Option<&ObjectType> {
        if let Some(Type::Object(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a interface type
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceType> {
        if let Some(Type::Interface(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a union type
    pub fn get_union(&self, name: &str) -> Option<&UnionType> {
        if let Some(Type::Union(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a enum type
    pub fn get_enum(&self, name: &str) -> Option<&EnumType> {
        if let Some(Type::Enum(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a input object type
    pub fn get_input_object(&self, name: &str) -> Option<&InputObjectType> {
        if let Some(Type::InputObject(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Return the meta-fields for a selection set.
    ///
    /// `is_root_operation` must be `Some` if and only if the selection set is the root of an operation.
    pub(crate) fn meta_field_definitions(
        is_root_operation_type: Option<mir::OperationType>,
    ) -> &'static [Located<mir::FieldDefinition>] {
        static ROOT_QUERY_FIELDS: LazyLock<[Located<mir::FieldDefinition>; 3]> =
            LazyLock::new(|| {
                [
                    // __typename: String!
                    Located::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__typename".into(),
                        arguments: Vec::new(),
                        ty: mir::Type::new_named("String").non_null(),
                        directives: Vec::new(),
                    }),
                    // __schema: __Schema!
                    Located::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__schema".into(),
                        arguments: Vec::new(),
                        ty: mir::Type::new_named("__Schema").non_null(),
                        directives: Vec::new(),
                    }),
                    // __type(name: String!): __Type
                    Located::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__type".into(),
                        arguments: vec![Harc::new(Ranged::no_location(
                            mir::InputValueDefinition {
                                description: None,
                                name: "name".into(),
                                ty: mir::Type::new_named("String").non_null(),
                                default_value: None,
                                directives: Vec::new(),
                            },
                        ))],
                        ty: mir::Type::new_named("__Type"),
                        directives: Vec::new(),
                    }),
                ]
            });
        static NON_ROOT_FIELDS: LazyLock<[Located<mir::FieldDefinition>; 1]> =
            LazyLock::new(|| {
                [
                    // __typename: String!
                    NON_ROOT_FIELDS.get()[0].clone(),
                ]
            });

        match is_root_operation_type {
            Some(mir::OperationType::Query) => ROOT_QUERY_FIELDS.get(),
            Some(mir::OperationType::Subscription) => &[],
            _ => NON_ROOT_FIELDS.get(),
        }
    }
}

macro_rules! directives_by_name {
    () => {
        pub fn directives_by_name<'def: 'name, 'name>(
            &'def self,
            name: &'name str,
        ) -> impl Iterator<Item = LocatedBorrow<'def, mir::Directive>> + 'name {
            self.directives().filter(move |dir| dir.name == name)
        }

        pub fn directive_by_name(&self, name: &str) -> Option<LocatedBorrow<'_, mir::Directive>> {
            self.directives_by_name(name).next()
        }
    };
}

macro_rules! directive_methods {
    () => {
        pub fn directives(&self) -> impl Iterator<Item = LocatedBorrow<'_, mir::Directive>> + '_ {
            self.definition
                .directives
                .iter()
                .map(|dir| self.definition.same_file_id(dir))
                .chain(
                    self.extensions
                        .iter()
                        .flat_map(|ext| ext.directives.iter().map(|dir| ext.same_file_id(dir))),
                )
        }

        directives_by_name!();
    };
}

impl Schema {
    fn new(definition: Located<mir::SchemaDefinition>) -> Schema {
        let mut schema = Schema {
            definition: None,
            extensions: Vec::new(),
            query: None,
            mutation: None,
            subscription: None,
        };
        schema.add_root_operations(&definition.root_operations);
        schema.definition = Some(definition);
        schema
    }

    fn extend(&mut self, extension: Located<mir::SchemaExtension>) {
        self.add_root_operations(&extension.root_operations);
        self.extensions.push(extension);
    }

    fn add_root_operations(&mut self, root_operations: &[(mir::OperationType, Name)]) {
        for (operation_type, object_type_name) in root_operations {
            match operation_type {
                mir::OperationType::Query => &mut self.query,
                mir::OperationType::Mutation => &mut self.mutation,
                mir::OperationType::Subscription => &mut self.subscription,
            }
            .get_or_insert_with(|| object_type_name.clone());
        }
    }

    /// Returns the name of the object type for the root operation with the given operation kind
    pub fn root_operation(&self, operation_type: mir::OperationType) -> Option<&Name> {
        match operation_type {
            mir::OperationType::Query => &self.query,
            mir::OperationType::Mutation => &self.mutation,
            mir::OperationType::Subscription => &self.subscription,
        }
        .as_ref()
    }

    fn implicit(types: &IndexMap<Name, Type>) -> Self {
        let if_has_object_type = |name: &str| {
            if let Some(Type::Object(_)) = types.get(name) {
                Some(Name::from(name))
            } else {
                None
            }
        };
        Self {
            definition: None,
            extensions: Vec::new(),
            query: if_has_object_type("Query"),
            mutation: if_has_object_type("Mutation"),
            subscription: if_has_object_type("Subscription"),
        }
    }

    pub fn directives(&self) -> impl Iterator<Item = LocatedBorrow<'_, mir::Directive>> + '_ {
        // Different implementation compared to other `.directives()` methods
        // because `self.definition` is optional here:
        self.definition
            .as_ref()
            .into_iter()
            .flat_map(|def| def.directives.iter().map(|dir| def.same_file_id(dir)))
            .chain(
                self.extensions
                    .iter()
                    .flat_map(|ext| ext.directives.iter().map(|dir| ext.same_file_id(dir))),
            )
    }

    directives_by_name!();
}

impl ScalarType {
    fn new(definition: Located<mir::ScalarTypeDefinition>) -> Self {
        Self {
            definition,
            extensions: Vec::new(),
        }
    }

    fn extend(&mut self, extension: Located<mir::ScalarTypeExtension>) {
        self.extensions.push(extension);
    }

    directive_methods!();
}

impl ObjectType {
    fn new(definition: Located<mir::ObjectTypeDefinition>) -> Self {
        let implements_interfaces = definition.implements_interfaces.iter().cloned().collect();
        let mut fields = IndexMap::new();
        for (i, field) in definition.fields.iter().enumerate() {
            fields
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_definition(i));
        }
        Self {
            definition,
            extensions: Vec::new(),
            implements_interfaces,
            fields,
        }
    }

    fn extend(&mut self, extension: Located<mir::ObjectTypeExtension>) {
        self.implements_interfaces
            .extend(extension.implements_interfaces.iter().cloned());
        let extension_index = self.extensions.len();
        for (i, field) in extension.fields.iter().enumerate() {
            self.fields
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_extension(extension_index, i));
        }
        self.extensions.push(extension);
    }

    fn field_by_index(&self, index: &ComponentIndex) -> LocatedBorrow<'_, mir::FieldDefinition> {
        index.get(
            &self.definition,
            || &self.definition.fields,
            || &self.extensions,
            |ext| &ext.fields,
        )
    }

    pub fn fields<'a>(
        &'a self,
    ) -> impl Iterator<Item = LocatedBorrow<'a, mir::FieldDefinition>> + 'a {
        self.fields.values().map(|index| self.field_by_index(index))
    }

    pub fn field_by_name(&self, name: &str) -> Option<LocatedBorrow<mir::FieldDefinition>> {
        self.fields
            .get(name)
            .map(|index| self.field_by_index(index))
    }

    directive_methods!();
}

impl InterfaceType {
    fn new(definition: Located<mir::InterfaceTypeDefinition>) -> Self {
        let implements_interfaces = definition.implements_interfaces.iter().cloned().collect();
        let mut fields = IndexMap::new();
        for (i, field) in definition.fields.iter().enumerate() {
            fields
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_definition(i));
        }
        Self {
            definition,
            extensions: Vec::new(),
            implements_interfaces,
            fields,
        }
    }

    fn extend(&mut self, extension: Located<mir::InterfaceTypeExtension>) {
        self.implements_interfaces
            .extend(extension.implements_interfaces.iter().cloned());
        let extension_index = self.extensions.len();
        for (i, field) in extension.fields.iter().enumerate() {
            self.fields
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_extension(extension_index, i));
        }
        self.extensions.push(extension);
    }

    fn field_by_index(&self, index: &ComponentIndex) -> LocatedBorrow<'_, mir::FieldDefinition> {
        index.get(
            &self.definition,
            || &self.definition.fields,
            || &self.extensions,
            |ext| &ext.fields,
        )
    }

    pub fn fields<'a>(
        &'a self,
    ) -> impl Iterator<Item = LocatedBorrow<'a, mir::FieldDefinition>> + 'a {
        self.fields.values().map(|index| self.field_by_index(index))
    }

    pub fn field_by_name(&self, name: &str) -> Option<LocatedBorrow<mir::FieldDefinition>> {
        self.fields
            .get(name)
            .map(|index| self.field_by_index(index))
    }

    directive_methods!();
}

impl UnionType {
    fn new(definition: Located<mir::UnionTypeDefinition>) -> Self {
        let members = definition.members.iter().cloned().collect();
        Self {
            definition,
            extensions: Vec::new(),
            members,
        }
    }

    fn extend(&mut self, extension: Located<mir::UnionTypeExtension>) {
        self.members.extend(extension.members.iter().cloned());
        self.extensions.push(extension);
    }

    directive_methods!();
}

impl EnumType {
    fn new(definition: Located<mir::EnumTypeDefinition>) -> Self {
        let mut values = IndexMap::new();
        for (i, value_def) in definition.values.iter().enumerate() {
            values
                .entry(value_def.value.clone())
                .or_insert_with(|| ComponentIndex::in_definition(i));
        }
        Self {
            definition,
            extensions: Vec::new(),
            values,
        }
    }

    fn extend(&mut self, extension: Located<mir::EnumTypeExtension>) {
        let extension_index = self.extensions.len();
        for (i, value_def) in extension.values.iter().enumerate() {
            self.values
                .entry(value_def.value.clone())
                .or_insert_with(|| ComponentIndex::in_extension(extension_index, i));
        }
        self.extensions.push(extension);
    }

    directive_methods!();
}

impl InputObjectType {
    fn new(definition: Located<mir::InputObjectTypeDefinition>) -> Self {
        let mut fields = IndexMap::new();
        for (i, field) in definition.fields.iter().enumerate() {
            fields
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_definition(i));
        }
        Self {
            definition,
            extensions: Vec::new(),
            values: fields,
        }
    }

    fn extend(&mut self, extension: Located<mir::InputObjectTypeExtension>) {
        let extension_index = self.extensions.len();
        for (i, field) in extension.fields.iter().enumerate() {
            self.values
                .entry(field.name.clone())
                .or_insert_with(|| ComponentIndex::in_extension(extension_index, i));
        }
        self.extensions.push(extension);
    }

    fn value_by_index(
        &self,
        index: &ComponentIndex,
    ) -> LocatedBorrow<'_, mir::InputValueDefinition> {
        index.get(
            &self.definition,
            || &self.definition.fields,
            || &self.extensions,
            |ext| &ext.fields,
        )
    }

    pub fn values<'a>(
        &'a self,
    ) -> impl Iterator<Item = LocatedBorrow<'a, mir::InputValueDefinition>> + 'a {
        self.values.values().map(|index| self.value_by_index(index))
    }

    pub fn value_by_name(&self, name: &str) -> Option<LocatedBorrow<mir::InputValueDefinition>> {
        self.values
            .get(name)
            .map(|index| self.value_by_index(index))
    }

    directive_methods!();
}

impl Type {
    /// For an object type or interface type, return the field with the given name.
    /// For other types, always returns `None`.
    pub fn field_by_name(&self, name: &str) -> Option<LocatedBorrow<'_, mir::FieldDefinition>> {
        match self {
            Type::Object(ty) => ty.field_by_name(name),
            Type::Interface(ty) => ty.field_by_name(name),
            Type::Scalar(_) | Type::Union(_) | Type::Enum(_) | Type::InputObject(_) => None,
        }
    }
}

// TODO: use `std::sync::LazyLock` when available https://github.com/rust-lang/rust/issues/109736
struct LazyLock<T> {
    value: OnceLock<T>,
    init: fn() -> T,
}

impl<T> LazyLock<T> {
    const fn new(init: fn() -> T) -> Self {
        Self {
            value: OnceLock::new(),
            init,
        }
    }

    fn get(&self) -> &T {
        self.value.get_or_init(self.init)
    }
}
