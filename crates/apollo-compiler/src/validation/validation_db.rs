use crate::validation::diagnostics::ValidationError;
use crate::validation::{
    self, directive, enum_, input_object, interface, object, operation, scalar, union_, FileId,
};
use crate::{ast, name, schema, InputDatabase, Node, ReprDatabase};
use std::collections::HashMap;
use std::sync::Arc;

use super::field;

#[salsa::query_group(ValidationStorage)]
pub(crate) trait ValidationDatabase: InputDatabase + ReprDatabase {
    fn ast_types(&self) -> Arc<ast::TypeSystem>;
    fn ast_named_fragments(
        &self,
        file_id: FileId,
    ) -> Arc<HashMap<ast::Name, Node<ast::FragmentDefinition>>>;

    /// Validate all documents.
    fn validate(&self) -> Vec<ValidationError>;

    /// Validate the type system, combined of all type system documents known to
    /// the compiler.
    #[salsa::invoke(validate_type_system)]
    fn validate_type_system(&self) -> Vec<ValidationError>;

    /// Validate an executable document.
    #[salsa::invoke(validate_executable)]
    fn validate_executable(&self, file_id: FileId) -> Vec<ValidationError>;

    /// Validate a standalone executable document, without knowledge of the type system it executes
    /// against.
    ///
    /// This runs a subset of the validations from `validate_executable`.
    #[salsa::invoke(validate_standalone_executable)]
    fn validate_standalone_executable(&self, file_id: FileId) -> Vec<ValidationError>;

    #[salsa::invoke(validation::schema::validate_schema_definition)]
    fn validate_schema_definition(
        &self,
        def: ast::TypeWithExtensions<ast::SchemaDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(scalar::validate_scalar_definitions)]
    fn validate_scalar_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(scalar::validate_scalar_definition)]
    fn validate_scalar_definition(
        &self,
        scalar_def: Node<schema::ScalarType>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(enum_::validate_enum_definitions)]
    fn validate_enum_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(enum_::validate_enum_definition)]
    fn validate_enum_definition(
        &self,
        enum_: ast::TypeWithExtensions<ast::EnumTypeDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(union_::validate_union_definitions)]
    fn validate_union_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(union_::validate_union_definition)]
    fn validate_union_definition(
        &self,
        union_: ast::TypeWithExtensions<ast::UnionTypeDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(interface::validate_interface_definitions)]
    fn validate_interface_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(interface::validate_interface_definition)]
    fn validate_interface_definition(
        &self,
        interface: ast::TypeWithExtensions<ast::InterfaceTypeDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(directive::validate_directive_definition)]
    fn validate_directive_definition(
        &self,
        directive_definition: Node<ast::DirectiveDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(directive::validate_directive_definitions)]
    fn validate_directive_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(input_object::validate_input_object_definitions)]
    fn validate_input_object_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(input_object::validate_input_object_definition)]
    fn validate_input_object_definition(
        &self,
        input_object: ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(object::validate_object_type_definitions)]
    fn validate_object_type_definitions(&self) -> Vec<ValidationError>;

    #[salsa::invoke(object::validate_object_type_definition)]
    fn validate_object_type_definition(
        &self,
        def: ast::TypeWithExtensions<ast::ObjectTypeDefinition>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(field::validate_field_definitions)]
    fn validate_field_definitions(
        &self,
        fields: Vec<Node<ast::FieldDefinition>>,
    ) -> Vec<ValidationError>;

    #[salsa::invoke(operation::validate_operation_definitions)]
    fn validate_operation_definitions(&self, file_id: FileId) -> Vec<ValidationError>;
}

