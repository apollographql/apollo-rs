use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use apollo_parser::{
    ast::{self, AstChildren, AstNode},
    SyntaxNode,
};

use crate::{
    database::{document::*, FileId},
    hir::*,
    AstDatabase, InputDatabase,
};
use indexmap::IndexMap;

// HIR creators *ignore* missing data entirely. *Only* missing data
// as a result of parser errors should be ignored.

#[salsa::query_group(HirStorage)]
pub trait HirDatabase: InputDatabase + AstDatabase {
    /// Return all type system definitions defined in the compiler.
    fn type_system_definitions(&self) -> Arc<TypeSystemDefinitions>;

    /// Return all the operations defined in a file.
    fn operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all the fragments defined in a file.
    fn fragments(&self, file_id: FileId) -> ByName<FragmentDefinition>;

    /// Return all the operations defined in any file.
    fn all_operations(&self) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all the fragments defined in any file.
    fn all_fragments(&self) -> ByName<FragmentDefinition>;

    /// Return schema definition defined in the compiler.
    fn schema(&self) -> Arc<SchemaDefinition>;

    /// Return all object type definitions defined in the compiler.
    fn object_types(&self) -> ByName<ObjectTypeDefinition>;

    /// Return all scalar type definitions defined in the compiler.
    fn scalars(&self) -> ByName<ScalarTypeDefinition>;

    /// Return all enum type definitions defined in the compiler.
    fn enums(&self) -> ByName<EnumTypeDefinition>;

    /// Return all union type definitions defined in the compiler.
    fn unions(&self) -> ByName<UnionTypeDefinition>;

    /// Return all interface type definitions defined in the compiler.
    fn interfaces(&self) -> ByName<InterfaceTypeDefinition>;

    /// Return all directive definitions defined in the compiler.
    fn directive_definitions(&self) -> ByName<DirectiveDefinition>;

    /// Return all input object type definitions defined in the compiler.
    fn input_objects(&self) -> ByName<InputObjectTypeDefinition>;

    // Derived from above queries:
    /// Return an operation definition corresponding to the name and file id.
    fn find_operation_by_name(
        &self,
        file_id: FileId,
        name: String,
    ) -> Option<Arc<OperationDefinition>>;

