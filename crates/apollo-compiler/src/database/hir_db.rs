use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use apollo_parser::ast::AstChildren;
use apollo_parser::ast::AstNode;
use apollo_parser::ast::{self};
use apollo_parser::SyntaxNode;
use indexmap::IndexMap;

use crate::database::document;
use crate::database::FileId;
use crate::hir::*;
use crate::AstDatabase;
use crate::InputDatabase;

const INTROSPECTION_OBJECT_TYS: [&str; 6] = [
    "__Schema",
    "__Type",
    "__Field",
    "__InputValue",
    "__EnumValue",
    "__Directive",
];

const INTROSPECTION_ENUM_TYS: [&str; 2] = ["__TypeKind", "__DirectiveLocation"];

// HIR creators *ignore* missing data entirely. *Only* missing data
// as a result of parser errors should be ignored.

#[salsa::query_group(HirStorage)]
pub trait HirDatabase: InputDatabase + AstDatabase {
    /// Return all type system definitions defined in the compiler.
    #[salsa::invoke(type_system_definitions)]
    fn type_system_definitions(&self) -> Arc<TypeSystemDefinitions>;

    /// Return a [`TypeSystem`] containing definitions and more.
    ///
    /// This can be used with [`set_type_system_hir`][crate::ApolloCompiler::set_type_system_hir]
    /// on another compiler.
    #[salsa::invoke(type_system)]
    fn type_system(&self) -> Arc<TypeSystem>;

    /// Return all the extensions defined in the type system.
    #[salsa::invoke(extensions)]
    fn extensions(&self) -> Arc<Vec<TypeExtension>>;

    /// Return all the operations defined in a file.
    #[salsa::invoke(operations)]
    fn operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all the fragments defined in a file.
    #[salsa::invoke(fragments)]
    fn fragments(&self, file_id: FileId) -> ByName<FragmentDefinition>;

    /// Return all the operations defined in any file.
    #[salsa::invoke(all_operations)]
    fn all_operations(&self) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all the fragments defined in any file.
    #[salsa::invoke(all_fragments)]
    fn all_fragments(&self) -> ByName<FragmentDefinition>;

    /// Return schema definition defined in the compiler.
    #[salsa::invoke(schema)]
    fn schema(&self) -> Arc<SchemaDefinition>;

    /// Return all object type definitions defined in the compiler.
    #[salsa::invoke(object_types)]
    fn object_types(&self) -> ByName<ObjectTypeDefinition>;

    /// Return all object type definitions, including instrospection types like
    /// `__Schema`, defined in the compiler.
    fn object_types_with_built_ins(&self) -> ByName<ObjectTypeDefinition>;

    /// Return all scalar type definitions defined in the compiler.
    #[salsa::invoke(scalars)]
    fn scalars(&self) -> ByName<ScalarTypeDefinition>;

    /// Return all enum type definitions defined in the compiler.
    #[salsa::invoke(enums)]
    fn enums(&self) -> ByName<EnumTypeDefinition>;

    /// Return all enums, including introspection types like `__TypeKind`, defined
    /// in the compiler.
    fn enums_with_built_ins(&self) -> ByName<EnumTypeDefinition>;

    /// Return all union type definitions defined in the compiler.
    #[salsa::invoke(unions)]
    fn unions(&self) -> ByName<UnionTypeDefinition>;

    /// Return all interface type definitions defined in the compiler.
    #[salsa::invoke(interfaces)]
    fn interfaces(&self) -> ByName<InterfaceTypeDefinition>;

    /// Return all directive definitions defined in the compiler.
    #[salsa::invoke(directive_definitions)]
    fn directive_definitions(&self) -> ByName<DirectiveDefinition>;

    /// Return all input object type definitions defined in the compiler.
    #[salsa::invoke(input_objects)]
    fn input_objects(&self) -> ByName<InputObjectTypeDefinition>;

    // Derived from above queries:

    /// Return an operation definition corresponding to the name and file id.
    /// If `name` is `None`, and there is only one operation, that operation will
    /// be returned.
    /// If `name` is `None`, and there is more than one operation, `None` will
    /// be returned.
    #[salsa::invoke(document::find_operation)]
    fn find_operation(
        &self,
        file_id: FileId,
        name: Option<String>,
    ) -> Option<Arc<OperationDefinition>>;

    /// Return an fragment definition corresponding to the name and file id.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_fragment_by_name)]
    fn find_fragment_by_name(
        &self,
        file_id: FileId,
        name: String,
    ) -> Option<Arc<FragmentDefinition>>;

