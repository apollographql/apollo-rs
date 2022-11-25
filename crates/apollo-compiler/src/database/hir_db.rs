use std::sync::Arc;

use apollo_parser::{
    ast::{self, AstChildren, AstNode},
    SyntaxNode,
};
use uuid::Uuid;

use crate::{database::FileId, hir::*, AstDatabase, InputDatabase};

#[salsa::query_group(HirStorage)]
pub trait HirDatabase: InputDatabase + AstDatabase {
    // fn definitions(&self) -> Arc<Vec<ast::Definition>>;

    fn db_definitions(&self) -> Arc<Vec<Definition>>;

    fn type_system_definitions(&self) -> Arc<Vec<Definition>>;

    fn operations(&self) -> Arc<Vec<OperationDefinition>>;

    fn fragments(&self) -> Arc<Vec<FragmentDefinition>>;

    fn schema(&self) -> Arc<SchemaDefinition>;

    fn object_types(&self) -> Arc<Vec<ObjectTypeDefinition>>;

    fn scalars(&self) -> Arc<Vec<ScalarTypeDefinition>>;

    fn enums(&self) -> Arc<Vec<EnumTypeDefinition>>;

    fn unions(&self) -> Arc<Vec<UnionTypeDefinition>>;

    fn interfaces(&self) -> Arc<Vec<InterfaceTypeDefinition>>;

    fn directive_definitions(&self) -> Arc<Vec<DirectiveDefinition>>;

    fn input_objects(&self) -> Arc<Vec<InputObjectTypeDefinition>>;

    fn schema_extensions(&self) -> Arc<Vec<SchemaExtension>>;

    fn scalar_type_extensions(&self) -> Arc<Vec<ScalarTypeExtension>>;

    fn object_type_extensions(&self) -> Arc<Vec<ObjectTypeExtension>>;

    fn interface_type_extensions(&self) -> Arc<Vec<InterfaceTypeExtension>>;

    fn union_type_extensions(&self) -> Arc<Vec<UnionTypeExtension>>;

    fn enum_type_extensions(&self) -> Arc<Vec<EnumTypeExtension>>;

    fn input_object_type_extensions(&self) -> Arc<Vec<InputObjectTypeExtension>>;
}

fn db_definitions(db: &dyn HirDatabase) -> Arc<Vec<Definition>> {
    let mut definitions = Vec::clone(&*db.type_system_definitions());

    let operations: Vec<Definition> = db
        .operations()
        .iter()
        .map(|def| Definition::OperationDefinition(def.clone()))
        .collect();
    let fragments: Vec<Definition> = db
        .fragments()
        .iter()
        .map(|def| Definition::FragmentDefinition(def.clone()))
        .collect();

    definitions.extend(operations);
    definitions.extend(fragments);

    Arc::new(definitions)
}

fn type_system_definitions(db: &dyn HirDatabase) -> Arc<Vec<Definition>> {
    let mut definitions = Vec::new();

    let directives: Vec<Definition> = db
        .directive_definitions()
        .iter()
        .map(|def| Definition::DirectiveDefinition(def.clone()))
        .collect();
    let scalars: Vec<Definition> = db
        .scalars()
        .iter()
        .map(|def| Definition::ScalarTypeDefinition(def.clone()))
        .collect();
    let objects: Vec<Definition> = db
        .object_types()
        .iter()
        .map(|def| Definition::ObjectTypeDefinition(def.clone()))
        .collect();
    let interfaces: Vec<Definition> = db
        .interfaces()
        .iter()
        .map(|def| Definition::InterfaceTypeDefinition(def.clone()))
        .collect();
    let unions: Vec<Definition> = db
        .unions()
        .iter()
        .map(|def| Definition::UnionTypeDefinition(def.clone()))
        .collect();
    let enums: Vec<Definition> = db
        .enums()
        .iter()
        .map(|def| Definition::EnumTypeDefinition(def.clone()))
        .collect();
    let input_objects: Vec<Definition> = db
        .input_objects()
        .iter()
        .map(|def| Definition::InputObjectTypeDefinition(def.clone()))
        .collect();
    let schema = Definition::SchemaDefinition(db.schema().as_ref().clone());
    let schema_extensions: Vec<Definition> = db
        .schema_extensions()
        .iter()
        .map(|def| Definition::SchemaExtension(def.clone()))
        .collect();
    let scalar_type_extensions: Vec<Definition> = db
        .scalar_type_extensions()
        .iter()
        .map(|def| Definition::ScalarTypeExtension(def.clone()))
        .collect();
    let object_type_extensions: Vec<Definition> = db
        .object_type_extensions()
        .iter()
        .map(|def| Definition::ObjectTypeExtension(def.clone()))
        .collect();
    let interface_type_extensions: Vec<Definition> = db
        .interface_type_extensions()
        .iter()
        .map(|def| Definition::InterfaceTypeExtension(def.clone()))
        .collect();
    let union_type_extensions: Vec<Definition> = db
        .union_type_extensions()
        .iter()
        .map(|def| Definition::UnionTypeExtension(def.clone()))
        .collect();
    let enum_type_extensions: Vec<Definition> = db
        .enum_type_extensions()
        .iter()
        .map(|def| Definition::EnumTypeExtension(def.clone()))
        .collect();
    let input_object_type_extensions: Vec<Definition> = db
        .input_object_type_extensions()
        .iter()
        .map(|def| Definition::InputObjectTypeExtension(def.clone()))
        .collect();

    definitions.extend(directives);
    definitions.extend(scalars);
    definitions.extend(objects);
    definitions.extend(interfaces);
    definitions.extend(unions);
    definitions.extend(enums);
    definitions.extend(input_objects);
    definitions.push(schema);
    definitions.extend(schema_extensions);
    definitions.extend(scalar_type_extensions);
    definitions.extend(object_type_extensions);
    definitions.extend(interface_type_extensions);
    definitions.extend(union_type_extensions);
    definitions.extend(enum_type_extensions);
    definitions.extend(input_object_type_extensions);

    Arc::new(definitions)
}