    /// Return an fragment definition corresponding to the name and file id.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_fragment_by_name(
        &self,
        file_id: FileId,
        name: String,
    ) -> Option<Arc<FragmentDefinition>>;

    /// Return an object type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_object_type_by_name(&self, name: String) -> Option<Arc<ObjectTypeDefinition>>;

    /// Return an union type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_union_by_name(&self, name: String) -> Option<Arc<UnionTypeDefinition>>;

    /// Return an enum type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_enum_by_name(&self, name: String) -> Option<Arc<EnumTypeDefinition>>;

    /// Return an interface type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_interface_by_name(&self, name: String) -> Option<Arc<InterfaceTypeDefinition>>;

    /// Return an directive definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_directive_definition_by_name(&self, name: String) -> Option<Arc<DirectiveDefinition>>;

    /// Return any type definitions that contain the corresponding directive
    fn find_types_with_directive(&self, directive: String) -> Arc<Vec<TypeDefinition>>;

    /// Return an input object type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_input_object_by_name(&self, name: String) -> Option<Arc<InputObjectTypeDefinition>>;

    fn types_definitions_by_name(&self) -> Arc<IndexMap<String, TypeDefinition>>;

    /// Return a type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    fn find_type_definition_by_name(&self, name: String) -> Option<TypeDefinition>;

    /// Return all query operations in a corresponding file.
    fn query_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all mutation operations in a corresponding file.
    fn mutation_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all subscription operations in a corresponding file.
    fn subscription_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all operation fields in a corresponding selection set.
    fn operation_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return all operation inline fragment fields in a corresponding selection set.
    fn operation_inline_fragment_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return all operation fragment spread fields in a corresponding selection set.
    fn operation_fragment_spread_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return all variables in a corresponding selection set.
    fn selection_variables(&self, selection_set: SelectionSet) -> Arc<HashSet<Variable>>;

    /// Return all variables in corresponding variable definitions.
    fn operation_definition_variables(
        &self,
        variables: Arc<Vec<VariableDefinition>>,
    ) -> Arc<HashSet<Variable>>;

    /// Return a subtype map of the current compiler's type system.
    ///
    /// Given the following schema,
    /// ```graphql
    /// type Query {
    ///   me: String
    /// }
    /// type Foo {
    ///   me: String
    /// }
    /// type Bar {
    ///   me: String
    /// }
    /// union UnionType = Foo | Bar
    ///
    /// interface Baz {
    ///   me: String,
    /// }
    /// type ObjectType implements Baz { me: String }
    /// interface InterfaceType implements Baz { me: String }
    /// ```
    /// we can say that:
    ///
    /// - `Foo` and `Bar` are a subtypes of `UnionType`.
    /// - `ObjectType` and `InterfaceType` are subtypes of `Baz`.
    fn subtype_map(&self) -> Arc<HashMap<String, HashSet<String>>>;

    /// Return `true` if the provided `maybe_subtype` is a subtype of the
    /// corresponding `abstract_type`.
    ///
    /// Given the following schema,
    /// ```graphql
    /// type Query {
    ///   me: String
    /// }
    /// type Foo {
    ///   me: String
    /// }
    /// type Bar {
    ///   me: String
    /// }
    /// union UnionType = Foo | Bar
    ///
    /// interface Baz {
    ///   me: String,
    /// }
    /// type ObjectType implements Baz { me: String }
    /// interface InterfaceType implements Baz { me: String }
    /// ```
    /// we can say that:
    ///
    /// - `db.is_subtype("UnionType".into(), "Foo".into()) // true`
    /// - `db.is_subtype("UnionType".into(), "Bar".into()) // true`
    /// - `db.is_subtype("Baz".into(), "ObjectType".into()) // true`
    /// - `db.is_subtype("Baz".into(), "InterfaceType".into()) // true`
    #[salsa::transparent]
    fn is_subtype(&self, abstract_type: String, maybe_subtype: String) -> bool;
}

fn type_system_definitions(db: &dyn HirDatabase) -> Arc<TypeSystemDefinitions> {
    Arc::new(TypeSystemDefinitions {
        schema: db.schema(),
        scalars: db.scalars(),
        objects: db.object_types(),
        interfaces: db.interfaces(),
        unions: db.unions(),
        enums: db.enums(),
        input_objects: db.input_objects(),
        directives: db.directive_definitions(),
    })
}

fn operations(db: &dyn HirDatabase, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>> {
    Arc::new(
        db.ast(file_id)
            .document()
            .syntax()
            .children()
            .filter_map(ast::OperationDefinition::cast)
            .filter_map(|def| operation_definition(db, def, file_id))
            .map(Arc::new)
            .collect(),
    )
}

fn fragments(db: &dyn HirDatabase, file_id: FileId) -> ByName<FragmentDefinition> {
    let mut map = IndexMap::new();
    for def in db
        .ast(file_id)
        .document()
        .syntax()
        .children()
        .filter_map(ast::FragmentDefinition::cast)
        .filter_map(|def| fragment_definition(db, def, file_id))
    {
        let name = def.name().to_owned();
        map.entry(name).or_insert_with(|| Arc::new(def));
    }
    Arc::new(map)
}

fn all_operations(db: &dyn HirDatabase) -> Arc<Vec<Arc<OperationDefinition>>> {
    let mut operations = Vec::new();
    for file_id in db.executable_definition_files() {
        operations.extend(db.operations(file_id).iter().cloned())
    }
    Arc::new(operations)
}

fn all_fragments(db: &dyn HirDatabase) -> ByName<FragmentDefinition> {
    let mut fragments = IndexMap::new();
    for file_id in db.executable_definition_files() {
        for (name, def) in db.fragments(file_id).iter() {
            fragments.entry(name.clone()).or_insert_with(|| def.clone());
        }
    }
    Arc::new(fragments)
}

/// Takes a fallible conversion from a specific AST type to an HIR type,
/// finds matching top-level AST nodes in type definition files,
/// and returns an iterator of successful conversion results.
///
/// Failed conversions are ignored.
fn type_definitions<'db, AstType, TryConvert, HirType>(
    db: &'db dyn HirDatabase,
    try_convert: TryConvert,
) -> impl Iterator<Item = HirType> + 'db
where
    AstType: 'db + ast::AstNode,
    TryConvert: 'db + Copy + Fn(&dyn HirDatabase, AstType, FileId) -> Option<HirType>,
{
    db.type_definition_files()
        .into_iter()
        .flat_map(move |file_id| {
            db.ast(file_id)
                .document()
                .syntax()
                .children()
                .filter_map(AstNode::cast)
                .filter_map(move |def| try_convert(db, def, file_id))
        })
}

