use super::*;
use indexmap::map::Entry;
use std::sync::Arc;

pub struct SchemaBuilder {
    adopt_orphan_extensions: bool,
    schema: Schema,
    schema_definition: SchemaDefinitionStatus,
    orphan_type_extensions: IndexMap<Name, Vec<ast::Definition>>,
}

enum SchemaDefinitionStatus {
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
            adopt_orphan_extensions: false,
            schema: Schema {
                sources: Default::default(),
                build_errors: Vec::new(),
                schema_definition: Node::new(SchemaDefinition {
                    description: None,
                    directives: Directives::default(),
                    query: None,
                    mutation: None,
                    subscription: None,
                }),
                directive_definitions: IndexMap::new(),
                types: IndexMap::new(),
            },
            schema_definition: SchemaDefinitionStatus::NoneSoFar {
                orphan_extensions: Vec::new(),
            },
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
                && matches!(
                    &builder.schema_definition,
                    SchemaDefinitionStatus::NoneSoFar { orphan_extensions }
                    if orphan_extensions.is_empty()
                )
        );
        builder
    }

    /// Configure the builder so that “orphan” schema extensions and type extensions
    /// (without a corresponding definition) are “adopted”:
    /// accepted as if extending an empty definition instead of being rejected as errors.
    pub fn adopt_orphan_extensions(mut self) -> Self {
        self.adopt_orphan_extensions = true;
        self
    }

    /// Parse an input file with the default configuration as an additional input for this schema.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    pub fn parse(mut self, source_text: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Parser::new().parse_into_schema_builder(source_text, path, &mut self);
        self
    }

    /// Add an AST document to the schema being built
    ///
    /// Executable definitions, if any, will be silently ignored.
    pub fn add_ast(mut self, document: &ast::Document) -> Self {
        let executable_definitions_are_errors = true;
        self.add_ast_document(document, executable_definitions_are_errors);
        self
    }

    pub(crate) fn add_ast_document(
        &mut self,
        document: &ast::Document,
        executable_definitions_are_errors: bool,
    ) {
        let sources = Arc::make_mut(&mut self.schema.sources);
        for (file_id, source_file) in document.sources.iter() {
            sources.insert(*file_id, source_file.clone());
        }
        for definition in &document.definitions {
            match definition {
                ast::Definition::SchemaDefinition(def) => match &self.schema_definition {
                    SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                        self.schema.schema_definition = SchemaDefinition::from_ast(
                            &mut self.schema.build_errors,
                            def,
                            orphan_extensions,
                        );
                        self.schema_definition = SchemaDefinitionStatus::Found;
                    }
                    SchemaDefinitionStatus::Found => {
                        self.schema
                            .build_errors
                            .push(BuildError::SchemaDefinitionCollision {
                                location: def.location(),
                                previous_location: self.schema.schema_definition.location(),
                            })
                    }
                },
                ast::Definition::DirectiveDefinition(def) => {
                    if let Err((_prev_name, previous)) =
                        insert_sticky(&mut self.schema.directive_definitions, &def.name, || {
                            def.clone()
                        })
                    {
                        if previous.is_built_in() {
                            // https://github.com/apollographql/apollo-rs/issues/656
                            // Re-defining a built-in definition is allowed, but only once.
                            // (`is_built_in` is based on file ID, not directive name,
                            // so the new definition won’t be considered built-in.)
                            *previous = def.clone()
                        } else {
                            self.schema.build_errors.push(
                                BuildError::DirectiveDefinitionCollision {
                                    location: def.name.location(),
                                    previous_location: previous.name.location(),
                                    name: def.name.clone(),
                                },
                            )
                        }
                    }
                }
                ast::Definition::ScalarTypeDefinition(def) => {
                    if let Err((prev_name, previous)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::Scalar(ScalarType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema.build_errors.push(if previous.is_built_in() {
                            BuildError::BuiltInScalarTypeRedefinition {
                                location: def.location(),
                            }
                        } else {
                            BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            }
                        })
                    }
                }
                ast::Definition::ObjectTypeDefinition(def) => {
                    if let Err((prev_name, _previous)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::Object(ObjectType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema
                            .build_errors
                            .push(BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            })
                    }
                }
                ast::Definition::InterfaceTypeDefinition(def) => {
                    if let Err((prev_name, _previous)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::Interface(InterfaceType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema
                            .build_errors
                            .push(BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            })
                    }
                }
                ast::Definition::UnionTypeDefinition(def) => {
                    if let Err((prev_name, _)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::Union(UnionType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema
                            .build_errors
                            .push(BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            })
                    }
                }
                ast::Definition::EnumTypeDefinition(def) => {
                    if let Err((prev_name, _previous)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::Enum(EnumType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema
                            .build_errors
                            .push(BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            })
                    }
                }
                ast::Definition::InputObjectTypeDefinition(def) => {
                    if let Err((prev_name, _previous)) =
                        insert_sticky(&mut self.schema.types, &def.name, || {
                            ExtendedType::InputObject(InputObjectType::from_ast(
                                &mut self.schema.build_errors,
                                def,
                                self.orphan_type_extensions
                                    .remove(&def.name)
                                    .unwrap_or_default(),
                            ))
                        })
                    {
                        self.schema
                            .build_errors
                            .push(BuildError::TypeDefinitionCollision {
                                location: def.name.location(),
                                previous_location: prev_name.location(),
                                name: def.name.clone(),
                            })
                    }
                }
                ast::Definition::SchemaExtension(ext) => match &mut self.schema_definition {
                    SchemaDefinitionStatus::Found => self
                        .schema
                        .schema_definition
                        .make_mut()
                        .extend_ast(&mut self.schema.build_errors, ext),
                    SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                        orphan_extensions.push(ext.clone())
                    }
                },
                ast::Definition::ScalarTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::Scalar(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::ObjectTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::Object(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::InterfaceTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::Interface(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::UnionTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::Union(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::EnumTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::Enum(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry(ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                }
                ast::Definition::InputObjectTypeExtension(ext) => {
                    if let Some((_, ty_name, ty)) = self.schema.types.get_full_mut(&ext.name) {
                        if let ExtendedType::InputObject(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.schema.build_errors, ext)
                        } else {
                            self.schema
                                .build_errors
                                .push(BuildError::TypeExtensionKindMismatch {
                                    location: ext.name.location(),
                                    name: ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty_name.location(),
                                    describe_def: ty.describe(),
                                })
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

    /// Returns the schema built from all added documents
    pub fn build(self) -> Schema {
        let SchemaBuilder {
            adopt_orphan_extensions,
            mut schema,
            schema_definition,
            orphan_type_extensions,
        } = self;
        match schema_definition {
            SchemaDefinitionStatus::Found => {}
            SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                // This a macro rather than a closure to generate separate `static`s
                let mut has_implicit_root_operation = false;
                macro_rules! default_root_operation {
                    ($($operation_type: path: $root_operation: expr,)+) => {{
                        $(
                            let name = $operation_type.default_type_name();
                            if let Some(ExtendedType::Object(_)) = schema.types.get(name) {
                                static OBJECT_TYPE_NAME: OnceLock<ComponentStr> = OnceLock::new();
                                $root_operation = Some(OBJECT_TYPE_NAME.get_or_init(|| {
                                    Name::new(name).to_component(ComponentOrigin::Definition)
                                }).clone());
                                has_implicit_root_operation = true;
                            }
                        )+
                    }};
                }
                let schema_def = schema.schema_definition.make_mut();
                default_root_operation!(
                    ast::OperationType::Query: schema_def.query,
                    ast::OperationType::Mutation: schema_def.mutation,
                    ast::OperationType::Subscription: schema_def.subscription,
                );

                let apply_schema_extensions =
                    // https://github.com/apollographql/apollo-rs/issues/682
                    // If we have no explict `schema` definition but do have object type(s)
                    // with a default type name for root operations,
                    // an implicit schema definition is generated with those root operations.
                    // That implict definition can be extended:
                    has_implicit_root_operation ||
                    // https://github.com/apollographql/apollo-rs/pull/678
                    // In this opt-in mode we unconditionally assume
                    // an implicit schema definition to extend
                    adopt_orphan_extensions;
                if apply_schema_extensions {
                    for ext in &orphan_extensions {
                        schema_def.extend_ast(&mut schema.build_errors, ext)
                    }
                } else {
                    schema
                        .build_errors
                        .extend(orphan_extensions.into_iter().map(|ext| {
                            BuildError::OrphanSchemaExtension {
                                location: ext.location(),
                            }
                        }));
                }
            }
        }
        // https://github.com/apollographql/apollo-rs/pull/678
        if adopt_orphan_extensions {
            for (type_name, extensions) in orphan_type_extensions {
                let type_def = adopt_type_extensions(&mut schema, &type_name, &extensions);
                let previous = schema.types.insert(type_name, type_def);
                assert!(previous.is_none());
            }
        } else {
            schema
                .build_errors
                .extend(orphan_type_extensions.into_values().flatten().map(|ext| {
                    let name = ext.name().unwrap().clone();
                    BuildError::OrphanTypeExtension {
                        location: name.location(),
                        name,
                    }
                }));
        }
        schema
    }
}

fn adopt_type_extensions(
    schema: &mut Schema,
    type_name: &NodeStr,
    extensions: &[ast::Definition],
) -> ExtendedType {
    macro_rules! extend {
        ($( $ExtensionVariant: path => $describe: literal $empty_def: expr )+) => {
            match &extensions[0] {
                $(
                    $ExtensionVariant(_) => {
                        let mut def = $empty_def;
                        for ext in extensions {
                            if let $ExtensionVariant(ext) = ext {
                                def.extend_ast(&mut schema.build_errors, ext)
                            } else {
                                let ext_name = ext.name().unwrap();
                                schema
                                    .build_errors
                                    .push(BuildError::TypeExtensionKindMismatch {
                                        location: ext_name.location(),
                                        name: ext_name.clone(),
                                        describe_ext: ext.describe(),
                                        def_location: type_name.location(),
                                        describe_def: $describe,
                                    })
                            }
                        }
                        def.into()
                    }
                )+
                _ => unreachable!(),
            }
        };
    }
    extend! {
        ast::Definition::ScalarTypeExtension => "a scalar type" ScalarType {
            description: Default::default(),
            directives: Default::default(),
        }
        ast::Definition::ObjectTypeExtension => "an object type" ObjectType {
            description: Default::default(),
            implements_interfaces: Default::default(),
            directives: Default::default(),
            fields: Default::default(),
        }
        ast::Definition::InterfaceTypeExtension => "an interface type" InterfaceType {
            description: Default::default(),
            implements_interfaces: Default::default(),
            directives: Default::default(),
            fields: Default::default(),
        }
        ast::Definition::UnionTypeExtension => "a union type" UnionType {
            description: Default::default(),
            directives: Default::default(),
            members: Default::default(),
        }
        ast::Definition::EnumTypeExtension => "an enum type" EnumType {
            description: Default::default(),
            directives: Default::default(),
            values: Default::default(),
        }
        ast::Definition::InputObjectTypeExtension => "an input object type" InputObjectType {
            description: Default::default(),
            directives: Default::default(),
            fields: Default::default(),
        }
    }
}

impl SchemaDefinition {
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
            match entry {
                None => *entry = Some(object_type_name.to_component(origin.clone())),
                Some(previous) => errors.push(BuildError::DuplicateRootOperation {
                    location: op.location(),
                    previous_location: previous.location(),
                    operation_type: operation_type.name(),
                }),
            }
        }
    }
}

impl ScalarType {
    fn from_ast(
        errors: &mut [BuildError],
        definition: &Node<ast::ScalarTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            directives: definition
                .directives
                .iter()
                .map(|d| d.to_component(ComponentOrigin::Definition))
                .collect(),
        };
        for def in &extensions {
            if let ast::Definition::ScalarTypeExtension(ext) = def {
                ty.extend_ast(errors, ext)
            }
        }
        definition.same_location(ty)
    }

    fn extend_ast(
        &mut self,
        _errors: &mut [BuildError],
        extension: &Node<ast::ScalarTypeExtension>,
    ) {
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
            description: definition.description.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |prev, dup| {
                    errors.push(BuildError::DuplicateImplementsInterfaceInObject {
                        location: dup.location(),
                        name_at_previous_location: prev.node.clone(),
                        type_name: definition.name.clone(),
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
                |prev_key, dup_value| {
                    errors.push(BuildError::ObjectFieldNameCollision {
                        location: dup_value.location(),
                        name_at_previous_location: prev_key.clone(),
                        type_name: definition.name.clone(),
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
            |prev, dup| {
                errors.push(BuildError::DuplicateImplementsInterfaceInObject {
                    location: dup.location(),
                    name_at_previous_location: prev.node.clone(),
                    type_name: extension.name.clone(),
                })
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |prev_key, dup_value| {
                errors.push(BuildError::ObjectFieldNameCollision {
                    location: dup_value.location(),
                    name_at_previous_location: prev_key.clone(),
                    type_name: extension.name.clone(),
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
            description: definition.description.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |prev, dup| {
                    errors.push(BuildError::DuplicateImplementsInterfaceInInterface {
                        location: dup.location(),
                        name_at_previous_location: prev.node.clone(),
                        type_name: definition.name.clone(),
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
                |prev_key, dup_value| {
                    errors.push(BuildError::InterfaceFieldNameCollision {
                        location: dup_value.location(),
                        name_at_previous_location: prev_key.clone(),
                        type_name: definition.name.clone(),
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
            |prev, dup| {
                errors.push(BuildError::DuplicateImplementsInterfaceInInterface {
                    location: dup.location(),
                    name_at_previous_location: prev.node.clone(),
                    type_name: extension.name.clone(),
                })
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |prev_key, dup_value| {
                errors.push(BuildError::InterfaceFieldNameCollision {
                    location: dup_value.location(),
                    name_at_previous_location: prev_key.clone(),
                    type_name: extension.name.clone(),
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
                |prev, dup| {
                    errors.push(BuildError::UnionMemberNameCollision {
                        location: dup.location(),
                        name_at_previous_location: prev.node.clone(),
                        type_name: definition.name.clone(),
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
            |prev, dup| {
                errors.push(BuildError::UnionMemberNameCollision {
                    location: dup.location(),
                    name_at_previous_location: prev.node.clone(),
                    type_name: extension.name.clone(),
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
                |prev_key, dup_value| {
                    errors.push(BuildError::EnumValueNameCollision {
                        location: dup_value.location(),
                        name_at_previous_location: prev_key.clone(),
                        type_name: definition.name.clone(),
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
            |prev_key, dup_value| {
                errors.push(BuildError::EnumValueNameCollision {
                    location: dup_value.location(),
                    name_at_previous_location: prev_key.clone(),
                    type_name: extension.name.clone(),
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
                |prev_key, dup_value| {
                    errors.push(BuildError::InputFieldNameCollision {
                        location: dup_value.location(),
                        name_at_previous_location: prev_key.clone(),
                        type_name: definition.name.clone(),
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
            |prev_key, dup_value| {
                errors.push(BuildError::InputFieldNameCollision {
                    location: dup_value.location(),
                    name_at_previous_location: prev_key.clone(),
                    type_name: extension.name.clone(),
                })
            },
        )
    }
}

/// Like `IndexMap::insert`, but does not replace the value
/// if an equivalent key is already in the map.
///
/// In that (error) case, returns the existing key and value
fn insert_sticky<'map, V>(
    map: &'map mut IndexMap<Name, V>,
    key: &Name,
    make_value: impl FnOnce() -> V,
) -> Result<(), (&'map Name, &'map mut V)> {
    match map.entry(key.clone()) {
        Entry::Vacant(entry) => {
            entry.insert(make_value());
            Ok(())
        }
        Entry::Occupied(_) => {
            let (_index, key, value) = map.get_full_mut(key).unwrap();
            Err((key, value))
        }
    }
}

/// Like `IndexMap::extend`, but does not replace a value if an equivalent key is already in the map.
///
/// On collision, calls `duplicate` with the previous key and the value not inserted
fn extend_sticky<'a, V>(
    map: &mut IndexMap<Name, V>,
    iter: impl IntoIterator<Item = (&'a Name, V)>,
    mut duplicate: impl FnMut(&Name, V),
) {
    for (key, value) in iter.into_iter() {
        match map.get_key_value(key) {
            None => {
                map.insert(key.clone(), value);
            }
            Some((prev_key, _)) => duplicate(prev_key, value),
        }
    }
}

/// Like `IndexMap::from_iterator`, but does not replace a value if an equivalent key is already in the map.
///
/// On collision, calls `duplicate` with the previous key and the value not inserted
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
    mut duplicate: impl FnMut(&ComponentStr, ComponentStr),
) {
    for value in iter.into_iter() {
        match set.get(&value) {
            None => {
                set.insert(value);
            }
            Some(previous) => duplicate(previous, value),
        }
    }
}
fn collect_sticky_set(
    iter: impl IntoIterator<Item = ComponentStr>,
    duplicate: impl FnMut(&ComponentStr, ComponentStr),
) -> IndexSet<ComponentStr> {
    let mut set = IndexSet::new();
    extend_sticky_set(&mut set, iter, duplicate);
    set
}