fn operations(db: &dyn HirDatabase) -> Arc<Vec<OperationDefinition>> {
    let operations = db
        .executable_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::OperationDefinition(op_def) => {
                        Some(operation_definition(db, op_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<OperationDefinition>>()
        })
        .collect();
    Arc::new(operations)
}

fn fragments(db: &dyn HirDatabase) -> Arc<Vec<FragmentDefinition>> {
    let fragments = db
        .executable_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::FragmentDefinition(fragment_def) => {
                        Some(fragment_definition(db, fragment_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<FragmentDefinition>>()
        })
        .collect();
    Arc::new(fragments)
}

// FIXME(@lrlna): if our compiler is composed of multiple documents that for
// some reason have more than one schema definition, we should be raising an
// error.
//
// This implementation currently just finds the first schema definition, which
// means we can't really diagnose the "multiple schema definitions" errors.
fn schema(db: &dyn HirDatabase) -> Arc<SchemaDefinition> {
    let schema: Option<(FileId, ast::SchemaDefinition)> = db.type_definition_files().into_iter().find_map(|id| {
        let schema: Option<(FileId, ast::SchemaDefinition)> =
            db.ast(id).document().definitions().into_iter().find_map(
                |definition| match definition {
                    ast::Definition::SchemaDefinition(schema) => Some((id, schema)),
                    _ => None,
                },
            );
        schema
    });
    let mut schema_def =
        schema.map_or(SchemaDefinition::default(), |s| schema_definition(s.1, s.0));

    // NOTE(@lrlna):
    //
    // "Query", "Subscription", "Mutation" object type definitions do not need
    // to be explicitly defined in a schema definition, but are implicitly
    // added.
    //
    // There will be a time when we need to distinguish between implicit and
    // explicit definitions for validation purposes.
    let type_defs = add_object_type_id_to_schema(db);
    type_defs
        .iter()
        .for_each(|ty| schema_def.set_root_operation_type_definition(ty.clone()));

    Arc::new(schema_def)
}

fn schema_extensions(db: &dyn HirDatabase) -> Arc<Vec<SchemaExtension>> {
    let schema_ext = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::SchemaExtension(def) => Some(SchemaExtension {
                        directives: directives(def.directives(), id),
                        root_operation_type_definition: root_operation_type_definition(
                            def.root_operation_type_definitions(),
                            id,
                        ),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<SchemaExtension>>()
        })
        .collect();
    Arc::new(schema_ext)
}

fn object_types(db: &dyn HirDatabase) -> Arc<Vec<ObjectTypeDefinition>> {
    let objects = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::ObjectTypeDefinition(obj_def) => {
                        Some(object_type_definition(obj_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<ObjectTypeDefinition>>()
        })
        .collect();
    Arc::new(objects)
}

fn object_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<ObjectTypeExtension>> {
    let objects = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::ObjectTypeExtension(def) => Some(ObjectTypeExtension {
                        directives: directives(def.directives(), id),
                        name: name(def.name(), id),
                        implements_interfaces: implements_interfaces(
                            def.implements_interfaces(),
                            id,
                        ),
                        fields_definition: fields_definition(def.fields_definition(), id),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<ObjectTypeExtension>>()
        })
        .collect();
    Arc::new(objects)
}

fn scalars(db: &dyn HirDatabase) -> Arc<Vec<ScalarTypeDefinition>> {
    let scalars = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::ScalarTypeDefinition(scalar_def) => {
                        Some(scalar_definition(scalar_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<ScalarTypeDefinition>>()
        })
        .collect();
    let scalars = built_in_scalars(scalars);

    Arc::new(scalars)
}

fn scalar_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<ScalarTypeExtension>> {
    let scalars = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::ScalarTypeExtension(def) => Some(ScalarTypeExtension {
                        directives: directives(def.directives(), id),
                        name: name(def.name(), id),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<ScalarTypeExtension>>()
        })
        .collect();
    Arc::new(scalars)
}

fn enums(db: &dyn HirDatabase) -> Arc<Vec<EnumTypeDefinition>> {
    let enums = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::EnumTypeDefinition(enum_def) => {
                        Some(enum_definition(enum_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<EnumTypeDefinition>>()
        })
        .collect();
    Arc::new(enums)
}

fn enum_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<EnumTypeExtension>> {
    let enums = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::EnumTypeExtension(def) => Some(EnumTypeExtension {
                        directives: directives(def.directives(), id),
                        name: name(def.name(), id),
                        enum_values_definition: enum_values_definition(
                            def.enum_values_definition(),
                            id,
                        ),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<EnumTypeExtension>>()
        })
        .collect();
    Arc::new(enums)
}

fn unions(db: &dyn HirDatabase) -> Arc<Vec<UnionTypeDefinition>> {
    let unions = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::UnionTypeDefinition(union_def) => {
                        Some(union_definition(union_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<UnionTypeDefinition>>()
        })
        .collect();
    Arc::new(unions)
}

fn union_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<UnionTypeExtension>> {
    let unions = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::UnionTypeExtension(def) => Some(UnionTypeExtension {
                        directives: directives(def.directives(), id),
                        name: name(def.name(), id),
                        union_members: union_members(def.union_member_types(), id),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<UnionTypeExtension>>()
        })
        .collect();
    Arc::new(unions)
}

fn interfaces(db: &dyn HirDatabase) -> Arc<Vec<InterfaceTypeDefinition>> {
    let interfaces = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::InterfaceTypeDefinition(interface_def) => {
                        Some(interface_definition(interface_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<InterfaceTypeDefinition>>()
        })
        .collect();
    Arc::new(interfaces)
}

fn interface_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<InterfaceTypeExtension>> {
    let interfaces = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::InterfaceTypeExtension(def) => Some(InterfaceTypeExtension {
                        directives: directives(def.directives(), id),
                        name: name(def.name(), id),
                        implements_interfaces: implements_interfaces(
                            def.implements_interfaces(),
                            id,
                        ),
                        fields_definition: fields_definition(def.fields_definition(), id),
                        loc: location(id, def.syntax()),
                    }),
                    _ => None,
                })
                .collect::<Vec<InterfaceTypeExtension>>()
        })
        .collect();
    Arc::new(interfaces)
}
fn directive_definitions(db: &dyn HirDatabase) -> Arc<Vec<DirectiveDefinition>> {
    let directives = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::DirectiveDefinition(directive_def) => {
                        Some(directive_definition(directive_def, id))
                    }
                    _ => None,
                })
                .collect::<Vec<DirectiveDefinition>>()
        })
        .collect();

    let directives = built_in_directives(directives);

    Arc::new(directives)
}