// FIXME(@lrlna): if our compiler is composed of multiple documents that for
// some reason have more than one schema definition, we should be raising an
// error.
//
// This implementation currently just finds the first schema definition, which
// means we can't really diagnose the "multiple schema definitions" errors.
fn schema(db: &dyn HirDatabase) -> Arc<SchemaDefinition> {
    let mut schema_def = type_definitions(db, schema_definition)
        .next()
        .unwrap_or_default();

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

macro_rules! by_name {
    ($db: ident, $convert: expr) => {{
        let mut map = IndexMap::new();
        for def in type_definitions($db, $convert) {
            let name = def.name().to_owned();
            map.entry(name).or_insert_with(|| Arc::new(def));
        }
        map
    }};
}

macro_rules! by_name_extensible {
    ($db: ident, $convert: expr, $convert_extension: expr) => {{
        let mut map = by_name!($db, $convert);
        for ext in type_definitions($db, $convert_extension) {
            if let Some(def) = map.get_mut(ext.name()) {
                Arc::get_mut(def).unwrap().extensions.push(ext)
            } else {
                // TODO: record orphan extensions for validation purpose?
            }
        }
        map
    }};
}

fn object_types(db: &dyn HirDatabase) -> ByName<ObjectTypeDefinition> {
    Arc::new(by_name_extensible!(
        db,
        object_type_definition,
        object_type_extension
    ))
}

fn scalars(db: &dyn HirDatabase) -> ByName<ScalarTypeDefinition> {
    Arc::new(built_in_scalars(by_name_extensible!(
        db,
        scalar_definition,
        scalar_extension
    )))
}

fn enums(db: &dyn HirDatabase) -> ByName<EnumTypeDefinition> {
    Arc::new(by_name_extensible!(db, enum_definition, enum_extension))
}

fn unions(db: &dyn HirDatabase) -> ByName<UnionTypeDefinition> {
    Arc::new(by_name_extensible!(db, union_definition, union_extension))
}

fn interfaces(db: &dyn HirDatabase) -> ByName<InterfaceTypeDefinition> {
    Arc::new(by_name_extensible!(
        db,
        interface_definition,
        interface_extension
    ))
}

fn input_objects(db: &dyn HirDatabase) -> ByName<InputObjectTypeDefinition> {
    Arc::new(by_name_extensible!(
        db,
        input_object_definition,
        input_object_extension
    ))
}

fn directive_definitions(db: &dyn HirDatabase) -> ByName<DirectiveDefinition> {
    Arc::new(built_in_directives(by_name!(db, directive_definition)))
}

fn operation_definition(
    db: &dyn HirDatabase,
    op_def: ast::OperationDefinition,
    file_id: FileId,
) -> Option<OperationDefinition> {
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

    Some(OperationDefinition {
        operation_ty: ty,
        name,
        variables,
        selection_set,
        directives,
        loc,
    })
}

fn fragment_definition(
    db: &dyn HirDatabase,
    fragment_def: ast::FragmentDefinition,
    file_id: FileId,
) -> Option<FragmentDefinition> {
    let name = name(fragment_def.fragment_name()?.name(), file_id)?;
    let type_condition = fragment_def
        .type_condition()?
        .named_type()?
        .name()?
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

    Some(FragmentDefinition {
        name,
        type_condition,
        selection_set,
        directives,
        loc,
    })
}

fn schema_definition(
    db: &dyn HirDatabase,
    schema_def: ast::SchemaDefinition,
    file_id: FileId,
) -> Option<SchemaDefinition> {
    let extensions = type_definitions(db, |_db, def: ast::SchemaExtension, file_id| {
        Some(SchemaExtension {
            directives: directives(def.directives(), file_id),
            root_operation_type_definition: root_operation_type_definition(
                def.root_operation_type_definitions(),
                file_id,
            ),
            loc: location(file_id, def.syntax()),
        })
    })
    .collect();

    let description = description(schema_def.description());
    let directives = directives(schema_def.directives(), file_id);
    let root_operation_type_definition =
        root_operation_type_definition(schema_def.root_operation_type_definitions(), file_id);
    let loc = location(file_id, schema_def.syntax());

    Some(SchemaDefinition {
        description,
        directives,
        root_operation_type_definition,
        loc: Some(loc),
        extensions,
    })
}

fn object_type_definition(
    _db: &dyn HirDatabase,
    obj_def: ast::ObjectTypeDefinition,
    file_id: FileId,
) -> Option<ObjectTypeDefinition> {
    let description = description(obj_def.description());
    let name = name(obj_def.name(), file_id)?;
    let implements_interfaces = implements_interfaces(obj_def.implements_interfaces(), file_id);
    let directives = directives(obj_def.directives(), file_id);
    let fields_definition = fields_definition(obj_def.fields_definition(), file_id);
    let loc = location(file_id, obj_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(ObjectTypeDefinition {
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        loc,
        extensions: Vec::new(),
    })
}

fn object_type_extension(
    _db: &dyn HirDatabase,
    def: ast::ObjectTypeExtension,
    file_id: FileId,
) -> Option<ObjectTypeExtension> {
    Some(ObjectTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        implements_interfaces: implements_interfaces(def.implements_interfaces(), file_id),
        fields_definition: fields_definition(def.fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    })
}

fn scalar_definition(
    _db: &dyn HirDatabase,
    scalar_def: ast::ScalarTypeDefinition,
    file_id: FileId,
) -> Option<ScalarTypeDefinition> {
    let description = description(scalar_def.description());
    let name = name(scalar_def.name(), file_id)?;
    let directives = directives(scalar_def.directives(), file_id);
    let loc = location(file_id, scalar_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(ScalarTypeDefinition {
        description,
        name,
        directives,
        loc: Some(loc),
        built_in: false,
        extensions: Vec::new(),
    })
}

fn scalar_extension(
    _db: &dyn HirDatabase,
    def: ast::ScalarTypeExtension,
    file_id: FileId,
) -> Option<ScalarTypeExtension> {
    Some(ScalarTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        loc: location(file_id, def.syntax()),
    })
}

fn enum_definition(
    _db: &dyn HirDatabase,
    enum_def: ast::EnumTypeDefinition,
    file_id: FileId,
) -> Option<EnumTypeDefinition> {
    let description = description(enum_def.description());
    let name = name(enum_def.name(), file_id)?;
    let directives = directives(enum_def.directives(), file_id);
    let enum_values_definition = enum_values_definition(enum_def.enum_values_definition(), file_id);
    let loc = location(file_id, enum_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(EnumTypeDefinition {
        description,
        name,
        directives,
        enum_values_definition,
        loc,
        extensions: Vec::new(),
    })
}

fn enum_extension(
    _db: &dyn HirDatabase,
    def: ast::EnumTypeExtension,
    file_id: FileId,
) -> Option<EnumTypeExtension> {
    Some(EnumTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        enum_values_definition: enum_values_definition(def.enum_values_definition(), file_id),
        loc: location(file_id, def.syntax()),
    })
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
                .filter_map(|e| enum_value_definition(e, file_id))
                .collect();
            Arc::new(enum_values)
        }
        None => Arc::new(Vec::new()),
    }
}

