use super::*;
use crate::ast::OperationType;
use crate::validation::WithErrors;
use indexmap::map::Entry;
use std::sync::Arc;

#[derive(Clone)]
pub struct SchemaBuilder {
    adopt_orphan_extensions: bool,
    schema: Schema,
    schema_definition: SchemaDefinitionStatus,
    orphan_type_extensions: IndexMap<Name, Vec<ast::Definition>>,
    pub(crate) errors: DiagnosticList,
}

#[derive(Clone)]
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
        static BUILT_IN_TYPES: std::sync::OnceLock<SchemaBuilder> = std::sync::OnceLock::new();
        BUILT_IN_TYPES
            .get_or_init(|| {
                let mut builder = SchemaBuilder {
                    adopt_orphan_extensions: false,
                    schema: Schema {
                        sources: Default::default(),
                        schema_definition: Node::new(SchemaDefinition {
                            description: None,
                            directives: DirectiveList::default(),
                            query: None,
                            mutation: None,
                            subscription: None,
                        }),
                        directive_definitions: IndexMap::with_hasher(Default::default()),
                        types: IndexMap::with_hasher(Default::default()),
                    },
                    schema_definition: SchemaDefinitionStatus::NoneSoFar {
                        orphan_extensions: Vec::new(),
                    },
                    orphan_type_extensions: IndexMap::with_hasher(Default::default()),
                    errors: DiagnosticList::new(Default::default()),
                };
                let input = include_str!("../built_in_types.graphql").to_owned();
                let path = "built_in.graphql";
                let id = FileId::BUILT_IN;
                let ast =
                    ast::Document::parser().parse_ast_inner(input, path, id, &mut builder.errors);
                let executable_definitions_are_errors = true;
                builder.add_ast_document(&ast, executable_definitions_are_errors);
                assert!(builder.errors.is_empty());
                builder
            })
            .clone()
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
        Arc::make_mut(&mut self.errors.sources)
            .extend(document.sources.iter().map(|(k, v)| (*k, v.clone())));
        self.add_ast_document_not_adding_sources(document, executable_definitions_are_errors)
    }

    pub(crate) fn add_ast_document_not_adding_sources(
        &mut self,
        document: &ast::Document,
        executable_definitions_are_errors: bool,
    ) {
        for definition in &document.definitions {
            macro_rules! type_definition {
                ($def: ident, $Type: ident, is_scalar = $is_scalar: literal) => {
                    match self.schema.types.entry($def.name.clone()) {
                        Entry::Vacant(entry) => {
                            let extended_def = $Type::from_ast(
                                &mut self.errors,
                                $def,
                                self.orphan_type_extensions
                                    .shift_remove(&$def.name)
                                    .unwrap_or_default(),
                            );
                            entry.insert(extended_def.into());
                        }
                        Entry::Occupied(entry) => {
                            let previous = entry.get();
                            if $is_scalar && previous.is_built_in() {
                                self.errors.push(
                                    $def.location(),
                                    BuildError::BuiltInScalarTypeRedefinition,
                                )
                            } else {
                                self.errors.push(
                                    $def.name.location(),
                                    BuildError::TypeDefinitionCollision {
                                        previous_location: previous.name().location(),
                                        name: $def.name.clone(),
                                    },
                                )
                            }
                        }
                    }
                };
            }
            macro_rules! type_extension {
                ($ext: ident, $Kind: ident) => {
                    if let Some(ty) = self.schema.types.get_mut(&$ext.name) {
                        if let ExtendedType::$Kind(ty) = ty {
                            ty.make_mut().extend_ast(&mut self.errors, $ext)
                        } else {
                            self.errors.push(
                                $ext.name.location(),
                                BuildError::TypeExtensionKindMismatch {
                                    name: $ext.name.clone(),
                                    describe_ext: definition.describe(),
                                    def_location: ty.name().location(),
                                    describe_def: ty.describe(),
                                },
                            )
                        }
                    } else {
                        self.orphan_type_extensions
                            .entry($ext.name.clone())
                            .or_default()
                            .push(definition.clone())
                    }
                };
            }
            match definition {
                ast::Definition::SchemaDefinition(def) => match &self.schema_definition {
                    SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                        self.schema.schema_definition =
                            SchemaDefinition::from_ast(&mut self.errors, def, orphan_extensions);
                        self.schema_definition = SchemaDefinitionStatus::Found;
                    }
                    SchemaDefinitionStatus::Found => self.errors.push(
                        def.location(),
                        BuildError::SchemaDefinitionCollision {
                            previous_location: self.schema.schema_definition.location(),
                        },
                    ),
                },
                ast::Definition::DirectiveDefinition(def) => {
                    match self.schema.directive_definitions.entry(def.name.clone()) {
                        Entry::Vacant(entry) => {
                            entry.insert(def.clone());
                        }
                        Entry::Occupied(mut entry) => {
                            let previous = entry.get_mut();
                            if previous.is_built_in() {
                                // https://github.com/apollographql/apollo-rs/issues/656
                                // Re-defining a built-in definition is allowed, but only once.
                                // (`is_built_in` is based on file ID, not directive name,
                                // so the new definition won’t be considered built-in.)
                                *previous = def.clone()
                            } else {
                                self.errors.push(
                                    def.name.location(),
                                    BuildError::DirectiveDefinitionCollision {
                                        previous_location: previous.name.location(),
                                        name: def.name.clone(),
                                    },
                                )
                            }
                        }
                    }
                }
                ast::Definition::ScalarTypeDefinition(def) => {
                    type_definition!(def, ScalarType, is_scalar = true)
                }
                ast::Definition::ObjectTypeDefinition(def) => {
                    type_definition!(def, ObjectType, is_scalar = false)
                }
                ast::Definition::InterfaceTypeDefinition(def) => {
                    type_definition!(def, InterfaceType, is_scalar = false)
                }
                ast::Definition::UnionTypeDefinition(def) => {
                    type_definition!(def, UnionType, is_scalar = false)
                }
                ast::Definition::EnumTypeDefinition(def) => {
                    type_definition!(def, EnumType, is_scalar = false)
                }
                ast::Definition::InputObjectTypeDefinition(def) => {
                    type_definition!(def, InputObjectType, is_scalar = false)
                }
                ast::Definition::SchemaExtension(ext) => match &mut self.schema_definition {
                    SchemaDefinitionStatus::Found => self
                        .schema
                        .schema_definition
                        .make_mut()
                        .extend_ast(&mut self.errors, ext),
                    SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                        orphan_extensions.push(ext.clone())
                    }
                },
                ast::Definition::ScalarTypeExtension(ext) => type_extension!(ext, Scalar),
                ast::Definition::ObjectTypeExtension(ext) => type_extension!(ext, Object),
                ast::Definition::InterfaceTypeExtension(ext) => type_extension!(ext, Interface),
                ast::Definition::UnionTypeExtension(ext) => type_extension!(ext, Union),
                ast::Definition::EnumTypeExtension(ext) => type_extension!(ext, Enum),
                ast::Definition::InputObjectTypeExtension(ext) => type_extension!(ext, InputObject),
                ast::Definition::OperationDefinition(_)
                | ast::Definition::FragmentDefinition(_) => {
                    if executable_definitions_are_errors {
                        self.errors.push(
                            definition.location(),
                            BuildError::ExecutableDefinition {
                                describe: definition.describe(),
                            },
                        )
                    }
                }
            }
        }
    }

    /// Returns the schema built from all added documents
    pub fn build(self) -> Result<Schema, WithErrors<Schema>> {
        let (schema, errors) = self.build_inner();
        errors.into_result_with(schema)
    }

    pub(crate) fn build_inner(self) -> (Schema, DiagnosticList) {
        let SchemaBuilder {
            adopt_orphan_extensions,
            mut schema,
            schema_definition,
            orphan_type_extensions,
            mut errors,
        } = self;
        schema.sources = errors.sources.clone();
        match schema_definition {
            SchemaDefinitionStatus::Found => {}
            SchemaDefinitionStatus::NoneSoFar { orphan_extensions } => {
                // This a macro rather than a closure to generate separate `static`s
                let schema_def = schema.schema_definition.make_mut();
                let mut has_implicit_root_operation = false;
                for (operation_type, root_operation) in [
                    (OperationType::Query, &mut schema_def.query),
                    (OperationType::Mutation, &mut schema_def.mutation),
                    (OperationType::Subscription, &mut schema_def.subscription),
                ] {
                    let name = operation_type.default_type_name();
                    if schema.types.get(&name).is_some_and(|def| def.is_object()) {
                        *root_operation = Some(name.into());
                        has_implicit_root_operation = true
                    }
                }

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
                        schema_def.extend_ast(&mut errors, ext)
                    }
                } else {
                    for ext in &orphan_extensions {
                        errors.push(ext.location(), BuildError::OrphanSchemaExtension)
                    }
                }
            }
        }
        // https://github.com/apollographql/apollo-rs/pull/678
        if adopt_orphan_extensions {
            for (type_name, extensions) in orphan_type_extensions {
                let type_def = adopt_type_extensions(&mut errors, &type_name, &extensions);
                let previous = schema.types.insert(type_name, type_def);
                assert!(previous.is_none());
            }
        } else {
            for extensions in orphan_type_extensions.values() {
                for ext in extensions {
                    let name = ext.name().unwrap().clone();
                    errors.push(name.location(), BuildError::OrphanTypeExtension { name })
                }
            }
        }
        (schema, errors)
    }
}

