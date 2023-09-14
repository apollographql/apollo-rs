use super::*;
use crate::ast::OperationType;
use crate::Arc;

pub struct SchemaBuilder {
    schema: Schema,
    schema_definition: SchemaDefinition,
    orphan_type_extensions: IndexMap<Name, Vec<ast::Definition>>,
}

enum SchemaDefinition {
    Found,
    NoneSoFar {
        orphan_extensions: Vec<Node<ast::SchemaExtension>>,
    },
}

impl Default for SchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaBuilder {
    /// Returns a new schema builder initialized with built-in directives, built-in scalars,
    /// and introspection types
    pub fn new() -> Self {
        let mut builder = SchemaBuilder {
            schema: Schema {
                description: None,
                directives: Vec::new(),
                directive_definitions: IndexMap::new(),
                types: IndexMap::new(),
                query_type: None,
                mutation_type: None,
                subscription_type: None,
            },
            schema_definition: SchemaDefinition::NoneSoFar {
                orphan_extensions: Vec::new(),
            },
            orphan_type_extensions: IndexMap::new(),
        };

        static BUILT_IN_TYPES: std::sync::OnceLock<Arc<ast::Document>> = std::sync::OnceLock::new();
        let built_in = BUILT_IN_TYPES.get_or_init(|| {
            let input = include_str!("../built_in_types.graphql");
            let result = ast::Document::parser().parse_with_file_id(input, FileId::BUILT_IN);
            debug_assert!(result.syntax_errors.is_empty());
            result.document
        });

        builder.add_document(built_in);
        debug_assert!(
            builder.orphan_type_extensions.is_empty()
                && matches!(
                    &builder.schema_definition,
                    SchemaDefinition::NoneSoFar { orphan_extensions }
                    if orphan_extensions.is_empty()
                )
        );
        builder
    }