    /// Return an object type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_object_type_by_name)]
    fn find_object_type_by_name(&self, name: String) -> Option<Arc<ObjectTypeDefinition>>;

    /// Return an union type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_union_by_name)]
    fn find_union_by_name(&self, name: String) -> Option<Arc<UnionTypeDefinition>>;

    /// Return an enum type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_enum_by_name)]
    fn find_enum_by_name(&self, name: String) -> Option<Arc<EnumTypeDefinition>>;

    /// Return a scalar type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_scalar_by_name)]
    fn find_scalar_by_name(&self, name: String) -> Option<Arc<ScalarTypeDefinition>>;

    /// Return an interface type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_interface_by_name)]
    fn find_interface_by_name(&self, name: String) -> Option<Arc<InterfaceTypeDefinition>>;

    /// Return an directive definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_directive_definition_by_name)]
    fn find_directive_definition_by_name(&self, name: String) -> Option<Arc<DirectiveDefinition>>;

    /// Return any type definitions that contain the corresponding directive
    #[salsa::invoke(document::find_types_with_directive)]
    fn find_types_with_directive(&self, directive: String) -> Arc<Vec<TypeDefinition>>;

    /// Return an input object type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_input_object_by_name)]
    fn find_input_object_by_name(&self, name: String) -> Option<Arc<InputObjectTypeDefinition>>;

    /// Returns a map of type definitions in a GraphQL schema,
    /// Where the key is the type name and the value is a `TypeDefinition` representing the type.
    #[salsa::invoke(document::types_definitions_by_name)]
    fn types_definitions_by_name(&self) -> Arc<IndexMap<String, TypeDefinition>>;

    /// Return a type definition corresponding to the name.
    /// Result of this query is not cached internally.
    #[salsa::transparent]
    #[salsa::invoke(document::find_type_definition_by_name)]
    fn find_type_definition_by_name(&self, name: String) -> Option<TypeDefinition>;

    /// Return all query operations in a corresponding file.
    #[salsa::invoke(document::query_operations)]
    fn query_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all mutation operations in a corresponding file.
    #[salsa::invoke(document::mutation_operations)]
    fn mutation_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return all subscription operations in a corresponding file.
    #[salsa::invoke(document::subscription_operations)]
    fn subscription_operations(&self, file_id: FileId) -> Arc<Vec<Arc<OperationDefinition>>>;

    /// Return the fields in a selection set, not including fragments.
    #[salsa::invoke(document::operation_fields)]
    fn operation_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return all operation inline fragment fields in a corresponding selection set.
    #[salsa::invoke(document::operation_inline_fragment_fields)]
    fn operation_inline_fragment_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return all operation fragment spread fields in a corresponding selection set.
    #[salsa::invoke(document::operation_fragment_spread_fields)]
    fn operation_fragment_spread_fields(&self, selection_set: SelectionSet) -> Arc<Vec<Field>>;

    /// Return the fields that `selection_set` selects including visiting fragments and inline fragments.
    #[salsa::invoke(document::flattened_operation_fields)]
    fn flattened_operation_fields(&self, selection_set: SelectionSet) -> Vec<Arc<Field>>;

    /// Return all variables in a corresponding selection set.
    #[salsa::invoke(document::selection_variables)]
    fn selection_variables(&self, selection_set: SelectionSet) -> Arc<HashSet<Variable>>;

    /// Return all variables in corresponding variable definitions.
    #[salsa::invoke(document::operation_definition_variables)]
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

    #[salsa::invoke(document::subtype_map)]
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
    #[salsa::invoke(document::is_subtype)]
    fn is_subtype(&self, abstract_type: String, maybe_subtype: String) -> bool;
}

fn type_system_definitions(db: &dyn HirDatabase) -> Arc<TypeSystemDefinitions> {
    Arc::new(TypeSystemDefinitions {
        schema: db.schema(),
        scalars: db.scalars(),
        objects: db.object_types_with_built_ins(),
        interfaces: db.interfaces(),
        unions: db.unions(),
        enums: db.enums_with_built_ins(),
        input_objects: db.input_objects(),
        directives: db.directive_definitions(),
    })
}