fn enum_value_definition(
    enum_value_def: ast::EnumValueDefinition,
    file_id: FileId,
) -> Option<EnumValueDefinition> {
    let description = description(enum_value_def.description());
    let enum_value = enum_value(enum_value_def.enum_value(), file_id)?;
    let directives = directives(enum_value_def.directives(), file_id);
    let loc = location(file_id, enum_value_def.syntax());

    Some(EnumValueDefinition {
        description,
        enum_value,
        directives,
        loc,
    })
}

fn union_definition(
    _db: &dyn HirDatabase,
    union_def: ast::UnionTypeDefinition,
    file_id: FileId,
) -> Option<UnionTypeDefinition> {
    let description = description(union_def.description());
    let name = name(union_def.name(), file_id)?;
    let directives = directives(union_def.directives(), file_id);
    let union_members = union_members(union_def.union_member_types(), file_id);
    let loc = location(file_id, union_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(UnionTypeDefinition {
        description,
        name,
        directives,
        union_members,
        loc,
        extensions: Vec::new(),
    })
}

fn union_extension(
    _db: &dyn HirDatabase,
    def: ast::UnionTypeExtension,
    file_id: FileId,
) -> Option<UnionTypeExtension> {
    Some(UnionTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        union_members: union_members(def.union_member_types(), file_id),
        loc: location(file_id, def.syntax()),
    })
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
                .filter_map(|u| union_member(u, file_id))
                .collect();
            Arc::new(mems)
        }
        None => Arc::new(Vec::new()),
    }
}