fn input_objects(db: &dyn HirDatabase) -> Arc<Vec<InputObjectTypeDefinition>> {
    let input_objs = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::InputObjectTypeDefinition(input_obj) => {
                        Some(input_object_definition(input_obj, id))
                    }
                    _ => None,
                })
                .collect::<Vec<InputObjectTypeDefinition>>()
        })
        .collect();

    Arc::new(input_objs)
}

fn input_object_type_extensions(db: &dyn HirDatabase) -> Arc<Vec<InputObjectTypeExtension>> {
    let input_objs = db
        .type_definition_files()
        .into_iter()
        .flat_map(|id| {
            db.ast(id)
                .document()
                .definitions()
                .into_iter()
                .filter_map(|definition| match definition {
                    ast::Definition::InputObjectTypeExtension(def) => {
                        Some(InputObjectTypeExtension {
                            directives: directives(def.directives(), id),
                            name: name(def.name(), id),
                            input_fields_definition: input_fields_definition(
                                def.input_fields_definition(),
                                id,
                            ),
                            loc: location(id, def.syntax()),
                        })
                    }
                    _ => None,
                })
                .collect::<Vec<InputObjectTypeExtension>>()
        })
        .collect();
    Arc::new(input_objs)
}

fn operation_definition(
    db: &dyn HirDatabase,
    op_def: ast::OperationDefinition,
    file_id: FileId,
) -> OperationDefinition {
    // check if there are already operations
    // if there are operations, they must have names
    // if there are no names, an error must be raised that all operations must have a name
    let name = op_def.name().map(|n| name_hir_node(n, file_id));
    let ty = operation_type(op_def.operation_type());
    let variables = variable_definitions(op_def.variable_definitions(), file_id);
    let parent_object_ty = db
        .schema()
        .root_operation_type_definition()
        .iter()
        .find_map(|op| {
            if op.operation_type() == ty {
                Some(op.named_type().name())
            } else {
                None
            }
        });
    let selection_set = selection_set(db, op_def.selection_set(), parent_object_ty, file_id);
    let directives = directives(op_def.directives(), file_id);
    let loc = location(file_id, op_def.syntax());

    OperationDefinition {
        id: Uuid::new_v4(),
        operation_ty: ty,
        name,
        variables,
        selection_set,
        directives,
        loc,
    }
}

fn fragment_definition(
    db: &dyn HirDatabase,
    fragment_def: ast::FragmentDefinition,
    file_id: FileId,
) -> FragmentDefinition {
    let name = name(
        fragment_def
            .fragment_name()
            .expect("Fragment Definition must have a name")
            .name(),
        file_id,
    );
    let type_condition = fragment_def
        .type_condition()
        .expect("Fragment Definition must have a type condition")
        .named_type()
        .expect("Type Condition must have a name")
        .name()
        .expect("Name must have text")
        .text()
        .to_string();
    let selection_set = selection_set(
        db,
        fragment_def.selection_set(),
        Some(type_condition.clone()),
        file_id,
    );
    let directives = directives(fragment_def.directives(), file_id);
    let loc = location(file_id, fragment_def.syntax());

    FragmentDefinition {
        id: Uuid::new_v4(),
        loc,
        name,
        type_condition,
        selection_set,
        directives,
    }
}

fn schema_definition(schema_def: ast::SchemaDefinition, file_id: FileId) -> SchemaDefinition {
    let description = description(schema_def.description());
    let directives = directives(schema_def.directives(), file_id);
    let root_operation_type_definition =
        root_operation_type_definition(schema_def.root_operation_type_definitions(), file_id);
    let loc = location(file_id, schema_def.syntax());

    SchemaDefinition {
        description,
        directives,
        root_operation_type_definition,
        loc: Some(loc),
    }
}