fn type_system(db: &dyn HirDatabase) -> Arc<TypeSystem> {
    if let Some(precomputed_input) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed_input;
    }
    Arc::new(TypeSystem {
        definitions: db.type_system_definitions(),
        type_definitions_by_name: db.types_definitions_by_name(),
        subtype_map: db.subtype_map(),
        inputs: db
            .type_definition_files()
            .into_iter()
            .map(|file_id| (file_id, db.input(file_id)))
            .collect(),
    })
}

fn extensions(db: &dyn HirDatabase) -> Arc<Vec<TypeExtension>> {
    let mut extensions = vec![];
    for file_id in db.type_definition_files() {
        extensions.extend(
            db.ast(file_id)
                .document()
                .syntax()
                .children()
                .filter_map(ast::Definition::cast)
                .filter_map(|def| extension(db, def, file_id)),
        );
    }

    Arc::new(extensions)
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
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.schema.clone();
    }
    Arc::new(
        type_definitions(db, schema_definition)
            .next()
            .unwrap_or_else(|| implicit_schema_definition(db)),
    )
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
    ($db: ident, $convert: expr, $extension_type: ident) => {{
        let mut map = by_name!($db, $convert);
        for ext in $db.extensions().iter() {
            // Orphan or incorrect extensions are reported by validation.
            if let TypeExtension::$extension_type(ext) = ext {
                if let Some(def) = map.get_mut(ext.name()) {
                    Arc::get_mut(def).unwrap().push_extension(Arc::clone(ext))
                }
            }
        }
        map
    }};
}

fn object_types_with_built_ins(db: &dyn HirDatabase) -> ByName<ObjectTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.objects.clone();
    }
    Arc::new(by_name_extensible!(
        db,
        object_type_definition,
        ObjectTypeExtension
    ))
}

fn object_types(db: &dyn HirDatabase) -> ByName<ObjectTypeDefinition> {
    let mut objs = db.object_types_with_built_ins().as_ref().clone();

    objs.retain(|_k, v| !v.is_introspection());
    Arc::new(objs)
}

fn scalars(db: &dyn HirDatabase) -> ByName<ScalarTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.scalars.clone();
    }
    Arc::new(by_name_extensible!(
        db,
        scalar_definition,
        ScalarTypeExtension
    ))
}
fn enums_with_built_ins(db: &dyn HirDatabase) -> ByName<EnumTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.enums.clone();
    }
    Arc::new(by_name_extensible!(db, enum_definition, EnumTypeExtension))
}

fn enums(db: &dyn HirDatabase) -> ByName<EnumTypeDefinition> {
    let mut enums = db.enums_with_built_ins().as_ref().clone();

    enums.retain(|_k, v| !v.is_introspection());
    Arc::new(enums)
}

fn unions(db: &dyn HirDatabase) -> ByName<UnionTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.unions.clone();
    }
    Arc::new(by_name_extensible!(
        db,
        union_definition,
        UnionTypeExtension
    ))
}

fn interfaces(db: &dyn HirDatabase) -> ByName<InterfaceTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.interfaces.clone();
    }
    Arc::new(by_name_extensible!(
        db,
        interface_definition,
        InterfaceTypeExtension
    ))
}

fn input_objects(db: &dyn HirDatabase) -> ByName<InputObjectTypeDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.input_objects.clone();
    }
    Arc::new(by_name_extensible!(
        db,
        input_object_definition,
        InputObjectTypeExtension
    ))
}