fn union_member(member: ast::NamedType, file_id: FileId) -> Option<UnionMember> {
    let name = name(member.name(), file_id)?;
    let loc = location(file_id, member.syntax());

    Some(UnionMember { name, loc })
}

fn interface_definition(
    _db: &dyn HirDatabase,
    interface_def: ast::InterfaceTypeDefinition,
    file_id: FileId,
) -> Option<InterfaceTypeDefinition> {
    let description = description(interface_def.description());
    let name = name(interface_def.name(), file_id)?;
    let implements_interfaces =
        implements_interfaces(interface_def.implements_interfaces(), file_id);
    let directives = directives(interface_def.directives(), file_id);
    let fields_definition = fields_definition(interface_def.fields_definition(), file_id);
    let loc = location(file_id, interface_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(InterfaceTypeDefinition {
        description,
        name,
        implements_interfaces,
        directives,
        fields_definition,
        loc,
        extensions: Vec::new(),
    })
}

fn interface_extension(
    _db: &dyn HirDatabase,
    def: ast::InterfaceTypeExtension,
    file_id: FileId,
) -> Option<InterfaceTypeExtension> {
    Some(InterfaceTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        implements_interfaces: implements_interfaces(def.implements_interfaces(), file_id),
        fields_definition: fields_definition(def.fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    })
}

fn directive_definition(
    _db: &dyn HirDatabase,
    directive_def: ast::DirectiveDefinition,
    file_id: FileId,
) -> Option<DirectiveDefinition> {
    let name = name(directive_def.name(), file_id)?;
    let description = description(directive_def.description());
    let arguments = arguments_definition(directive_def.arguments_definition(), file_id);
    let repeatable = directive_def.repeatable_token().is_some();
    let directive_locations = directive_locations(directive_def.directive_locations());
    let loc = location(file_id, directive_def.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(DirectiveDefinition {
        description,
        name,
        arguments,
        repeatable,
        directive_locations,
        loc: Some(loc),
    })
}

fn input_object_definition(
    _db: &dyn HirDatabase,
    input_obj: ast::InputObjectTypeDefinition,
    file_id: FileId,
) -> Option<InputObjectTypeDefinition> {
    let description = description(input_obj.description());
    let name = name(input_obj.name(), file_id)?;
    let directives = directives(input_obj.directives(), file_id);
    let input_fields_definition =
        input_fields_definition(input_obj.input_fields_definition(), file_id);
    let loc = location(file_id, input_obj.syntax());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(InputObjectTypeDefinition {
        description,
        name,
        directives,
        input_fields_definition,
        loc,
        extensions: Vec::new(),
    })
}

fn input_object_extension(
    _db: &dyn HirDatabase,
    def: ast::InputObjectTypeExtension,
    file_id: FileId,
) -> Option<InputObjectTypeExtension> {
    Some(InputObjectTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        input_fields_definition: input_fields_definition(def.input_fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    })
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
        .values()
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
                .filter_map(|n| {
                    let name = n.name()?;
                    Some(ImplementsInterface {
                        interface: name_hir_node(name, file_id),
                        loc: location(file_id, n.syntax()),
                    })
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
                .filter_map(|f| field_definition(f, file_id))
                .collect();
            Arc::new(fields)
        }
        None => Arc::new(Vec::new()),
    }
}

fn field_definition(field: ast::FieldDefinition, file_id: FileId) -> Option<FieldDefinition> {
    let description = description(field.description());
    let name = name(field.name(), file_id)?;
    let arguments = arguments_definition(field.arguments_definition(), file_id);
    let ty = ty(field.ty()?, file_id)?;
    let directives = directives(field.directives(), file_id);
    let loc = location(file_id, field.syntax());

    Some(FieldDefinition {
        description,
        name,
        arguments,
        ty,
        directives,
        loc,
    })
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
        .filter_map(|input| {
            let description = description(input.description());
            let name = name(input.name(), file_id)?;
            let ty = ty(input.ty()?, file_id)?;
            let default_value = default_value(input.default_value(), file_id);
            let directives = directives(input.directives(), file_id);
            let loc = location(file_id, input.syntax());

            Some(InputValueDefinition {
                description,
                name,
                ty,
                default_value,
                directives,
                loc: Some(loc),
            })
        })
        .collect();
    Arc::new(input_values)
}

fn default_value(
    default_value: Option<ast::DefaultValue>,
    file_id: FileId,
) -> Option<DefaultValue> {
    default_value
        .and_then(|val| val.value())
        .and_then(|val| value(val, file_id))
}

fn root_operation_type_definition(
    root_type_def: AstChildren<ast::RootOperationTypeDefinition>,
    file_id: FileId,
) -> Arc<Vec<RootOperationTypeDefinition>> {
    let type_defs: Vec<RootOperationTypeDefinition> = root_type_def
        .into_iter()
        .filter_map(|ty| {
            let operation_type = operation_type(ty.operation_type());
            let named_type = named_type(ty.named_type()?.name()?, file_id);
            let loc = location(file_id, ty.syntax());

            Some(RootOperationTypeDefinition {
                operation_type,
                named_type,
                loc: Some(loc),
            })
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
                .filter_map(|v| variable_definition(v, file_id))
                .collect();
            Arc::new(variable_definitions)
        }
        None => Arc::new(Vec::new()),
    }
}

fn variable_definition(
    var: ast::VariableDefinition,
    file_id: FileId,
) -> Option<VariableDefinition> {
    let name = name(var.variable()?.name(), file_id)?;
    let directives = directives(var.directives(), file_id);
    let default_value = default_value(var.default_value(), file_id);
    let ty = ty(var.ty()?, file_id)?;
    let loc = location(file_id, var.syntax());

    Some(VariableDefinition {
        name,
        directives,
        ty,
        default_value,
        loc,
    })
}

fn ty(ty_: ast::Type, file_id: FileId) -> Option<Type> {
    match ty_ {
        ast::Type::NamedType(name) => name.name().map(|name| named_type(name, file_id)),
        ast::Type::ListType(list) => Some(Type::List {
            ty: Box::new(ty(list.ty()?, file_id)?),
            loc: Some(location(file_id, list.syntax())),
        }),
        ast::Type::NonNullType(non_null) => {
            if let Some(n) = non_null.named_type() {
                let named_type = n.name().map(|name| named_type(name, file_id))?;
                Some(Type::NonNull {
                    ty: Box::new(named_type),
                    loc: Some(location(file_id, n.syntax())),
                })
            } else if let Some(list) = non_null.list_type() {
                let list_type = Type::List {
                    ty: Box::new(ty(list.ty()?, file_id)?),
                    loc: Some(location(file_id, list.syntax())),
                };
                Some(Type::NonNull {
                    ty: Box::new(list_type),
                    loc: Some(location(file_id, list.syntax())),
                })
            } else {
                // TODO: parser should have caught an error if there wasn't
                // either a named type or list type. Figure out a graceful way
                // to surface this error from the parser.
                panic!("Parser should have caught this error");
            }
        }
    }
}

fn named_type(name: ast::Name, file_id: FileId) -> Type {
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
                .filter_map(|d| directive(d, file_id))
                .collect();
            Arc::new(directives)
        }
        None => Arc::new(Vec::new()),
    }
}