fn object_type_definition(
    obj_def: ast::ObjectTypeDefinition,
    file_id: FileId,
) -> ObjectTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(obj_def.description());
    let name = name(obj_def.name(), file_id);
    let implements_interfaces = implements_interfaces(obj_def.implements_interfaces(), file_id);
    let directives = directives(obj_def.directives(), file_id);
    let fields_definition = fields_definition(obj_def.fields_definition(), file_id);
    let loc = location(file_id, obj_def.syntax());

    ObjectTypeDefinition {
        id,
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        loc,
    }
}

fn scalar_definition(
    scalar_def: ast::ScalarTypeDefinition,
    file_id: FileId,
) -> ScalarTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(scalar_def.description());
    let name = name(scalar_def.name(), file_id);
    let directives = directives(scalar_def.directives(), file_id);
    let loc = location(file_id, scalar_def.syntax());

    ScalarTypeDefinition {
        id,
        description,
        name,
        directives,
        loc: Some(loc),
        built_in: false,
    }
}

fn enum_definition(enum_def: ast::EnumTypeDefinition, file_id: FileId) -> EnumTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(enum_def.description());
    let name = name(enum_def.name(), file_id);
    let directives = directives(enum_def.directives(), file_id);
    let enum_values_definition = enum_values_definition(enum_def.enum_values_definition(), file_id);
    let loc = location(file_id, enum_def.syntax());

    EnumTypeDefinition {
        id,
        description,
        name,
        directives,
        enum_values_definition,
        loc,
    }
}

fn enum_values_definition(
    enum_values_def: Option<ast::EnumValuesDefinition>,
    file_id: FileId,
) -> Arc<Vec<EnumValueDefinition>> {
    match enum_values_def {
        Some(enum_values) => {
            let enum_values = enum_values
                .enum_value_definitions()
                .into_iter()
                .map(|e| enum_value_definition(e, file_id))
                .collect();
            Arc::new(enum_values)
        }
        None => Arc::new(Vec::new()),
    }
}

fn enum_value_definition(
    enum_value_def: ast::EnumValueDefinition,
    file_id: FileId,
) -> EnumValueDefinition {
    let description = description(enum_value_def.description());
    let enum_value = enum_value(enum_value_def.enum_value(), file_id);
    let directives = directives(enum_value_def.directives(), file_id);
    let loc = location(file_id, enum_value_def.syntax());

    EnumValueDefinition {
        description,
        enum_value,
        directives,
        loc,
    }
}

fn union_definition(union_def: ast::UnionTypeDefinition, file_id: FileId) -> UnionTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(union_def.description());
    let name = name(union_def.name(), file_id);
    let directives = directives(union_def.directives(), file_id);
    let union_members = union_members(union_def.union_member_types(), file_id);
    let loc = location(file_id, union_def.syntax());

    UnionTypeDefinition {
        id,
        description,
        name,
        directives,
        union_members,
        loc,
    }
}

fn union_members(
    union_members: Option<ast::UnionMemberTypes>,
    file_id: FileId,
) -> Arc<Vec<UnionMember>> {
    match union_members {
        Some(members) => {
            let mems = members
                .named_types()
                .into_iter()
                .map(|u| union_member(u, file_id))
                .collect();
            Arc::new(mems)
        }
        None => Arc::new(Vec::new()),
    }
}

fn union_member(member: ast::NamedType, file_id: FileId) -> UnionMember {
    let name = name(member.name(), file_id);
    let loc = location(file_id, member.syntax());

    UnionMember { name, loc }
}

fn interface_definition(
    interface_def: ast::InterfaceTypeDefinition,
    file_id: FileId,
) -> InterfaceTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(interface_def.description());
    let name = name(interface_def.name(), file_id);
    let implements_interfaces =
        implements_interfaces(interface_def.implements_interfaces(), file_id);
    let directives = directives(interface_def.directives(), file_id);
    let fields_definition = fields_definition(interface_def.fields_definition(), file_id);
    let loc = location(file_id, interface_def.syntax());

    InterfaceTypeDefinition {
        id,
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        loc,
    }
}

fn directive_definition(
    directive_def: ast::DirectiveDefinition,
    file_id: FileId,
) -> DirectiveDefinition {
    let name = name(directive_def.name(), file_id);
    let description = description(directive_def.description());
    let arguments = arguments_definition(directive_def.arguments_definition(), file_id);
    let repeatable = directive_def.repeatable_token().is_some();
    let directive_locations = directive_locations(directive_def.directive_locations());
    let loc = location(file_id, directive_def.syntax());

    DirectiveDefinition {
        id: Uuid::new_v4(),
        description,
        name,
        arguments,
        repeatable,
        directive_locations,
        loc: Some(loc),
    }
}

fn input_object_definition(
    input_obj: ast::InputObjectTypeDefinition,
    file_id: FileId,
) -> InputObjectTypeDefinition {
    let id = Uuid::new_v4();
    let description = description(input_obj.description());
    let name = name(input_obj.name(), file_id);
    let directives = directives(input_obj.directives(), file_id);
    let input_fields_definition =
        input_fields_definition(input_obj.input_fields_definition(), file_id);
    let loc = location(file_id, input_obj.syntax());

    InputObjectTypeDefinition {
        id,
        description,
        name,
        directives,
        input_fields_definition,
        loc,
    }
}