fn directive_definitions(db: &dyn HirDatabase) -> ByName<DirectiveDefinition> {
    if let Some(precomputed) = db.type_system_hir_input() {
        // Panics in `ApolloCompiler` methods ensure `type_definition_files().is_empty()`
        return precomputed.definitions.directives.clone();
    }
    Arc::new(by_name!(db, directive_definition))
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
    let parent_object_ty = db.schema().self_root_operations().iter().find_map(|op| {
        if op.operation_ty() == ty {
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
    let description = description(schema_def.description());
    let directives = directives(schema_def.directives(), file_id);
    let mut operations =
        root_operation_type_definition(schema_def.root_operation_type_definitions(), file_id);
    let loc = location(file_id, schema_def.syntax());
    let extensions = schema_extensions(db);
    let mut root_operation_names = root_operations_names(&operations, &extensions);
    add_implicit_operations(db, &mut operations, &mut root_operation_names);

    Some(SchemaDefinition {
        description,
        directives,
        root_operation_type_definition: Arc::new(operations),
        loc: Some(loc),
        extensions,
        root_operation_names,
    })
}

fn implicit_schema_definition(db: &dyn HirDatabase) -> SchemaDefinition {
    let extensions = schema_extensions(db);
    let mut operations = Vec::new();
    let mut root_operation_names = root_operations_names(&operations, &extensions);
    add_implicit_operations(db, &mut operations, &mut root_operation_names);
    SchemaDefinition {
        description: None,
        directives: Arc::new(Vec::new()),
        root_operation_type_definition: Arc::new(operations),
        loc: None,
        extensions,
        root_operation_names,
    }
}

fn schema_extensions(db: &dyn HirDatabase) -> Vec<Arc<SchemaExtension>> {
    type_definitions(db, |_db, def: ast::SchemaExtension, file_id| {
        let directives = directives(def.directives(), file_id);
        let root_operation_type_definition = Arc::new(root_operation_type_definition(
            def.root_operation_type_definitions(),
            file_id,
        ));
        let loc = location(file_id, def.syntax());
        Some(Arc::new(SchemaExtension {
            directives,
            root_operation_type_definition,
            loc,
        }))
    })
    .collect()
}

fn root_operations_names(
    root_operation_type_definition: &[RootOperationTypeDefinition],
    extensions: &[Arc<SchemaExtension>],
) -> RootOperationNames {
    let mut names = RootOperationNames::default();
    let mut add_operations = |ops: &[RootOperationTypeDefinition]| {
        for op in ops {
            let name_field = match op.operation_ty() {
                OperationType::Query => &mut names.query,
                OperationType::Mutation => &mut names.mutation,
                OperationType::Subscription => &mut names.subscription,
            };
            if name_field.is_none() {
                *name_field = Some(op.named_type().name());
            }
        }
    };
    add_operations(root_operation_type_definition);
    for extension in extensions {
        add_operations(extension.root_operations());
    }
    names
}

/// https://spec.graphql.org/October2021/#sec-Root-Operation-Types.Default-Root-Operation-Type-Names
///
/// To distinguish between implicit and explicit definitions for validation purposes,
/// check `operation.loc.is_none()`.
///
/// NOTE(@lrlna):
/// "Query", "Subscription", "Mutation" object type definitions do not need
/// to be explicitly defined in a schema definition, but are implicitly
/// added.
fn add_implicit_operations(
    db: &dyn HirDatabase,
    operations: &mut Vec<RootOperationTypeDefinition>,
    names: &mut RootOperationNames,
) {
    for (name_field, operation_ty) in [
        (&mut names.query, OperationType::Query),
        (&mut names.mutation, OperationType::Mutation),
        (&mut names.subscription, OperationType::Subscription),
    ] {
        let name = operation_ty.into();
        if name_field.is_none() && db.object_types_with_built_ins().contains_key(name) {
            *name_field = Some(name.to_owned());
            operations.push(RootOperationTypeDefinition {
                operation_ty,
                named_type: Type::Named {
                    name: name.to_owned(),
                    loc: None,
                },
                loc: None,
            })
        }
    }
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
    let fields_by_name = ByNameWithExtensions::new(&fields_definition, FieldDefinition::name);
    let implements_interfaces_by_name =
        ByNameWithExtensions::new(&implements_interfaces, ImplementsInterface::interface);
    let is_introspection = INTROSPECTION_OBJECT_TYS.contains(&obj_def.name()?.text().as_str());
    let implicit_fields = Arc::new(vec![type_field(), typename_field(), schema_field()]);

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
        fields_by_name,
        implements_interfaces_by_name,
        is_introspection,
        implicit_fields,
    })
}

fn object_type_extension(
    _db: &dyn HirDatabase,
    def: ast::ObjectTypeExtension,
    file_id: FileId,
) -> Option<Arc<ObjectTypeExtension>> {
    Some(Arc::new(ObjectTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        implements_interfaces: implements_interfaces(def.implements_interfaces(), file_id),
        fields_definition: fields_definition(def.fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    }))
}

fn scalar_definition(
    db: &dyn HirDatabase,
    scalar_def: ast::ScalarTypeDefinition,
    file_id: FileId,
) -> Option<ScalarTypeDefinition> {
    let description = description(scalar_def.description());
    let name = name(scalar_def.name(), file_id)?;
    let directives = directives(scalar_def.directives(), file_id);
    let loc = location(file_id, scalar_def.syntax());
    let built_in = db.input(file_id).source_type().is_built_in();

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(ScalarTypeDefinition {
        description,
        name,
        directives,
        loc,
        built_in,
        extensions: Vec::new(),
    })
}

fn scalar_extension(
    _db: &dyn HirDatabase,
    def: ast::ScalarTypeExtension,
    file_id: FileId,
) -> Option<Arc<ScalarTypeExtension>> {
    Some(Arc::new(ScalarTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        loc: location(file_id, def.syntax()),
    }))
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
    let values_by_name =
        ByNameWithExtensions::new(&enum_values_definition, EnumValueDefinition::enum_value);
    let is_introspection = INTROSPECTION_ENUM_TYS.contains(&enum_def.name()?.text().as_str());

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(EnumTypeDefinition {
        description,
        name,
        directives,
        enum_values_definition,
        loc,
        extensions: Vec::new(),
        values_by_name,
        is_introspection,
    })
}