    /// Add an AST document to the schema being built
    ///
    /// Executable definitions, if any, will be silently ignored.
    pub fn add_document(&mut self, document: &ast::Document) {
        for definition in &document.definitions {
            match definition {
                ast::Definition::SchemaDefinition(def) => {
                    if let SchemaDefinition::NoneSoFar { orphan_extensions } =
                        &self.schema_definition
                    {
                        self.schema.set_ast(def, orphan_extensions);
                        self.schema_definition = SchemaDefinition::Found;
                    }
                }
                ast::Definition::DirectiveDefinition(def) => {
                    insert_sticky(&mut self.schema.directive_definitions, &def.name, || {
                        def.clone()
                    })
                }
                ast::Definition::ScalarTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Scalar(ScalarType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::ObjectTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Object(ObjectType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::InterfaceTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Interface(InterfaceType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::UnionTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Union(UnionType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::EnumTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Enum(EnumType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::InputObjectTypeDefinition(def) => {
                    insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::InputObject(InputObjectType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    });
                }
                ast::Definition::SchemaExtension(ext) => match &mut self.schema_definition {
                    SchemaDefinition::Found => self.schema.extend_ast(ext),
                    SchemaDefinition::NoneSoFar { orphan_extensions } => {
                        orphan_extensions.push(ext.clone())
                    }
                },
                ast::Definition::ScalarTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::Scalar(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::ObjectTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::Object(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::InterfaceTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::Interface(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::UnionTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::Union(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::EnumTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::Enum(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::InputObjectTypeExtension(ext) => {
                    if let Some(ty) = self.schema.types.get_mut(&ext.name) {
                        if let ExtendedType::InputObject(ty) = ty {
                            ty.make_mut().extend_ast(ext)
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::OperationDefinition(_)
                | ast::Definition::FragmentDefinition(_) => {
                    // Operation-only definitions are not relevant to the type system.
                }
            }
        }
    }

    /// Returns the schema built from all added documents, and orphan extensions:
    ///
    /// * `Definition::SchemaExtension` variants if no `Definition::SchemaDefinition` was found
    /// * `Definition::*TypeExtension` if no `Definition::*TypeDefinition` with the same name
    ///   was found, or if it is a different kind of type
    pub fn build(mut self) -> (Schema, impl Iterator<Item = ast::Definition>) {
        let orphan_schema_extensions =
            if let SchemaDefinition::NoneSoFar { orphan_extensions } = self.schema_definition {
                // Implict `schema`, ignoring extensions
                let if_has_object_type = |ty: OperationType| {
                    let name = ty.default_type_name();
                    self.schema
                        .types
                        .get(name)?
                        .is_object()
                        .then(|| Name::new(name).to_component(ComponentOrigin::Definition))
                };
                self.schema.query_type = if_has_object_type(OperationType::Query);
                self.schema.mutation_type = if_has_object_type(OperationType::Mutation);
                self.schema.subscription_type = if_has_object_type(OperationType::Subscription);
                orphan_extensions
            } else {
                Vec::new()
            };
        let orphan_definitions = orphan_schema_extensions
            .into_iter()
            .map(ast::Definition::SchemaExtension)
            .chain(self.orphan_type_extensions.into_values().flatten());
        (self.schema, orphan_definitions)
    }
}

impl Schema {
    fn set_ast(
        &mut self,
        definition: &Node<ast::SchemaDefinition>,
        extensions: &[Node<ast::SchemaExtension>],
    ) {
        self.description = definition.description.clone();
        self.directives = definition
            .directives
            .iter()
            .map(|d| d.to_component(ComponentOrigin::Definition))
            .collect();
        self.add_root_operations(ComponentOrigin::Definition, &definition.root_operations);
        for ext in extensions {
            self.extend_ast(ext)
        }
    }

    fn extend_ast(&mut self, extension: &Node<ast::SchemaExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        self.add_root_operations(origin, &extension.root_operations)
    }

    fn add_root_operations(
        &mut self,
        origin: ComponentOrigin,
        root_operations: &[(ast::OperationType, Name)],
    ) {
        for (operation_type, object_type_name) in root_operations {
            match operation_type {
                ast::OperationType::Query => &mut self.query_type,
                ast::OperationType::Mutation => &mut self.mutation_type,
                ast::OperationType::Subscription => &mut self.subscription_type,
            }
            .get_or_insert_with(|| object_type_name.to_component(origin.clone()));
        }
    }
}

impl ScalarType {
    fn from_ast(
        definition: &Node<ast::ScalarTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
        };
        for def in &extensions {
            if let ast::Definition::ScalarTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::ScalarTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
    }
}

impl ObjectType {
    fn from_ast(
        definition: &Node<ast::ObjectTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            implements_interfaces: collect_sticky(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| (name, ComponentOrigin::Definition)),
            ),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, field.to_component(ComponentOrigin::Definition))),
            ),
        };
        for def in &extensions {
            if let ast::Definition::ObjectTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::ObjectTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| (name, origin.clone())),
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
        );
    }
}

impl InterfaceType {
    fn from_ast(
        definition: &Node<ast::InterfaceTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            implements_interfaces: collect_sticky(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| (name, ComponentOrigin::Definition)),
            ),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, field.to_component(ComponentOrigin::Definition))),
            ),
        };
        for def in &extensions {
            if let ast::Definition::InterfaceTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::InterfaceTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| (name, origin.clone())),
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
        );
    }
}

impl UnionType {
    fn from_ast(
        definition: &Node<ast::UnionTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            members: collect_sticky(
                definition
                    .members
                    .iter()
                    .map(|name| (name, ComponentOrigin::Definition)),
            ),
        };
        for def in &extensions {
            if let ast::Definition::UnionTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::UnionTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky(
            &mut self.members,
            extension.members.iter().map(|name| (name, origin.clone())),
        );
    }
}

impl EnumType {
    fn from_ast(
        definition: &Node<ast::EnumTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            values: collect_sticky(definition.values.iter().map(|value_def| {
                (
                    &value_def.value,
                    value_def.to_component(ComponentOrigin::Definition),
                )
            })),
        };
        for def in &extensions {
            if let ast::Definition::EnumTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::EnumTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky(
            &mut self.values,
            extension
                .values
                .iter()
                .map(|value_def| (&value_def.value, value_def.to_component(origin.clone()))),
        )
    }
}

impl InputObjectType {
    fn from_ast(
        definition: &Node<ast::InputObjectTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            fields: collect_sticky(
                definition
                    .fields
                    .iter()
                    .map(|field| (&field.name, field.to_component(ComponentOrigin::Definition))),
            ),
        };
        for def in &extensions {
            if let ast::Definition::InputObjectTypeExtension(ext) = def {
                ty.extend_ast(ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(&mut self, extension: &Node<ast::InputObjectTypeExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
        )
    }
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