fn ast_types(db: &dyn ValidationDatabase) -> Arc<ast::TypeSystem> {
    let mut objects = HashMap::new();
    let mut scalars = HashMap::new();
    let mut interfaces = HashMap::new();
    let mut unions = HashMap::new();
    let mut enums = HashMap::new();
    let mut input_objects = HashMap::new();

    let mut schema_definition = None;
    let mut schema_extensions = vec![];

    for file_id in db.type_definition_files() {
        if file_id == FileId::BUILT_IN {
            continue;
        }

        let document = db.ast(file_id);
        for definition in document.definitions.iter() {
            match definition {
                ast::Definition::SchemaDefinition(schema) => {
                    schema_definition = Some(schema.clone());
                }
                ast::Definition::ObjectTypeDefinition(object) => {
                    objects.insert(
                        object.name.clone(),
                        ast::TypeWithExtensions {
                            definition: object.clone(),
                            extensions: vec![],
                        },
                    );
                }
                ast::Definition::ScalarTypeDefinition(scalar) => {
                    scalars.insert(
                        scalar.name.clone(),
                        ast::TypeWithExtensions {
                            definition: scalar.clone(),
                            extensions: vec![],
                        },
                    );
                }
                ast::Definition::InterfaceTypeDefinition(interface) => {
                    interfaces.insert(
                        interface.name.clone(),
                        ast::TypeWithExtensions {
                            definition: interface.clone(),
                            extensions: vec![],
                        },
                    );
                }
                ast::Definition::UnionTypeDefinition(union_) => {
                    unions.insert(
                        union_.name.clone(),
                        ast::TypeWithExtensions {
                            definition: union_.clone(),
                            extensions: vec![],
                        },
                    );
                }
                ast::Definition::EnumTypeDefinition(enum_) => {
                    enums.insert(
                        enum_.name.clone(),
                        ast::TypeWithExtensions {
                            definition: enum_.clone(),
                            extensions: vec![],
                        },
                    );
                }
                ast::Definition::InputObjectTypeDefinition(input_object) => {
                    input_objects.insert(
                        input_object.name.clone(),
                        ast::TypeWithExtensions {
                            definition: input_object.clone(),
                            extensions: vec![],
                        },
                    );
                }
                _ => (),
            }
        }
        for definition in document.definitions.iter() {
            match definition {
                ast::Definition::SchemaExtension(schema) => {
                    schema_extensions.push(schema.clone());
                }
                ast::Definition::ObjectTypeExtension(extension) => {
                    if let Some(ty) = objects.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                ast::Definition::ScalarTypeExtension(extension) => {
                    if let Some(ty) = scalars.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                ast::Definition::InterfaceTypeExtension(extension) => {
                    if let Some(ty) = interfaces.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                ast::Definition::UnionTypeExtension(extension) => {
                    if let Some(ty) = unions.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                ast::Definition::EnumTypeExtension(extension) => {
                    if let Some(ty) = enums.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                ast::Definition::InputObjectTypeExtension(extension) => {
                    if let Some(ty) = input_objects.get_mut(&extension.name) {
                        ty.extensions.push(extension.clone());
                    }
                }
                _ => (),
            }
        }
    }

    let schema = ast::TypeWithExtensions {
        definition: schema_definition.unwrap_or_else(|| {
            Node::new(ast::SchemaDefinition {
                description: None,
                directives: ast::DirectiveList::new(),
                root_operations: {
                    let mut operations = Vec::with_capacity(3);
                    let query_name = name!("Query");
                    if objects.contains_key(&query_name) {
                        operations.push((ast::OperationType::Query, query_name).into());
                    }
                    let mutation_name = name!("Mutation");
                    if objects.contains_key(&mutation_name) {
                        operations.push((ast::OperationType::Mutation, mutation_name).into());
                    }
                    let subscription_name = name!("Subscription");
                    if objects.contains_key(&subscription_name) {
                        operations
                            .push((ast::OperationType::Subscription, subscription_name).into());
                    }
                    operations
                },
            })
        }),
        extensions: schema_extensions,
    };

    Arc::new(ast::TypeSystem {
        schema,
        objects,
        scalars,
        interfaces,
        unions,
        enums,
        input_objects,
    })
}

pub(crate) fn ast_named_fragments(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Arc<HashMap<ast::Name, Node<ast::FragmentDefinition>>> {
    let document = db.ast(file_id);
    let mut named_fragments = HashMap::new();
    for definition in &document.definitions {
        if let ast::Definition::FragmentDefinition(fragment) = definition {
            named_fragments
                .entry(fragment.name.clone())
                .or_insert(fragment.clone());
        }
    }
    Arc::new(named_fragments)
}

pub(crate) fn validate(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_type_system());

    for file_id in db.executable_definition_files() {
        diagnostics.extend(db.validate_executable(file_id));
    }

    diagnostics
}

pub(crate) fn validate_type_system(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.ast_types().schema.clone();
    diagnostics.extend(db.validate_schema_definition(schema));

    diagnostics.extend(db.validate_scalar_definitions());
    diagnostics.extend(db.validate_enum_definitions());
    diagnostics.extend(db.validate_union_definitions());

    diagnostics.extend(db.validate_interface_definitions());
    diagnostics.extend(db.validate_directive_definitions());
    diagnostics.extend(db.validate_input_object_definitions());
    diagnostics.extend(db.validate_object_type_definitions());

    diagnostics
}

fn validate_executable_inner(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::operation::validate_operation_definitions_inner(
        db, file_id, has_schema,
    ));
    for def in db.ast_named_fragments(file_id).values() {
        diagnostics.extend(super::fragment::validate_fragment_used(
            db,
            def.clone(),
            file_id,
        ));
    }

    diagnostics
}

pub(crate) fn validate_standalone_executable(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ValidationError> {
    validate_executable_inner(db, file_id, false)
}

pub(crate) fn validate_executable(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ValidationError> {
    validate_executable_inner(db, file_id, true)
}