fn enum_extension(
    _db: &dyn HirDatabase,
    def: ast::EnumTypeExtension,
    file_id: FileId,
) -> Option<Arc<EnumTypeExtension>> {
    Some(Arc::new(EnumTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        enum_values_definition: enum_values_definition(def.enum_values_definition(), file_id),
        loc: location(file_id, def.syntax()),
    }))
}

fn enum_values_definition(
    enum_values_def: Option<ast::EnumValuesDefinition>,
    file_id: FileId,
) -> Arc<Vec<EnumValueDefinition>> {
    match enum_values_def {
        Some(enum_values) => {
            let enum_values = enum_values
                .enum_value_definitions()
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
    let members_by_name = ByNameWithExtensions::new(&union_members, UnionMember::name);
    let implicit_fields = Arc::new(vec![typename_field()]);

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(UnionTypeDefinition {
        description,
        name,
        directives,
        union_members,
        loc,
        extensions: Vec::new(),
        members_by_name,
        implicit_fields,
    })
}

fn union_extension(
    _db: &dyn HirDatabase,
    def: ast::UnionTypeExtension,
    file_id: FileId,
) -> Option<Arc<UnionTypeExtension>> {
    let directives = directives(def.directives(), file_id);
    let name = name(def.name(), file_id)?;
    let union_members = union_members(def.union_member_types(), file_id);
    let loc = location(file_id, def.syntax());
    let members_by_name = ByNameWithExtensions::new(&union_members, UnionMember::name);
    Some(Arc::new(UnionTypeExtension {
        directives,
        name,
        union_members,
        loc,
        members_by_name,
    }))
}

fn union_members(
    union_members: Option<ast::UnionMemberTypes>,
    file_id: FileId,
) -> Arc<Vec<UnionMember>> {
    match union_members {
        Some(members) => {
            let mems = members
                .named_types()
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
    let fields_by_name = ByNameWithExtensions::new(&fields_definition, FieldDefinition::name);
    let implements_interfaces_by_name =
        ByNameWithExtensions::new(&implements_interfaces, ImplementsInterface::interface);
    let implicit_fields = Arc::new(vec![typename_field()]);

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
        fields_by_name,
        implements_interfaces_by_name,
        implicit_fields,
    })
}

fn interface_extension(
    _db: &dyn HirDatabase,
    def: ast::InterfaceTypeExtension,
    file_id: FileId,
) -> Option<Arc<InterfaceTypeExtension>> {
    Some(Arc::new(InterfaceTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        implements_interfaces: implements_interfaces(def.implements_interfaces(), file_id),
        fields_definition: fields_definition(def.fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    }))
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
        loc,
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
    let input_fields_by_name =
        ByNameWithExtensions::new(&input_fields_definition, InputValueDefinition::name);

    // TODO(@goto-bus-stop) when a name is missing on this,
    // we might still want to produce a HIR node, so we can validate other parts of the definition
    Some(InputObjectTypeDefinition {
        description,
        name,
        directives,
        input_fields_definition,
        loc,
        extensions: Vec::new(),
        input_fields_by_name,
    })
}

fn input_object_extension(
    _db: &dyn HirDatabase,
    def: ast::InputObjectTypeExtension,
    file_id: FileId,
) -> Option<Arc<InputObjectTypeExtension>> {
    Some(Arc::new(InputObjectTypeExtension {
        directives: directives(def.directives(), file_id),
        name: name(def.name(), file_id)?,
        input_fields_definition: input_fields_definition(def.input_fields_definition(), file_id),
        loc: location(file_id, def.syntax()),
    }))
}

fn extension(db: &dyn HirDatabase, def: ast::Definition, file_id: FileId) -> Option<TypeExtension> {
    match def {
        ast::Definition::ScalarTypeExtension(def) => {
            scalar_extension(db, def, file_id).map(TypeExtension::ScalarTypeExtension)
        }
        ast::Definition::ObjectTypeExtension(def) => {
            object_type_extension(db, def, file_id).map(TypeExtension::ObjectTypeExtension)
        }
        ast::Definition::InterfaceTypeExtension(def) => {
            interface_extension(db, def, file_id).map(TypeExtension::InterfaceTypeExtension)
        }
        ast::Definition::UnionTypeExtension(def) => {
            union_extension(db, def, file_id).map(TypeExtension::UnionTypeExtension)
        }
        ast::Definition::EnumTypeExtension(def) => {
            enum_extension(db, def, file_id).map(TypeExtension::EnumTypeExtension)
        }
        ast::Definition::InputObjectTypeExtension(def) => {
            input_object_extension(db, def, file_id).map(TypeExtension::InputObjectTypeExtension)
        }
        _ => None,
    }
}

fn type_field() -> FieldDefinition {
    FieldDefinition {
        description: None,
        name: Name {
            src: "__type".into(),
            loc: None,
        },
        arguments: ArgumentsDefinition {
            input_values: Arc::new(vec![InputValueDefinition {
                description: None,
                name: Name {
                    src: "name".into(),
                    loc: None,
                },
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
        ty: Type::Named {
            name: "__Type".into(),
            loc: None,
        },
        directives: Arc::new(Vec::new()),
        loc: None,
    }
}

fn schema_field() -> FieldDefinition {
    FieldDefinition {
        description: None,
        name: Name {
            src: "__schema".into(),
            loc: None,
        },
        arguments: ArgumentsDefinition {
            input_values: Arc::new(Vec::new()),
            loc: None,
        },
        ty: Type::NonNull {
            ty: Box::new(Type::Named {
                name: "__Schema".into(),
                loc: None,
            }),
            loc: None,
        },
        directives: Arc::new(Vec::new()),
        loc: None,
    }
}

fn typename_field() -> FieldDefinition {
    FieldDefinition {
        description: None,
        name: Name {
            src: "__typename".into(),
            loc: None,
        },
        arguments: ArgumentsDefinition {
            input_values: Arc::new(Vec::new()),
            loc: None,
        },
        ty: Type::NonNull {
            ty: Box::new(Type::Named {
                name: "String".into(),
                loc: None,
            }),
            loc: None,
        },
        directives: Arc::new(Vec::new()),
        loc: None,
    }
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
        loc: Some(loc),
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
) -> Vec<RootOperationTypeDefinition> {
    root_type_def
        .into_iter()
        .filter_map(|ty| {
            if let Some(named_ty) = ty.named_type() {
                let operation_type = operation_type(ty.operation_type());
                let named_type = named_type(named_ty.name()?, file_id);
                let loc = location(file_id, ty.syntax());

                Some(RootOperationTypeDefinition {
                    operation_ty: operation_type,
                    named_type,
                    loc: Some(loc),
                })
            } else {
                None
            }
        })
        .collect()
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
            fragment_spread(db, fragment, parent_obj_ty, file_id).map(Selection::FragmentSpread)
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
    let new_parent_obj = type_condition
        .clone()
        .map_or_else(|| parent_obj.clone(), |tc| Some(tc.src().to_string()));
    let selection_set: SelectionSet =
        selection_set(db, fragment.selection_set(), new_parent_obj, file_id);
    let loc = location(file_id, fragment.syntax());

    let fragment_data = InlineFragment {
        // for implicit inline fragments, the type condition is implied to be
        // that of the current scope
        type_condition: type_condition.or(parent_obj.clone().map(|o| Name { src: o, loc: None })),
        directives,
        selection_set,
        parent_obj,
        loc,
    };
    Arc::new(fragment_data)
}

fn fragment_spread(
    _db: &dyn HirDatabase,
    fragment: ast::FragmentSpread,
    parent_obj: Option<String>,
    file_id: FileId,
) -> Option<Arc<FragmentSpread>> {
    let name = name(fragment.fragment_name()?.name(), file_id)?;
    let directives = directives(fragment.directives(), file_id);
    let loc = location(file_id, fragment.syntax());

    let fragment_data = FragmentSpread {
        name,
        directives,
        parent_obj,
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
            .field(db, field_name)?
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