fn add_object_type_id_to_schema(db: &dyn HirDatabase) -> Arc<Vec<RootOperationTypeDefinition>> {
    // Schema Definition does not have to be present in the SDL if ObjectType name is
    // - Query
    // - Subscription
    // - Mutation
    //
    // Compiler's internal schema, however, should have a reference to these
    // object types if they are present
    let type_defs: Vec<RootOperationTypeDefinition> = db
        .object_types()
        .iter()
        .filter_map(|obj_type| {
            let obj_name = obj_type.name();
            if matches!(obj_name, "Query" | "Subscription" | "Mutation") {
                let operation_type = obj_name.into();
                Some(RootOperationTypeDefinition {
                    operation_type,
                    named_type: Type::Named {
                        name: obj_name.to_string(),
                        loc: None,
                    },
                    loc: None,
                })
            } else {
                None
            }
        })
        .collect();

    Arc::new(type_defs)
}

fn implements_interfaces(
    implements_interfaces: Option<ast::ImplementsInterfaces>,
    file_id: FileId,
) -> Arc<Vec<ImplementsInterface>> {
    let interfaces: Vec<ImplementsInterface> = implements_interfaces
        .iter()
        .flat_map(|interfaces| {
            let types: Vec<ImplementsInterface> = interfaces
                .named_types()
                .map(|n| {
                    let name = n.name().expect("Name must have text");
                    ImplementsInterface {
                        interface: name_hir_node(name, file_id),
                        loc: location(file_id, n.syntax()),
                    }
                })
                .collect();
            types
        })
        .collect();

    Arc::new(interfaces)
}

fn fields_definition(
    fields_definition: Option<ast::FieldsDefinition>,
    file_id: FileId,
) -> Arc<Vec<FieldDefinition>> {
    match fields_definition {
        Some(fields_def) => {
            let fields: Vec<FieldDefinition> = fields_def
                .field_definitions()
                .map(|f| field_definition(f, file_id))
                .collect();
            Arc::new(fields)
        }
        None => Arc::new(Vec::new()),
    }
}

fn field_definition(field: ast::FieldDefinition, file_id: FileId) -> FieldDefinition {
    let description = description(field.description());
    let name = name(field.name(), file_id);
    let arguments = arguments_definition(field.arguments_definition(), file_id);
    let ty = ty(field.ty().expect("Field must have a type"), file_id);
    let directives = directives(field.directives(), file_id);
    let loc = location(file_id, field.syntax());

    FieldDefinition {
        description,
        name,
        arguments,
        ty,
        directives,
        loc,
    }
}

fn arguments_definition(
    arguments_definition: Option<ast::ArgumentsDefinition>,
    file_id: FileId,
) -> ArgumentsDefinition {
    match arguments_definition {
        Some(arguments) => {
            let input_values =
                input_value_definitions(arguments.input_value_definitions(), file_id);
            let loc = location(file_id, arguments.syntax());

            ArgumentsDefinition {
                input_values,
                loc: Some(loc),
            }
        }
        None => ArgumentsDefinition {
            input_values: Arc::new(Vec::new()),
            loc: None,
        },
    }
}

fn input_fields_definition(
    input_fields: Option<ast::InputFieldsDefinition>,
    file_id: FileId,
) -> Arc<Vec<InputValueDefinition>> {
    match input_fields {
        Some(fields) => input_value_definitions(fields.input_value_definitions(), file_id),
        None => Arc::new(Vec::new()),
    }
}

fn input_value_definitions(
    input_values: AstChildren<ast::InputValueDefinition>,
    file_id: FileId,
) -> Arc<Vec<InputValueDefinition>> {
    let input_values: Vec<InputValueDefinition> = input_values
        .map(|input| {
            let description = description(input.description());
            let name = name(input.name(), file_id);
            let ty = ty(
                input.ty().expect("Input Definition must have a type"),
                file_id,
            );
            let default_value = default_value(input.default_value(), file_id);
            let directives = directives(input.directives(), file_id);
            let loc = location(file_id, input.syntax());

            InputValueDefinition {
                description,
                name,
                ty,
                default_value,
                directives,
                loc: Some(loc),
            }
        })
        .collect();
    Arc::new(input_values)
}

fn default_value(
    default_value: Option<ast::DefaultValue>,
    file_id: FileId,
) -> Option<DefaultValue> {
    default_value.map(|val| {
        value(
            val.value().expect("Default Value must have a value token"),
            file_id,
        )
    })
}

fn root_operation_type_definition(
    root_type_def: AstChildren<ast::RootOperationTypeDefinition>,
    file_id: FileId,
) -> Arc<Vec<RootOperationTypeDefinition>> {
    let type_defs: Vec<RootOperationTypeDefinition> = root_type_def
        .into_iter()
        .map(|ty| {
            let operation_type = operation_type(ty.operation_type());
            let named_type = named_type(
                ty.named_type()
                    .expect("Root Operation Type Definition must have Named Type.")
                    .name(),
                file_id,
            );
            let loc = location(file_id, ty.syntax());

            RootOperationTypeDefinition {
                operation_type,
                named_type,
                loc: Some(loc),
            }
        })
        .collect();

    Arc::new(type_defs)
}

