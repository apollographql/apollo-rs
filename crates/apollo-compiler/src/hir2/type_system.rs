use crate::hir2::Component;
use crate::hir2::ExtensionId;
use crate::hir2::Located;
use crate::hir2::LocatedBorrow;
use crate::FileId;
use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::Ranged;
use apollo_parser::BowString;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::sync::OnceLock;

/// Results of analysis of type system definitions from any number of input files.
///
/// Information about a given type can come either from its “main” definition
/// or from an extension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypeSystem {
    /// The source location is that of the "main" definition (if any).
    pub schema: Located<Schema>,
    pub directives: IndexMap<Name, Located<mir::DirectiveDefinition>>,
    pub types: IndexMap<mir::NamedType, Type>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub description: Option<BowString>,
    pub directives: Vec<Component<mir::Directive>>,

    /// Name of the object type for the `query` root operation
    pub query: Option<mir::NamedType>,
    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<mir::NamedType>,
    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<mir::NamedType>,

    /// Which schema extension (if any) defined the query root operation.
    /// Only meaningful when `self.query` is `Some`.
    pub query_extension: Option<ExtensionId>,
    /// Which schema extension (if any) defined the mutation root operation.
    /// Only meaningful when `self.mutation` is `Some`.
    pub mutation_extension: Option<ExtensionId>,
    /// Which schema extension (if any) defined the subscription root operation.
    /// Only meaningful when `self.subscription` is `Some`.
    pub subscription_extension: Option<ExtensionId>,
}