fn directive(directive: ast::Directive, file_id: FileId) -> Option<Directive> {
    let name = name(directive.name(), file_id)?;
    let arguments = arguments(directive.arguments(), file_id);
    let loc = location(file_id, directive.syntax());

    Some(Directive {
        name,
        arguments,
        loc,
    })
}

fn arguments(arguments: Option<ast::Arguments>, file_id: FileId) -> Arc<Vec<Argument>> {
    match arguments {
        Some(arguments) => {
            let arguments = arguments
                .arguments()
                .into_iter()
                .filter_map(|a| argument(a, file_id))
                .collect();
            Arc::new(arguments)
        }
        None => Arc::new(Vec::new()),
    }
}

fn argument(argument: ast::Argument, file_id: FileId) -> Option<Argument> {
    let name = name(argument.name(), file_id)?;
    let value = value(argument.value()?, file_id)?;
    let loc = location(file_id, argument.syntax());

    Some(Argument { name, value, loc })
}

fn value(val: ast::Value, file_id: FileId) -> Option<Value> {
    let hir_val = match val {
        ast::Value::Variable(var) => Value::Variable(Variable {
            name: var.name()?.text().to_string(),
            loc: location(file_id, var.syntax()),
        }),
        ast::Value::StringValue(string_val) => Value::String(string_val.into()),
        // TODO(@goto-bus-stop) do not unwrap
        ast::Value::FloatValue(float) => Value::Float(Float::new(float.try_into().unwrap())),
        ast::Value::IntValue(int) => Value::Int(Float::new(f64::try_from(int).unwrap())),
        ast::Value::BooleanValue(bool) => Value::Boolean(bool.try_into().unwrap()),
        ast::Value::NullValue(_) => Value::Null,
        ast::Value::EnumValue(enum_) => Value::Enum(name(enum_.name(), file_id)?),
        ast::Value::ListValue(list) => {
            let list: Vec<Value> = list.values().filter_map(|v| value(v, file_id)).collect();
            Value::List(list)
        }
        ast::Value::ObjectValue(object) => {
            let object_values: Vec<(Name, Value)> = object
                .object_fields()
                .filter_map(|o| {
                    let name = name(o.name(), file_id)?;
                    let value = value(o.value()?, file_id)?;
                    Some((name, value))
                })
                .collect();
            Value::Object(object_values)
        }
    };
    Some(hir_val)
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
            .filter_map(|sel| selection(db, sel, parent_obj_ty.as_ref().cloned(), file_id))
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
) -> Option<Selection> {
    match selection {
        ast::Selection::Field(sel_field) => {
            field(db, sel_field, parent_obj_ty, file_id).map(Selection::Field)
        }
        ast::Selection::FragmentSpread(fragment) => {
            fragment_spread(fragment, file_id).map(Selection::FragmentSpread)
        }
        ast::Selection::InlineFragment(fragment) => Some(Selection::InlineFragment(
            inline_fragment(db, fragment, parent_obj_ty, file_id),
        )),
    }
}