fn operation_type(op_type: Option<ast::OperationType>) -> OperationType {
    match op_type {
        Some(ty) => {
            if ty.query_token().is_some() {
                OperationType::Query
            } else if ty.mutation_token().is_some() {
                OperationType::Mutation
            } else if ty.subscription_token().is_some() {
                OperationType::Subscription
            } else {
                OperationType::Query
            }
        }
        None => OperationType::Query,
    }
}

fn variable_definitions(
    variable_definitions: Option<ast::VariableDefinitions>,
    file_id: FileId,
) -> Arc<Vec<VariableDefinition>> {
    match variable_definitions {
        Some(vars) => {
            let variable_definitions = vars
                .variable_definitions()
                .into_iter()
                .map(|v| variable_definition(v, file_id))
                .collect();
            Arc::new(variable_definitions)
        }
        None => Arc::new(Vec::new()),
    }
}

fn variable_definition(var: ast::VariableDefinition, file_id: FileId) -> VariableDefinition {
    let name = name(
        var.variable()
            .expect("values::Variable Definition must have a variable")
            .name(),
        file_id,
    );
    let directives = directives(var.directives(), file_id);
    let default_value = default_value(var.default_value(), file_id);
    let ty = ty(
        var.ty()
            .expect("values::Variable Definition must have a type"),
        file_id,
    );
    let loc = location(file_id, var.syntax());

    VariableDefinition {
        name,
        directives,
        ty,
        default_value,
        loc,
    }
}

fn ty(ty_: ast::Type, file_id: FileId) -> Type {
    match ty_ {
        ast::Type::NamedType(name) => named_type(name.name(), file_id),
        ast::Type::ListType(list) => Type::List {
            ty: Box::new(ty(list.ty().expect("List Type must have a type"), file_id)),
            loc: Some(location(file_id, list.syntax())),
        },
        ast::Type::NonNullType(non_null) => {
            if let Some(n) = non_null.named_type() {
                let named_type = named_type(n.name(), file_id);
                Type::NonNull {
                    ty: Box::new(named_type),
                    loc: Some(location(file_id, n.syntax())),
                }
            } else if let Some(list) = non_null.list_type() {
                let list_type = Type::List {
                    ty: Box::new(ty(list.ty().expect("List Type must have a type"), file_id)),
                    loc: Some(location(file_id, list.syntax())),
                };
                Type::NonNull {
                    ty: Box::new(list_type),
                    loc: Some(location(file_id, list.syntax())),
                }
            } else {
                // TODO: parser should have caught an error if there wasn't
                // either a named type or list type. Figure out a graceful way
                // to surface this error from the parser.
                panic!("Parser should have caught this error");
            }
        }
    }
}

fn named_type(n: Option<ast::Name>, file_id: FileId) -> Type {
    let name = n.expect("Named Type must have a name");

    Type::Named {
        name: name.text().to_string(),
        loc: Some(location(file_id, name.syntax())),
    }
}

fn directive_locations(
    directive_locations: Option<ast::DirectiveLocations>,
) -> Arc<Vec<DirectiveLocation>> {
    match directive_locations {
        Some(directive_loc) => {
            let locations: Vec<DirectiveLocation> = directive_loc
                .directive_locations()
                .into_iter()
                .map(|loc| loc.into())
                .collect();
            Arc::new(locations)
        }
        None => Arc::new(Vec::new()),
    }
}

fn directives(directives: Option<ast::Directives>, file_id: FileId) -> Arc<Vec<Directive>> {
    match directives {
        Some(directives) => {
            let directives = directives
                .directives()
                .into_iter()
                .map(|d| directive(d, file_id))
                .collect();
            Arc::new(directives)
        }
        None => Arc::new(Vec::new()),
    }
}

fn directive(directive: ast::Directive, file_id: FileId) -> Directive {
    let name = name(directive.name(), file_id);
    let arguments = arguments(directive.arguments(), file_id);
    let loc = location(file_id, directive.syntax());

    Directive {
        name,
        arguments,
        loc,
    }
}

fn arguments(arguments: Option<ast::Arguments>, file_id: FileId) -> Arc<Vec<Argument>> {
    match arguments {
        Some(arguments) => {
            let arguments = arguments
                .arguments()
                .into_iter()
                .map(|a| argument(a, file_id))
                .collect();
            Arc::new(arguments)
        }
        None => Arc::new(Vec::new()),
    }
}

fn argument(argument: ast::Argument, file_id: FileId) -> Argument {
    let name = name(argument.name(), file_id);
    let value = value(
        argument.value().expect("Argument must have a value"),
        file_id,
    );
    let loc = location(file_id, argument.syntax());

    Argument { name, value, loc }
}

fn value(val: ast::Value, file_id: FileId) -> Value {
    match val {
        ast::Value::Variable(var) => Value::Variable(Variable {
            name: var
                .name()
                .expect("Variable must have text")
                .text()
                .to_string(),
            loc: location(file_id, var.syntax()),
        }),
        ast::Value::StringValue(string_val) => Value::String(string_val.into()),
        // TODO(@goto-bus-stop) do not unwrap
        ast::Value::FloatValue(float) => Value::Float(Float::new(float.try_into().unwrap())),
        ast::Value::IntValue(int) => Value::Int(int.try_into().unwrap()),
        ast::Value::BooleanValue(bool) => Value::Boolean(bool.try_into().unwrap()),
        ast::Value::NullValue(_) => Value::Null,
        ast::Value::EnumValue(enum_) => Value::Enum(name(enum_.name(), file_id)),
        ast::Value::ListValue(list) => {
            let list: Vec<Value> = list.values().map(|v| value(v, file_id)).collect();
            Value::List(list)
        }
        ast::Value::ObjectValue(object) => {
            let object_values: Vec<(Name, Value)> = object
                .object_fields()
                .map(|o| {
                    let name = name(o.name(), file_id);
                    let value = value(o.value().expect("Object Value must have a value"), file_id);
                    (name, value)
                })
                .collect();
            Value::Object(object_values)
        }
    }
}