/// The definition of a named type, with all information from type extensions folded in.
///
/// The source location is that of the "main" definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(Located<ScalarType>),
    Object(Located<ObjectType>),
    Interface(Located<InterfaceType>),
    Union(Located<UnionType>),
    Enum(Located<EnumType>),
    InputObject(Located<InputObjectType>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct ScalarType {
    pub description: Option<BowString>,
    pub directives: Vec<Component<mir::Directive>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ObjectType {
    pub description: Option<BowString>,

    /// * Keys: name of the implemented interface
    /// * Values: which object type extension defined this implementation,
    ///   or `None` for the object type definition.
    pub implements_interfaces: IndexMap<Name, Option<ExtensionId>>,

    pub directives: Vec<Component<mir::Directive>>,

    pub fields: IndexMap<Name, Component<mir::FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceType {
    pub description: Option<BowString>,

    /// * Key: name of an implemented interface
    /// * Value: which interface type extension defined this implementation,
    ///   or `None` for the interface type definition.
    pub implements_interfaces: IndexMap<Name, Option<ExtensionId>>,

    pub directives: Vec<Component<mir::Directive>>,

    pub fields: IndexMap<Name, Component<mir::FieldDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnionType {
    pub description: Option<BowString>,
    pub directives: Vec<Component<mir::Directive>>,

    /// * Key: name of a member object type
    /// * Value: which union type extension defined this implementation,
    ///   or `None` for the union type definition.
    pub members: IndexMap<mir::NamedType, Option<ExtensionId>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EnumType {
    pub description: Option<BowString>,
    pub directives: Vec<Component<mir::Directive>>,
    pub values: IndexMap<Name, Component<mir::EnumValueDefinition>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InputObjectType {
    pub description: Option<BowString>,
    pub directives: Vec<Component<mir::Directive>>,
    pub fields: IndexMap<Name, Component<mir::InputValueDefinition>>,
}

/// Like `IndexMap::insert`, but does not replace the value if an equivalent key is already in the map.
fn insert_sticky<K, V>(map: &mut IndexMap<K, V>, key: impl Into<K>, make_value: impl FnOnce() -> V)
where
    K: std::hash::Hash + Eq,
{
    map.entry(key.into()).or_insert_with(make_value);
}

/// Like `IndexMap::extend`, but does not replace a value if an equivalent key is already in the map.
fn extend_sticky<K, V>(map: &mut IndexMap<K, V>, iter: impl IntoIterator<Item = (impl Into<K>, V)>)
where
    K: std::hash::Hash + Eq,
{
    for (key, value) in iter.into_iter() {
        map.entry(key.into()).or_insert(value);
    }
}

/// Like `IndexMap::from_iterator`, but does not replace a value if an equivalent key is already in the map.
fn collect_sticky<K, V>(iter: impl IntoIterator<Item = (impl Into<K>, V)>) -> IndexMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    let mut map = IndexMap::new();
    extend_sticky(&mut map, iter);
    map
}

impl TypeSystem {
    pub fn from_mir(input_files: &[(FileId, &mir::Document)]) -> Self {
        static BUILT_IN_TYPES: std::sync::OnceLock<mir::Document> = std::sync::OnceLock::new();
        let built_in = BUILT_IN_TYPES.get_or_init(|| {
            let ast = apollo_parser::Parser::new(include_str!("../built_in_types.graphql")).parse();
            debug_assert_eq!(ast.errors().as_slice(), []);
            ast.into_mir()
        });
        let documents = std::iter::once((FileId::BUILT_IN, built_in))
            .chain(input_files.iter().map(|(id, doc)| (*id, *doc)));
        let mut opt_schema = None;
        let mut directives = IndexMap::new();
        let mut types = IndexMap::new();
        // Clone the iterator so we can later iterate again from the start
        for (file_id, document) in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaDefinition(def) => {
                        opt_schema.get_or_insert_with(|| {
                            Schema::from_mir(LocatedBorrow::with_file_id(def, file_id))
                        });
                    }
                    mir::Definition::DirectiveDefinition(def) => {
                        insert_sticky(&mut directives, &def.name, || {
                            Located::with_file_id(def.clone(), file_id)
                        })
                    }
                    mir::Definition::ScalarTypeDefinition(def) => {
                        insert_sticky(&mut types, &def.name, || {
                            Type::Scalar(ScalarType::from_mir(LocatedBorrow::with_file_id(
                                def, file_id,
                            )))
                        });
                    }
                    mir::Definition::ObjectTypeDefinition(def) => {
                        insert_sticky(&mut types, &def.name, || {
                            Type::Object(ObjectType::from_mir(LocatedBorrow::with_file_id(
                                def, file_id,
                            )))
                        });
                    }
                    mir::Definition::InterfaceTypeDefinition(def) => {
                        types.entry(def.name.clone()).or_insert(Type::Interface(
                            InterfaceType::from_mir(LocatedBorrow::with_file_id(def, file_id)),
                        ));
                    }
                    mir::Definition::UnionTypeDefinition(def) => {
                        insert_sticky(&mut types, &def.name, || {
                            Type::Union(UnionType::from_mir(LocatedBorrow::with_file_id(
                                def, file_id,
                            )))
                        });
                    }
                    mir::Definition::EnumTypeDefinition(def) => {
                        insert_sticky(&mut types, &def.name, || {
                            Type::Enum(EnumType::from_mir(LocatedBorrow::with_file_id(
                                def, file_id,
                            )))
                        });
                    }
                    mir::Definition::InputObjectTypeDefinition(def) => {
                        insert_sticky(&mut types, &def.name, || {
                            Type::InputObject(InputObjectType::from_mir(
                                LocatedBorrow::with_file_id(def, file_id),
                            ))
                        });
                    }
                    mir::Definition::SchemaExtension(_)
                    | mir::Definition::ScalarTypeExtension(_)
                    | mir::Definition::ObjectTypeExtension(_)
                    | mir::Definition::InterfaceTypeExtension(_)
                    | mir::Definition::UnionTypeExtension(_)
                    | mir::Definition::EnumTypeExtension(_)
                    | mir::Definition::InputObjectTypeExtension(_) => {
                        // Extensions are handled separately below.
                    }
                    mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_) => {
                        // Operation-only definitions are not relevant to the type system.
                    }
                }
            }
        }
        for (file_id, document) in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaExtension(ext) => {
                        if let Some(schema) = &mut opt_schema {
                            schema
                                .make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::ScalarTypeExtension(ext) => {
                        if let Some(Type::Scalar(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::ObjectTypeExtension(ext) => {
                        if let Some(Type::Object(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::InterfaceTypeExtension(ext) => {
                        if let Some(Type::Interface(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::UnionTypeExtension(ext) => {
                        if let Some(Type::Union(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::EnumTypeExtension(ext) => {
                        if let Some(Type::Enum(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::InputObjectTypeExtension(ext) => {
                        if let Some(Type::InputObject(ty)) = types.get_mut(&ext.name) {
                            ty.make_mut()
                                .extend_mir(LocatedBorrow::with_file_id(ext, file_id))
                        }
                    }
                    mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_) => {
                        // Operation-only definitions are not relevant to the type system.
                    }
                    mir::Definition::DirectiveDefinition(_)
                    | mir::Definition::SchemaDefinition(_)
                    | mir::Definition::ScalarTypeDefinition(_)
                    | mir::Definition::ObjectTypeDefinition(_)
                    | mir::Definition::InterfaceTypeDefinition(_)
                    | mir::Definition::UnionTypeDefinition(_)
                    | mir::Definition::EnumTypeDefinition(_)
                    | mir::Definition::InputObjectTypeDefinition(_) => {
                        // Base definitions were already handled.
                    }
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
    ) -> &'static [Component<mir::FieldDefinition>] {
        static ROOT_QUERY_FIELDS: LazyLock<[Component<mir::FieldDefinition>; 3]> =
            LazyLock::new(|| {
                [
                    // __typename: String!
                    Component::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__typename".into(),
                        arguments: Vec::new(),
                        ty: mir::Type::new_named("String").non_null(),
                        directives: Vec::new(),
                    }),
                    // __schema: __Schema!
                    Component::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__schema".into(),
                        arguments: Vec::new(),
                        ty: mir::Type::new_named("__Schema").non_null(),
                        directives: Vec::new(),
                    }),
                    // __type(name: String!): __Type
                    Component::no_location(mir::FieldDefinition {
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
        static NON_ROOT_FIELDS: LazyLock<[Component<mir::FieldDefinition>; 1]> =
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

fn directives_from_mir<'a, T>(
    node: LocatedBorrow<'a, T>,
    extension: Option<&'a ExtensionId>,
    directives: &'a [Harc<Ranged<mir::Directive>>],
) -> impl Iterator<Item = Component<mir::Directive>> + 'a {
    directives
        .iter()
        .map(move |directive| node.component(directive, extension))
}

impl Schema {
    fn from_mir(definition: LocatedBorrow<'_, mir::SchemaDefinition>) -> Located<Self> {
        let mut schema = Schema {
            description: definition.description.clone(),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            query: None,
            mutation: None,
            subscription: None,
            query_extension: None,
            mutation_extension: None,
            subscription_extension: None,
        };
        schema.add_root_operations(None, &definition.root_operations);
        definition.same_location(schema)
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<mir::SchemaExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        self.add_root_operations(Some(id), &extension.root_operations);
    }

    fn add_root_operations(
        &mut self,
        extension: Option<ExtensionId>,
        root_operations: &[(mir::OperationType, Name)],
    ) {
        for (operation_type, object_type_name) in root_operations {
            let (name, ext) = match operation_type {
                mir::OperationType::Query => (&mut self.query, &mut self.query_extension),
                mir::OperationType::Mutation => (&mut self.mutation, &mut self.mutation_extension),
                mir::OperationType::Subscription => {
                    (&mut self.subscription, &mut self.subscription_extension)
                }
            };
            if name.is_none() {
                *name = Some(object_type_name.clone());
                *ext = extension.clone()
            }
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

    fn implicit(types: &IndexMap<Name, Type>) -> Located<Self> {
        let if_has_object_type = |name: &str| {
            if let Some(Type::Object(_)) = types.get(name) {
                Some(Name::from(name))
            } else {
                None
            }
        };
        Located::no_location(Self {
            description: None,
            directives: Vec::new(),
            query: if_has_object_type("Query"),
            mutation: if_has_object_type("Mutation"),
            subscription: if_has_object_type("Subscription"),
            query_extension: None,
            mutation_extension: None,
            subscription_extension: None,
        })
    }

    /// Collect schema extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(
                self.query
                    .as_ref()
                    .and_then(|_| self.query_extension.as_ref()),
            )
            .chain(
                self.mutation
                    .as_ref()
                    .and_then(|_| self.mutation_extension.as_ref()),
            )
            .chain(
                self.subscription
                    .as_ref()
                    .and_then(|_| self.subscription_extension.as_ref()),
            )
            .collect()
    }
}

impl ScalarType {
    fn from_mir(definition: LocatedBorrow<'_, mir::ScalarTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::ScalarTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
    }

    /// Collect scalar type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .collect()
    }
}

impl ObjectType {
    fn from_mir(definition: LocatedBorrow<'_, mir::ObjectTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            implements_interfaces: collect_sticky(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| (name, None)),
            ),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, definition.component(field, None))),
            ),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::ObjectTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        extend_sticky(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| (name, Some(id.clone()))),
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, extension.component(field, Some(&id)))),
        );
    }

    /// Collect object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(self.fields.values().flat_map(|field| field.extension_id()))
            .collect()
    }
}

impl InterfaceType {
    fn from_mir(definition: LocatedBorrow<'_, mir::InterfaceTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            implements_interfaces: collect_sticky(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| (name, None)),
            ),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, definition.component(field, None))),
            ),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::InterfaceTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        extend_sticky(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| (name, Some(id.clone()))),
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, extension.component(field, Some(&id)))),
        );
    }

    /// Collect interface type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(self.fields.values().flat_map(|field| field.extension_id()))
            .collect()
    }
}