fn adopt_type_extensions(
    errors: &mut DiagnosticList,
    type_name: &Name,
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
                                def.extend_ast(errors, ext)
                            } else {
                                let ext_name = ext.name().unwrap();
                                errors.push(
                                    ext_name.location(),
                                    BuildError::TypeExtensionKindMismatch {
                                        name: ext_name.clone(),
                                        describe_ext: ext.describe(),
                                        def_location: type_name.location(),
                                        describe_def: $describe,
                                    }
                                )
                            }
                        }
                        def.into()
                    }
                )+
                _ => unreachable!(),
            }
        };
    }
    let name = type_name.clone();
    extend! {
        ast::Definition::ScalarTypeExtension => "a scalar type" ScalarType {
            description: Default::default(),
            name,
            directives: Default::default(),
        }
        ast::Definition::ObjectTypeExtension => "an object type" ObjectType {
            description: Default::default(),
            name,
            implements_interfaces: Default::default(),
            directives: Default::default(),
            fields: Default::default(),
        }
        ast::Definition::InterfaceTypeExtension => "an interface type" InterfaceType {
            description: Default::default(),
            name,
            implements_interfaces: Default::default(),
            directives: Default::default(),
            fields: Default::default(),
        }
        ast::Definition::UnionTypeExtension => "a union type" UnionType {
            description: Default::default(),
            name,
            directives: Default::default(),
            members: Default::default(),
        }
        ast::Definition::EnumTypeExtension => "an enum type" EnumType {
            description: Default::default(),
            name,
            directives: Default::default(),
            values: Default::default(),
        }
        ast::Definition::InputObjectTypeExtension => "an input object type" InputObjectType {
            description: Default::default(),
            name,
            directives: Default::default(),
            fields: Default::default(),
        }
    }
}