fn selection_set(
    db: &dyn HirDatabase,
    selections: Option<ast::SelectionSet>,
    parent_obj_ty: Option<String>,
    file_id: FileId,
) -> SelectionSet {
    let selection_set = match selections {
        Some(sel) => sel
            .selections()
            .into_iter()
            .map(|sel| selection(db, sel, parent_obj_ty.as_ref().cloned(), file_id))
            .collect(),
        None => Vec::new(),
    };

    SelectionSet {
        selection: Arc::new(selection_set),
    }
}

fn selection(
    db: &dyn HirDatabase,
    selection: ast::Selection,
    parent_obj_ty: Option<String>,
    file_id: FileId,
) -> Selection {
    match selection {
        ast::Selection::Field(sel_field) => {
            let field = field(db, sel_field, parent_obj_ty, file_id);
            Selection::Field(field)
        }
        ast::Selection::FragmentSpread(fragment) => {
            let fragment_spread = fragment_spread(fragment, file_id);
            Selection::FragmentSpread(fragment_spread)
        }
        ast::Selection::InlineFragment(fragment) => {
            let inline_fragment = inline_fragment(db, fragment, parent_obj_ty, file_id);
            Selection::InlineFragment(inline_fragment)
        }
    }
}

fn inline_fragment(
    db: &dyn HirDatabase,
    fragment: ast::InlineFragment,
    parent_obj: Option<String>,
    file_id: FileId,
) -> Arc<InlineFragment> {
    let type_condition = fragment.type_condition().map(|tc| {
        let tc = tc
            .named_type()
            .expect("Type Condition must have a name")
            .name()
            .expect("Name must have text");
        name_hir_node(tc, file_id)
    });
    let directives = directives(fragment.directives(), file_id);
    let new_parent_obj = if let Some(type_condition) = type_condition.clone() {
        Some(type_condition.src().to_string())
    } else {
        parent_obj
    };
    let selection_set: SelectionSet =
        selection_set(db, fragment.selection_set(), new_parent_obj, file_id);
    let loc = location(file_id, fragment.syntax());

    let fragment_data = InlineFragment {
        type_condition,
        directives,
        selection_set,
        loc,
    };
    Arc::new(fragment_data)
}

fn fragment_spread(fragment: ast::FragmentSpread, file_id: FileId) -> Arc<FragmentSpread> {
    let name = name(
        fragment
            .fragment_name()
            .expect("Fragment Spread must have a name")
            .name(),
        file_id,
    );
    let directives = directives(fragment.directives(), file_id);
    let loc = location(file_id, fragment.syntax());

    let fragment_data = FragmentSpread {
        name,
        directives,
        loc,
    };
    Arc::new(fragment_data)
}

fn field(
    db: &dyn HirDatabase,
    field: ast::Field,
    parent_obj: Option<String>,
    file_id: FileId,
) -> Arc<Field> {
    let name = name(field.name(), file_id);
    let alias = alias(field.alias());
    let new_parent_obj = parent_ty(db, name.src(), parent_obj.clone());
    let selection_set = selection_set(db, field.selection_set(), new_parent_obj, file_id);
    let directives = directives(field.directives(), file_id);
    let arguments = arguments(field.arguments(), file_id);
    let loc = location(file_id, field.syntax());

    let field_data = Field {
        name,
        alias,
        selection_set,
        parent_obj,
        directives,
        arguments,
        loc,
    };
    Arc::new(field_data)
}

fn parent_ty(db: &dyn HirDatabase, field_name: &str, parent_obj: Option<String>) -> Option<String> {
    if let Some(name) = parent_obj {
        db.type_system_definitions().iter().find_map(|def| {
            if let Some(n) = def.name() {
                if name == n {
                    return Some(def.field(field_name)?.ty().name());
                }
            }
            None
        })
    } else {
        None
    }
}

fn name(name: Option<ast::Name>, file_id: FileId) -> Name {
    name_hir_node(name.expect("Field must have a name"), file_id)
}

fn name_hir_node(name: ast::Name, file_id: FileId) -> Name {
    Name {
        src: name.text().to_string(),
        loc: Some(location(file_id, name.syntax())),
    }
}

fn enum_value(enum_value: Option<ast::EnumValue>, file_id: FileId) -> Name {
    let name = enum_value
        .expect("Enum value must have a name")
        .name()
        .expect("Name must have text");
    name_hir_node(name, file_id)
}

fn description(description: Option<ast::Description>) -> Option<String> {
    description.map(|desc| {
        desc.string_value()
            .expect("Description must have text")
            .into()
    })
}

fn alias(alias: Option<ast::Alias>) -> Option<Arc<Alias>> {
    alias.map(|alias| {
        let name = alias
            .name()
            .expect("Alias must have a name")
            .text()
            .to_string();
        let alias_data = Alias(name);
        Arc::new(alias_data)
    })
}

