use super::*;
use indexmap::map::Entry;

pub struct SchemaBuilder {
    schema: Schema,
    orphan_schema_extensions: Vec<Node<ast::SchemaExtension>>,
    orphan_type_extensions: IndexMap<Name, Vec<ast::Definition>>,
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
                sources: IndexMap::new(),
                build_errors: Vec::new(),
                root_operations: None,
                directive_definitions: IndexMap::new(),
                types: IndexMap::new(),
            },
            orphan_schema_extensions: Vec::new(),
            orphan_type_extensions: IndexMap::new(),
        };

        static BUILT_IN_TYPES: std::sync::OnceLock<ast::Document> = std::sync::OnceLock::new();
        let built_in = BUILT_IN_TYPES.get_or_init(|| {
            let input = include_str!("../built_in_types.graphql").to_owned();
            let path = "built_in.graphql".into();
            let id = FileId::BUILT_IN;
            let document = ast::Document::parser().parse_with_file_id(input, path, id);
            debug_assert!(document.check_parse_errors().is_ok());
            document
        });

        let executable_definitions_are_errors = true;
        builder.add_ast_document(built_in, executable_definitions_are_errors);
        debug_assert!(
            builder.schema.build_errors.is_empty()
                && builder.orphan_type_extensions.is_empty()
                && builder.orphan_schema_extensions.is_empty()
                && builder.schema.root_operations.is_none(),
        );
        builder
    }

    /// Parse an input file with the default configuration as an additional input for this schema.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    pub fn parse(&mut self, source_text: impl Into<String>, path: impl AsRef<Path>) {
        Parser::new().parse_into_schema_builder(source_text, path, self)
    }

    /// Add an AST document to the schema being built
    ///
    /// Executable definitions, if any, will be silently ignored.
    pub(crate) fn add_ast_document(
        &mut self,
        document: &ast::Document,
        executable_definitions_are_errors: bool,
    ) {
        if let Some((file_id, source_file)) = document.source.clone() {
            self.schema.sources.insert(file_id, source_file);
        }
        for definition in &document.definitions {
            match definition {
                ast::Definition::SchemaDefinition(def) => {
                    if self.schema.root_operations.is_none() {
                        self.schema.root_operations = Some(RootOperations::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            &self.orphan_schema_extensions,
                        ));
                        self.orphan_schema_extensions = Vec::new();
                    } else {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::DirectiveDefinition(def) => {
                    if !insert_sticky(&mut self.schema.directive_definitions, &def.name, || {
                        def.clone()
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::ScalarTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Scalar(ScalarType::from_ast(
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::ObjectTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Object(ObjectType::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::InterfaceTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Interface(InterfaceType::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::UnionTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Union(UnionType::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::EnumTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::Enum(EnumType::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::InputObjectTypeDefinition(def) => {
                    if !insert_sticky(&mut self.schema.types, &def.name, || {
                        ExtendedType::InputObject(InputObjectType::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            self.orphan_type_extensions
                                .remove(&def.name)
                                .unwrap_or_default(),
                        ))
                    }) {
                        self.schema
                            .build_errors
                            .push(BuildError::DefinitionCollision(definition.clone()))
                    }
                }
                ast::Definition::SchemaExtension(ext) => {
                    if let Some(root) = &mut self.schema.root_operations {
                        root.make_mut()
                            .extend_ast(&mut self.schema.build_errors, ext)
                    } else {
                        self.orphan_schema_extensions.push(ext.clone())
                    }
                }
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
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
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
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
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
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
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
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
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
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
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
                    if executable_definitions_are_errors {
                        self.schema
                            .build_errors
                            .push(BuildError::ExecutableDefinition {
                                location: definition.location(),
                                describe: definition.describe(),
                            })
                    }
                }
            }
        }
    }

    /// Returns the schema built from all added documents, and orphan extensions:
    ///
    /// * `Definition::SchemaExtension` variants if no `Definition::SchemaDefinition` was found
    /// * `Definition::*TypeExtension` if no `Definition::*TypeDefinition` with the same name
    ///   was found, or if it is a different kind of type
    pub fn build(self) -> Schema {
        let SchemaBuilder {
            mut schema,
            orphan_schema_extensions,
            orphan_type_extensions,
        } = self;
        schema.build_errors.extend(
            orphan_schema_extensions
                .into_iter()
                .map(|ext| BuildError::OrphanExtension(ast::Definition::SchemaExtension(ext))),
        );
        schema.build_errors.extend(
            orphan_type_extensions
                .into_values()
                .flatten()
                .map(BuildError::OrphanExtension),
        );
        schema
    }
}

impl RootOperations {
    fn from_ast(
        errors: &mut Vec<BuildError>,
        definition: &Node<ast::SchemaDefinition>,
        extensions: &[Node<ast::SchemaExtension>],
    ) -> Node<Self> {
        let mut root = Self {
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
            query: None,
            mutation: None,
            subscription: None,
        };
        root.add_root_operations(
            errors,
            ComponentOrigin::Definition,
            &definition.root_operations,
        );
        for ext in extensions {
            root.extend_ast(errors, ext)
        }
        definition.same_location(root)
    }

    fn extend_ast(&mut self, errors: &mut Vec<BuildError>, extension: &Node<ast::SchemaExtension>) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        self.add_root_operations(errors, origin, &extension.root_operations)
    }

    fn add_root_operations(
        &mut self,
        errors: &mut Vec<BuildError>,
        origin: ComponentOrigin,
        root_operations: &[Node<(ast::OperationType, Name)>],
    ) {
        for op in root_operations {
            let (operation_type, object_type_name) = &**op;
            let entry = match operation_type {
                ast::OperationType::Query => &mut self.query,
                ast::OperationType::Mutation => &mut self.mutation,
                ast::OperationType::Subscription => &mut self.subscription,
            };
            if entry.is_none() {
                *entry = Some(object_type_name.to_component(origin.clone()))
            } else {
                errors.push(BuildError::DuplicateRootOperation {
                    operation_type: *operation_type,
                    object_type: object_type_name.clone(),
                })
            }
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
        errors: &mut Vec<BuildError>,
        definition: &Node<ast::ObjectTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |dup| {
                    errors.push(BuildError::DuplicateImplementsInterface {
                        implementer_name: definition.name.clone(),
                        interface_name: dup.node,
                    })
                },
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
                |_, dup_value| {
                    errors.push(BuildError::FieldNameCollision {
                        type_name: definition.name.clone(),
                        field: dup_value.node,
                    })
                },
            ),
        };
        for def in &extensions {
            if let ast::Definition::ObjectTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        errors: &mut Vec<BuildError>,
        extension: &Node<ast::ObjectTypeExtension>,
    ) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky_set(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| name.to_component(origin.clone())),
            |dup| {
                errors.push(BuildError::DuplicateImplementsInterface {
                    implementer_name: extension.name.clone(),
                    interface_name: dup.node,
                })
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |_, dup_value| {
                errors.push(BuildError::FieldNameCollision {
                    type_name: extension.name.clone(),
                    field: dup_value.node,
                })
            },
        );
    }
}

impl InterfaceType {
    fn from_ast(
        errors: &mut Vec<BuildError>,
        definition: &Node<ast::InterfaceTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            name: definition.name.clone(),
            description: definition.description.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |dup| {
                    errors.push(BuildError::DuplicateImplementsInterface {
                        implementer_name: definition.name.clone(),
                        interface_name: dup.node,
                    })
                },
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
                |_, dup_value| {
                    errors.push(BuildError::FieldNameCollision {
                        type_name: definition.name.clone(),
                        field: dup_value.node,
                    })
                },
            ),
        };
        for def in &extensions {
            if let ast::Definition::InterfaceTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        errors: &mut Vec<BuildError>,
        extension: &Node<ast::InterfaceTypeExtension>,
    ) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky_set(
            &mut self.implements_interfaces,
            extension
                .implements_interfaces
                .iter()
                .map(|name| name.to_component(origin.clone())),
            |dup| {
                errors.push(BuildError::DuplicateImplementsInterface {
                    implementer_name: extension.name.clone(),
                    interface_name: dup.node,
                })
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |_, dup_value| {
                errors.push(BuildError::FieldNameCollision {
                    type_name: extension.name.clone(),
                    field: dup_value.node,
                })
            },
        );
    }
}

impl UnionType {
    fn from_ast(
        errors: &mut Vec<BuildError>,
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
            members: collect_sticky_set(
                definition
                    .members
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |dup| {
                    errors.push(BuildError::UnionMemberNameCollision {
                        union_name: definition.name.clone(),
                        member: dup.node,
                    })
                },
            ),
        };
        for def in &extensions {
            if let ast::Definition::UnionTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        errors: &mut Vec<BuildError>,
        extension: &Node<ast::UnionTypeExtension>,
    ) {
        let origin = ComponentOrigin::Extension(ExtensionId::new(extension));
        self.directives.extend(
            extension
                .directives
                .iter()
                .map(|d| d.to_component(origin.clone())),
        );
        extend_sticky_set(
            &mut self.members,
            extension
                .members
                .iter()
                .map(|name| name.to_component(origin.clone())),
            |dup| {
                errors.push(BuildError::UnionMemberNameCollision {
                    union_name: extension.name.clone(),
                    member: dup.node,
                })
            },
        );
    }
}

impl EnumType {
    fn from_ast(
        errors: &mut Vec<BuildError>,
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
            values: collect_sticky(
                definition.values.iter().map(|value_def| {
                    (
                        &value_def.value,
                        value_def.to_component(ComponentOrigin::Definition),
                    )
                }),
                |_, dup_value| {
                    errors.push(BuildError::EnumValueNameCollision {
                        enum_name: definition.name.clone(),
                        value: dup_value.node,
                    })
                },
            ),
        };
        for def in &extensions {
            if let ast::Definition::EnumTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        errors: &mut Vec<BuildError>,
        extension: &Node<ast::EnumTypeExtension>,
    ) {
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
            |_, dup_value| {
                errors.push(BuildError::EnumValueNameCollision {
                    enum_name: extension.name.clone(),
                    value: dup_value.node,
                })
            },
        )
    }
}

impl InputObjectType {
    fn from_ast(
        errors: &mut Vec<BuildError>,
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
                |_, dup_value| {
                    errors.push(BuildError::InputFieldNameCollision {
                        type_name: definition.name.clone(),
                        field: dup_value.node,
                    })
                },
            ),
        };
        for def in &extensions {
            if let ast::Definition::InputObjectTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        errors: &mut Vec<BuildError>,
        extension: &Node<ast::InputObjectTypeExtension>,
    ) {
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
            |_, dup_value| {
                errors.push(BuildError::InputFieldNameCollision {
                    type_name: extension.name.clone(),
                    field: dup_value.node,
                })
            },
        )
    }
}

/// Like `IndexMap::insert`, but does not replace the value if an equivalent key is already in the map.
///
/// Returns wether the value was inserted
fn insert_sticky<K, V>(
    map: &mut IndexMap<K, V>,
    key: impl Into<K>,
    make_value: impl FnOnce() -> V,
) -> bool
where
    K: std::hash::Hash + Eq,
{
    match map.entry(key.into()) {
        Entry::Vacant(entry) => {
            entry.insert(make_value());
            true
        }
        Entry::Occupied(_) => false,
    }
}

/// Like `IndexMap::extend`, but does not replace a value if an equivalent key is already in the map.
///
/// Calls `duplicate` with values not inserted
fn extend_sticky<'a, V>(
    map: &mut IndexMap<Name, V>,
    iter: impl IntoIterator<Item = (&'a Name, V)>,
    mut duplicate: impl FnMut(&Name, V),
) {
    for (key, value) in iter.into_iter() {
        match map.entry(key.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
            Entry::Occupied(_) => duplicate(key, value),
        }
    }
}

/// Like `IndexMap::from_iterator`, but does not replace a value if an equivalent key is already in the map.
///
/// Calls `duplicate` with values not inserted
fn collect_sticky<'a, V>(
    iter: impl IntoIterator<Item = (&'a Name, V)>,
    duplicate: impl FnMut(&Name, V),
) -> IndexMap<Name, V> {
    let mut map = IndexMap::new();
    extend_sticky(&mut map, iter, duplicate);
    map
}

fn extend_sticky_set(
    set: &mut IndexSet<ComponentStr>,
    iter: impl IntoIterator<Item = ComponentStr>,
    mut duplicate: impl FnMut(ComponentStr),
) {
    for value in iter.into_iter() {
        if !set.contains(&value) {
            set.insert(value);
        } else {
            duplicate(value)
        }
    }
}
fn collect_sticky_set(
    iter: impl IntoIterator<Item = ComponentStr>,
    duplicate: impl FnMut(ComponentStr),
) -> IndexSet<ComponentStr> {
    let mut set = IndexSet::new();
    extend_sticky_set(&mut set, iter, duplicate);
    set
}