impl SchemaDefinition {
    fn from_ast(
        errors: &mut DiagnosticList,
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

    fn extend_ast(&mut self, errors: &mut DiagnosticList, extension: &Node<ast::SchemaExtension>) {
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
        errors: &mut DiagnosticList,
        origin: ComponentOrigin,
        root_operations: &[Node<(OperationType, Name)>],
    ) {
        for op in root_operations {
            let (operation_type, object_type_name) = &**op;
            let entry = match operation_type {
                OperationType::Query => &mut self.query,
                OperationType::Mutation => &mut self.mutation,
                OperationType::Subscription => &mut self.subscription,
            };
            match entry {
                None => *entry = Some(object_type_name.to_component(origin.clone())),
                Some(previous) => errors.push(
                    op.location(),
                    BuildError::DuplicateRootOperation {
                        previous_location: previous.location(),
                        operation_type: operation_type.name(),
                    },
                ),
            }
        }
    }
}

impl ScalarType {
    fn from_ast(
        errors: &mut DiagnosticList,
        definition: &Node<ast::ScalarTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
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
        _errors: &mut DiagnosticList,
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
        errors: &mut DiagnosticList,
        definition: &Node<ast::ObjectTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |prev, dup| {
                    errors.push(
                        dup.location(),
                        BuildError::DuplicateImplementsInterfaceInObject {
                            name_at_previous_location: prev.name.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
                    errors.push(
                        dup_value.location(),
                        BuildError::ObjectFieldNameCollision {
                            name_at_previous_location: prev_key.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
        errors: &mut DiagnosticList,
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
                errors.push(
                    dup.location(),
                    BuildError::DuplicateImplementsInterfaceInObject {
                        name_at_previous_location: prev.name.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |prev_key, dup_value| {
                errors.push(
                    dup_value.location(),
                    BuildError::ObjectFieldNameCollision {
                        name_at_previous_location: prev_key.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        );
    }
}

impl InterfaceType {
    fn from_ast(
        errors: &mut DiagnosticList,
        definition: &Node<ast::InterfaceTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
            implements_interfaces: collect_sticky_set(
                definition
                    .implements_interfaces
                    .iter()
                    .map(|name| name.to_component(ComponentOrigin::Definition)),
                |prev, dup| {
                    errors.push(
                        dup.location(),
                        BuildError::DuplicateImplementsInterfaceInInterface {
                            name_at_previous_location: prev.name.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
                    errors.push(
                        dup_value.location(),
                        BuildError::InterfaceFieldNameCollision {
                            name_at_previous_location: prev_key.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
        errors: &mut DiagnosticList,
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
                errors.push(
                    dup.location(),
                    BuildError::DuplicateImplementsInterfaceInInterface {
                        name_at_previous_location: prev.name.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        );
        extend_sticky(
            &mut self.fields,
            extension
                .fields
                .iter()
                .map(|field| (&field.name, field.to_component(origin.clone()))),
            |prev_key, dup_value| {
                errors.push(
                    dup_value.location(),
                    BuildError::InterfaceFieldNameCollision {
                        name_at_previous_location: prev_key.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        );
    }
}

impl UnionType {
    fn from_ast(
        errors: &mut DiagnosticList,
        definition: &Node<ast::UnionTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
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
                    errors.push(
                        dup.location(),
                        BuildError::UnionMemberNameCollision {
                            name_at_previous_location: prev.name.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
        errors: &mut DiagnosticList,
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
                errors.push(
                    dup.location(),
                    BuildError::UnionMemberNameCollision {
                        name_at_previous_location: prev.name.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        );
    }
}

impl EnumType {
    fn from_ast(
        errors: &mut DiagnosticList,
        definition: &Node<ast::EnumTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
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
                    errors.push(
                        dup_value.location(),
                        BuildError::EnumValueNameCollision {
                            name_at_previous_location: prev_key.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
        errors: &mut DiagnosticList,
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
                errors.push(
                    dup_value.location(),
                    BuildError::EnumValueNameCollision {
                        name_at_previous_location: prev_key.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        )
    }
}

impl InputObjectType {
    fn from_ast(
        errors: &mut DiagnosticList,
        definition: &Node<ast::InputObjectTypeDefinition>,
        extensions: Vec<ast::Definition>,
    ) -> Node<Self> {
        let mut ty = Self {
            description: definition.description.clone(),
            name: definition.name.clone(),
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
                    errors.push(
                        dup_value.location(),
                        BuildError::InputFieldNameCollision {
                            name_at_previous_location: prev_key.clone(),
                            type_name: definition.name.clone(),
                        },
                    )
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
        errors: &mut DiagnosticList,
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
                errors.push(
                    dup_value.location(),
                    BuildError::InputFieldNameCollision {
                        name_at_previous_location: prev_key.clone(),
                        type_name: extension.name.clone(),
                    },
                )
            },
        )
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
    let mut map = IndexMap::with_hasher(Default::default());
    extend_sticky(&mut map, iter, duplicate);
    map
}

fn extend_sticky_set(
    set: &mut IndexSet<ComponentName>,
    iter: impl IntoIterator<Item = ComponentName>,
    mut duplicate: impl FnMut(&ComponentName, ComponentName),
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
    iter: impl IntoIterator<Item = ComponentName>,
    duplicate: impl FnMut(&ComponentName, ComponentName),
) -> IndexSet<ComponentName> {
    let mut set = IndexSet::with_hasher(Default::default());
    extend_sticky_set(&mut set, iter, duplicate);
    set
}