fn inline_fragment(
    db: &dyn HirDatabase,
    fragment: ast::InlineFragment,
    parent_obj: Option<String>,
    file_id: FileId,
) -> Arc<InlineFragment> {
    let type_condition = fragment.type_condition().and_then(|tc| {
        let tc = tc.named_type()?.name()?;
        Some(name_hir_node(tc, file_id))
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

fn fragment_spread(fragment: ast::FragmentSpread, file_id: FileId) -> Option<Arc<FragmentSpread>> {
    let name = name(fragment.fragment_name()?.name(), file_id)?;
    let directives = directives(fragment.directives(), file_id);
    let loc = location(file_id, fragment.syntax());

    let fragment_data = FragmentSpread {
        name,
        directives,
        loc,
    };
    Some(Arc::new(fragment_data))
}

fn field(
    db: &dyn HirDatabase,
    field: ast::Field,
    parent_obj: Option<String>,
    file_id: FileId,
) -> Option<Arc<Field>> {
    let name = name(field.name(), file_id)?;
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
    Some(Arc::new(field_data))
}

fn parent_ty(db: &dyn HirDatabase, field_name: &str, parent_obj: Option<String>) -> Option<String> {
    Some(
        db.find_type_definition_by_name(parent_obj?)?
            .field(field_name)?
            .ty()
            .name(),
    )
}

fn name(name: Option<ast::Name>, file_id: FileId) -> Option<Name> {
    name.map(|name| name_hir_node(name, file_id))
}

fn name_hir_node(name: ast::Name, file_id: FileId) -> Name {
    Name {
        src: name.text().to_string(),
        loc: Some(location(file_id, name.syntax())),
    }
}

fn enum_value(enum_value: Option<ast::EnumValue>, file_id: FileId) -> Option<Name> {
    let name = enum_value?.name()?;
    Some(name_hir_node(name, file_id))
}

fn description(description: Option<ast::Description>) -> Option<String> {
    description.and_then(|desc| Some(desc.string_value()?.into()))
}

fn alias(alias: Option<ast::Alias>) -> Option<Arc<Alias>> {
    alias.and_then(|alias| {
        let name = alias.name()?.text().to_string();
        let alias_data = Alias(name);
        Some(Arc::new(alias_data))
    })
}

fn location(file_id: FileId, syntax_node: &SyntaxNode) -> HirNodeLocation {
    HirNodeLocation::new(file_id, syntax_node)
}

// Add `Int`, `Float`, `String`, `Boolean`, and `ID`
fn built_in_scalars(
    mut scalars: IndexMap<String, Arc<ScalarTypeDefinition>>,
) -> IndexMap<String, Arc<ScalarTypeDefinition>> {
    for built_in in [
        int_scalar(),
        float_scalar(),
        string_scalar(),
        boolean_scalar(),
        id_scalar(),
    ] {
        scalars
            .entry(built_in.name().to_owned())
            .or_insert_with(|| Arc::new(built_in));
    }
    scalars
}

fn int_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        description: Some("The `Int` scalar type represents non-fractional signed whole numeric values. Int can represent values between -(2^31) and 2^31 - 1.".into()),
        name: "Int".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
        extensions: Vec::new(),
    }
}

fn float_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        description: Some("The `Float` scalar type represents signed double-precision fractional values as specified by [IEEE 754](https://en.wikipedia.org/wiki/IEEE_floating_point).".into()),
        name: "Float".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
        extensions: Vec::new(),
    }
}

