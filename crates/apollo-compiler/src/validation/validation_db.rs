use crate::validation::diagnostics::ValidationError;
use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definitions;
use crate::validation::input_object::validate_input_object_definitions;
use crate::validation::interface::validate_interface_definitions;
use crate::validation::object::validate_object_type_definitions;
use crate::validation::scalar::validate_scalar_definitions;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definitions;
use crate::validation::FileId;
use crate::{ast, name, InputDatabase, Node, ReprDatabase};
use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(ValidationStorage)]
pub(crate) trait ValidationDatabase: InputDatabase + ReprDatabase {
    fn ast_types(&self) -> Arc<ast::TypeSystem>;
    fn ast_named_fragments(
        &self,
        file_id: FileId,
    ) -> Arc<HashMap<ast::Name, Node<ast::FragmentDefinition>>>;
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

pub(crate) fn validate_type_system(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.ast_types().schema.clone();
    diagnostics.extend(validate_schema_definition(db, schema));

    diagnostics.extend(validate_scalar_definitions(db));
    diagnostics.extend(validate_enum_definitions(db));
    diagnostics.extend(validate_union_definitions(db));

    diagnostics.extend(validate_interface_definitions(db));
    diagnostics.extend(validate_directive_definitions(db));
    diagnostics.extend(validate_input_object_definitions(db));
    diagnostics.extend(validate_object_type_definitions(db));

    diagnostics
}

fn validate_executable_inner(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::operation::validate_operation_definitions(
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