fn location(file_id: FileId, syntax_node: &SyntaxNode) -> HirNodeLocation {
    let text_range = syntax_node.text_range();

    HirNodeLocation {
        offset: text_range.start().into(),
        node_len: text_range.len().into(),
        file_id,
    }
}

//  Int, Float, String, Boolean, and ID
fn built_in_scalars(mut scalars: Vec<ScalarTypeDefinition>) -> Vec<ScalarTypeDefinition> {
    scalars.push(int_scalar());
    scalars.push(float_scalar());
    scalars.push(string_scalar());
    scalars.push(boolean_scalar());
    scalars.push(id_scalar());

    scalars
}

fn int_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.".into()),
        name: "Int".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true
    }
}

fn float_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).".into()),
        name: "Float".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true
    }
}

fn string_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.".into()),
        name: "String".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true
    }
}

fn boolean_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `Boolean` scalar type represents `true` or `false`.".into()),
        name: "Boolean".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
    }
}

fn id_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        id: Uuid::new_v4(),
        description: Some("The `ID` scalar type represents a unique identifier, often used to refetch an object or as key for a cache. The ID type appears in a JSON response as a String; however, it is not intended to be human-readable. When expected as an input type, any string (such as `\"4\"`) or integer (such as `4`) input value will be accepted as an ID.".into()),
        name: "ID".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true
    }
}

fn built_in_directives(mut directives: Vec<DirectiveDefinition>) -> Vec<DirectiveDefinition> {
    if !directives.iter().any(|dir| dir.name() == "skip") {
        directives.push(skip_directive());
    }

    if !directives.iter().any(|dir| dir.name() == "specifiedBy") {
        directives.push(specified_by_directive());
    }

    if !directives.iter().any(|dir| dir.name() == "deprecated") {
        directives.push(deprecated_directive());
    }

    if !directives.iter().any(|dir| dir.name() == "include") {
        directives.push(include_directive());
    }

    directives
}

fn skip_directive() -> DirectiveDefinition {
    // "Directs the executor to skip this field or fragment when the `if` argument is true."
    // directive @skip(
    //   "Skipped when true."
    //   if: Boolean!
    // ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some(
            "Directs the executor to skip this field or fragment when the `if` argument is true."
                .into(),
        ),
        name: "skip".to_string().into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some("Skipped when true.".into()),
                name: "if".to_string().into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "Boolean".into(),
                        loc: None,
                    }),
                    loc: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                loc: None,
            }]),
            loc: None,
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::Field,
            DirectiveLocation::FragmentSpread,
            DirectiveLocation::InlineFragment,
        ]),
        loc: None,
    }
}

fn specified_by_directive() -> DirectiveDefinition {
    // "Exposes a URL that specifies the behaviour of this scalar."
    // directive @specifiedBy(
    //     "The URL that specifies the behaviour of this scalar."
    //     url: String!
    // ) on SCALAR
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Exposes a URL that specifies the behaviour of this scalar.".into()),
        name: "specifiedBy".to_string().into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some("The URL that specifies the behaviour of this scalar.".into()),
                name: "url".to_string().into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "String".into(),
                        loc: None,
                    }),
                    loc: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                loc: None,
            }]),
            loc: None,
        },
        repeatable: false,
        directive_locations: Arc::new(vec![DirectiveLocation::Scalar]),
        loc: None,
    }
}

fn deprecated_directive() -> DirectiveDefinition {
    // "Marks an element of a GraphQL schema as no longer supported."
    // directive @deprecated(
    //   """
    //   Explains why this element was deprecated, usually also including a
    //   suggestion for how to access supported similar data. Formatted using
    //   the Markdown syntax, as specified by
    //   [CommonMark](https://commonmark.org/).
    //   """
    //   reason: String = "No longer supported"
    // ) on FIELD_DEFINITION | ENUM_VALUE
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Marks an element of a GraphQL schema as no longer supported.".into()),
        name: "deprecated".to_string().into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some(
                    "Explains why this element was deprecated, usually also including a suggestion for how to access supported similar data. Formatted using the Markdown syntax, as specified by [CommonMark](https://commonmark.org/).".into(),
                ),
                name: "reason".to_string().into(),
                ty: Type::Named {
                    name: "String".into(),
                    loc: None,
                },
                default_value: Some(DefaultValue::String("No longer supported".into())),
                directives: Arc::new(Vec::new()),
                loc: None
            }]),
            loc: None
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::FieldDefinition,
            DirectiveLocation::EnumValue
        ]),
        loc: None
    }
}

fn include_directive() -> DirectiveDefinition {
    // "Directs the executor to include this field or fragment only when the `if` argument is true."
    // directive @include(
    //   "Included when true."
    //   if: Boolean!
    // ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
    DirectiveDefinition {
        id: Uuid::new_v4(),
        description: Some("Directs the executor to include this field or fragment only when the `if` argument is true.".into()),
        name: "include".to_string().into(),
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: Some(
                    "Included when true.".into(),
                ),
                name: "if".to_string().into(),
                ty: Type::NonNull {
                    ty: Box::new(Type::Named {
                        name: "Boolean".into(),
                        loc: None,
                    }),
                    loc: None,
                },
                default_value: None,
                directives: Arc::new(Vec::new()),
                loc: None
            }]),
            loc: None
        },
        repeatable: false,
        directive_locations: Arc::new(vec![
            DirectiveLocation::Field,
            DirectiveLocation::FragmentSpread,
            DirectiveLocation::InlineFragment,
        ]),
        loc: None
    }
}