fn string_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        description: Some("The `String` scalar type represents textual data, represented as UTF-8 character sequences. The String type is most often used by GraphQL to represent free-form human-readable text.".into()),
        name: "String".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
        extensions: Vec::new(),
    }
}

fn boolean_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        description: Some("The `Boolean` scalar type represents `true` or `false`.".into()),
        name: "Boolean".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
        extensions: Vec::new(),
    }
}

fn id_scalar() -> ScalarTypeDefinition {
    ScalarTypeDefinition {
        description: Some("The `ID` scalar type represents a unique identifier, often used to refetch an object or as key for a cache. The ID type appears in a JSON response as a String; however, it is not intended to be human-readable. When expected as an input type, any string (such as `\"4\"`) or integer (such as `4`) input value will be accepted as an ID.".into()),
        name: "ID".to_string().into(),
        directives: Arc::new(Vec::new()),
        loc: None,
        built_in: true,
        extensions: Vec::new(),
    }
}

fn built_in_directives(
    mut directives: IndexMap<String, Arc<DirectiveDefinition>>,
) -> IndexMap<String, Arc<DirectiveDefinition>> {
    directives
        .entry("skip".to_owned())
        .or_insert_with(|| Arc::new(skip_directive()));
    directives
        .entry("specifiedBy".to_owned())
        .or_insert_with(|| Arc::new(specified_by_directive()));
    directives
        .entry("deprecated".to_owned())
        .or_insert_with(|| Arc::new(deprecated_directive()));
    directives
        .entry("include".to_owned())
        .or_insert_with(|| Arc::new(include_directive()));
    directives
}

fn skip_directive() -> DirectiveDefinition {
    // "Directs the executor to skip this field or fragment when the `if` argument is true."
    // directive @skip(
    //   "Skipped when true."
    //   if: Boolean!
    // ) on FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
    DirectiveDefinition {
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