impl UnionType {
    fn from_mir(definition: LocatedBorrow<'_, mir::UnionTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            members: collect_sticky(definition.members.iter().map(|name| (name, None))),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::UnionTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        extend_sticky(
            &mut self.members,
            extension
                .members
                .iter()
                .map(|name| (name, Some(id.clone()))),
        );
    }

    /// Collect union type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(self.members.values().flatten())
            .collect()
    }
}

impl EnumType {
    fn from_mir(definition: LocatedBorrow<'_, mir::EnumTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            values: collect_sticky(
                definition
                    .values
                    .iter()
                    .map(|value_def| (&value_def.value, definition.component(value_def, None))),
            ),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::EnumTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        extend_sticky(
            &mut self.values,
            extension
                .values
                .iter()
                .map(|value_def| (&value_def.value, extension.component(value_def, Some(&id)))),
        )
    }

    /// Collect enum type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(self.values.values().flat_map(|value| value.extension_id()))
            .collect()
    }
}

impl InputObjectType {
    fn from_mir(definition: LocatedBorrow<'_, mir::InputObjectTypeDefinition>) -> Located<Self> {
        definition.same_location(Self {
            description: definition.description.clone(),
            directives: directives_from_mir(definition, None, &definition.directives).collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, definition.component(field, None))),
            ),
        })
    }

    fn extend_mir(&mut self, extension: LocatedBorrow<'_, mir::InputObjectTypeExtension>) {
        let id = ExtensionId::new(extension);
        self.directives.extend(directives_from_mir(
            extension,
            Some(&id),
            &extension.directives,
        ));
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, extension.component(field, Some(&id)))),
        )
    }

    /// Collect input object type extensions that contribute any component
    ///
    /// The order of the returned set is unspecified but deterministic
    /// for a given apollo-compiler version.
    pub fn extensions(&self) -> IndexSet<&ExtensionId> {
        self.directives
            .iter()
            .flat_map(|dir| dir.extension_id())
            .chain(self.fields.values().flat_map(|field| field.extension_id()))
            .collect()
    }
}

impl Type {
    /// Returns the fields of an object type or interface type.
    /// For other types, always returns `None`.
    pub fn fields(&self) -> Option<&IndexMap<Name, Component<mir::FieldDefinition>>> {
        match self {
            Type::Object(ty) => Some(&ty.fields),
            Type::Interface(ty) => Some(&ty.fields),
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
