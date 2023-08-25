use std::collections::{hash_map::Entry, HashMap};
use std::sync::Arc;

use crate::{
    database::db::Upcast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::*,
    validation::{
        argument, directive, enum_, extension, fragment, input_object, interface, object,
        operation, scalar, schema, selection, union_, value, variable,
    },
    CstDatabase, FileId, HirDatabase, InputDatabase,
};
use apollo_parser::cst;
use apollo_parser::cst::CstNode;

use super::field;

const BUILT_IN_SCALARS: [&str; 5] = ["Int", "Float", "Boolean", "String", "ID"];

#[salsa::query_group(ValidationStorage)]
pub trait ValidationDatabase:
    Upcast<dyn HirDatabase> + InputDatabase + CstDatabase + HirDatabase
{
    /// Validate all documents.
    fn validate(&self) -> Vec<ApolloDiagnostic>;

    /// Validate the type system, combined of all type system documents known to
    /// the compiler.
    fn validate_type_system(&self) -> Vec<ApolloDiagnostic>;

    /// Validate an executable document.
    fn validate_executable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

    fn validate_name_uniqueness(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(schema::validate_schema_definition)]
    fn validate_schema_definition(&self, def: Arc<SchemaDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(schema::validate_root_operation_definitions)]
    fn validate_root_operation_definitions(
        &self,
        defs: Vec<RootOperationTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(scalar::validate_scalar_definitions)]
    fn validate_scalar_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(scalar::validate_scalar_definition)]
    fn validate_scalar_definition(
        &self,
        scalar_def: Arc<ScalarTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_definitions)]
    fn validate_enum_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_definition)]
    fn validate_enum_definition(&self, def: Arc<EnumTypeDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_value)]
    fn validate_enum_value(&self, def: EnumValueDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(union_::validate_union_definitions)]
    fn validate_union_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(union_::validate_union_definition)]
    fn validate_union_definition(&self, def: Arc<UnionTypeDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_interface_definitions)]
    fn validate_interface_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_interface_definition)]
    fn validate_interface_definition(
        &self,
        def: Arc<InterfaceTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_implements_interfaces)]
    fn validate_implements_interfaces(
        &self,
        implementor_name: String,
        impl_interfaces: Vec<ImplementsInterface>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(directive::validate_directive_definitions)]
    fn validate_directive_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(directive::validate_directives)]
    fn validate_directives(
        &self,
        dirs: Vec<Directive>,
        loc: DirectiveLocation,
        var_defs: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_object_definitions)]
    fn validate_input_object_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_object_definition)]
    fn validate_input_object_definition(
        &self,
        def: Arc<InputObjectTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_values)]
    fn validate_input_values(
        &self,
        vals: Arc<Vec<InputValueDefinition>>,
        dir_loc: DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(object::validate_object_type_definitions)]
    fn validate_object_type_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(extension::validate_extensions)]
    fn validate_extensions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_subscription_operations)]
    fn validate_subscription_operations(
        &self,
        defs: Arc<Vec<Arc<OperationDefinition>>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_query_operations)]
    fn validate_query_operations(
        &self,
        defs: Arc<Vec<Arc<OperationDefinition>>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_mutation_operations)]
    fn validate_mutation_operations(
        &self,
        mutations: Arc<Vec<Arc<OperationDefinition>>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(object::validate_object_type_definition)]
    fn validate_object_type_definition(
        &self,
        def: Arc<ObjectTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field_definitions)]
    fn validate_field_definitions(&self, fields: Vec<FieldDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field_definition)]
    fn validate_field_definition(&self, field: FieldDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field)]
    fn validate_field(
        &self,
        field: Arc<Field>,
        vars: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_leaf_field_selection)]
    fn validate_leaf_field_selection(
        &self,
        field: Arc<Field>,
        field_ty: Type,
    ) -> Result<(), ApolloDiagnostic>;

    #[salsa::invoke(argument::validate_arguments_definition)]
    fn validate_arguments_definition(
        &self,
        def: ArgumentsDefinition,
        loc: DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(argument::validate_arguments)]
    fn validate_arguments(&self, arg: Vec<Argument>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_operation_definitions)]
    fn validate_operation_definitions(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

    /// Given a type definition, find all the types that can be used for fragment spreading.
    ///
    /// Spec: https://spec.graphql.org/October2021/#GetPossibleTypes()
    #[salsa::invoke(fragment::get_possible_types)]
    fn get_possible_types(&self, ty: TypeDefinition) -> Vec<TypeDefinition>;

    #[salsa::invoke(fragment::validate_fragment_selection)]
    fn validate_fragment_selection(&self, spread: FragmentSelection) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_fragment_spread)]
    fn validate_fragment_spread(
        &self,
        spread: Arc<FragmentSpread>,
        var_defs: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_inline_fragment)]
    fn validate_inline_fragment(
        &self,
        inline: Arc<InlineFragment>,
        var_defs: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_fragment_definition)]
    #[salsa::transparent]
    fn validate_fragment_definition(
        &self,
        def: Arc<FragmentDefinition>,
        var_defs: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_fragment_cycles)]
    fn validate_fragment_cycles(&self, def: Arc<FragmentDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_fragment_type_condition)]
    fn validate_fragment_type_condition(
        &self,
        type_cond: String,
        loc: HirNodeLocation,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(fragment::validate_fragment_used)]
    fn validate_fragment_used(
        &self,
        def: Arc<FragmentDefinition>,
        file_id: FileId,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(selection::validate_selection_set)]
    fn validate_selection_set(
        &self,
        sel_set: SelectionSet,
        vars: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(selection::validate_selection)]
    fn validate_selection(
        &self,
        sel: Arc<Vec<Selection>>,
        vars: Arc<Vec<VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(variable::validate_variable_definitions)]
    fn validate_variable_definitions(&self, defs: Vec<VariableDefinition>)
        -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(variable::validate_unused_variables)]
    fn validate_unused_variable(&self, op: Arc<OperationDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::transparent]
    #[salsa::invoke(variable::validate_variable_usage)]
    fn validate_variable_usage(
        &self,
        var_usage: InputValueDefinition,
        var_defs: Arc<Vec<VariableDefinition>>,
        arg: Argument,
    ) -> Result<(), ApolloDiagnostic>;

    #[salsa::transparent]
    #[salsa::invoke(value::validate_values)]
    fn validate_values(
        &self,
        ty: &Type,
        arg: &Argument,
        var_defs: Arc<Vec<VariableDefinition>>,
    ) -> Result<(), Vec<ApolloDiagnostic>>;

    /// Check if two fields will output the same type.
    ///
    /// If the two fields output different types, returns an `Err` containing diagnostic information.
    /// To simply check if outputs are the same, you can use `.is_ok()`:
    /// ```rust,ignore
    /// let is_same = db.same_response_shape(field_a, field_b).is_ok();
    /// // `is_same` is `bool`
    /// ```
    ///
    /// Spec: https://spec.graphql.org/October2021/#SameResponseShape()
    #[salsa::invoke(selection::same_response_shape)]
    fn same_response_shape(
        &self,
        field_a: Arc<Field>,
        field_b: Arc<Field>,
    ) -> Result<(), ApolloDiagnostic>;

    /// Check if the fields in a given selection set can be merged.
    ///
    /// If the fields cannot be merged, returns an `Err` containing diagnostic information.
    ///
    /// Spec: https://spec.graphql.org/October2021/#FieldsInSetCanMerge()
    #[salsa::invoke(selection::fields_in_set_can_merge)]
    fn fields_in_set_can_merge(
        &self,
        selection_set: SelectionSet,
    ) -> Result<(), Vec<ApolloDiagnostic>>;
}

pub fn validate(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(db.syntax_errors());

    diagnostics.extend(db.validate_type_system());

    for file_id in db.executable_definition_files() {
        diagnostics.extend(db.validate_executable(file_id));
    }

    diagnostics
}

fn validate_name_uniqueness(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Different node types use different namespaces.
    let mut fragment_scope = HashMap::<String, (FileId, cst::Name)>::new();
    let mut operation_scope = HashMap::new();
    let mut directive_scope = HashMap::new();
    let mut type_scope = HashMap::new();

    let all_types = db
        .type_definition_files()
        .into_iter()
        .flat_map(move |file_id| {
            db.cst(file_id)
                .document()
                .syntax()
                .children()
                .filter_map(cst::Definition::cast)
                // Extension names are allowed to be duplicates,
                // and schema definitions don't have names.
                .filter(|def| {
                    !def.is_extension_definition()
                        && !matches!(def, cst::Definition::SchemaDefinition(_))
                })
                .map(move |def| (file_id, def))
        });

    for (file_id, cst_def) in all_types {
        let ty_ = match cst_def {
            cst::Definition::OperationDefinition(_) => "operation",
            cst::Definition::FragmentDefinition(_) => "fragment",
            cst::Definition::DirectiveDefinition(_) => "directive",
            cst::Definition::ScalarTypeDefinition(_)
            | cst::Definition::ObjectTypeDefinition(_)
            | cst::Definition::InterfaceTypeDefinition(_)
            | cst::Definition::UnionTypeDefinition(_)
            | cst::Definition::EnumTypeDefinition(_)
            | cst::Definition::InputObjectTypeDefinition(_) => "type",
            cst::Definition::SchemaDefinition(_)
            | cst::Definition::SchemaExtension(_)
            | cst::Definition::ScalarTypeExtension(_)
            | cst::Definition::ObjectTypeExtension(_)
            | cst::Definition::InterfaceTypeExtension(_)
            | cst::Definition::UnionTypeExtension(_)
            | cst::Definition::EnumTypeExtension(_)
            | cst::Definition::InputObjectTypeExtension(_) => unreachable!(),
        };
        let scope = match cst_def {
            cst::Definition::OperationDefinition(_) => &mut operation_scope,
            cst::Definition::FragmentDefinition(_) => &mut fragment_scope,
            cst::Definition::DirectiveDefinition(_) => &mut directive_scope,
            _ => &mut type_scope,
        };

        if let Some(name_node) = cst_def.name() {
            let name = &*name_node.text();
            match scope.entry(name.to_string()) {
                Entry::Occupied(entry) => {
                    let (original_file_id, original) = entry.get();
                    let original_definition = (*original_file_id, original.syntax().text_range());
                    let redefined_definition = (file_id, name_node.syntax().text_range());
                    let is_built_in = db.input(file_id).source_type().is_built_in();
                    let is_scalar = matches!(cst_def, cst::Definition::ScalarTypeDefinition(_));

                    if is_scalar && BUILT_IN_SCALARS.contains(&name) && !is_built_in {
                        diagnostics.push(
                            ApolloDiagnostic::new(
                                db,
                                redefined_definition.into(),
                                DiagnosticData::BuiltInScalarDefinition,
                            )
                            .label(Label::new(
                                redefined_definition,
                                "remove this scalar definition",
                            )),
                        );
                    } else {
                        diagnostics.push(
                            ApolloDiagnostic::new(
                                db,
                                redefined_definition.into(),
                                DiagnosticData::UniqueDefinition {
                                    ty: ty_,
                                    name: name.to_string(),
                                    original_definition: original_definition.into(),
                                    redefined_definition: redefined_definition.into(),
                                },
                            )
                            .labels([
                                Label::new(
                                    original_definition,
                                    format!("previous definition of `{name}` here"),
                                ),
                                Label::new(
                                    redefined_definition,
                                    format!("`{name}` redefined here"),
                                ),
                            ])
                            .help(format!(
                                "`{name}` must only be defined once in this document."
                            )),
                        );
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert((file_id, name_node));
                }
            }
        }
    }

    diagnostics
}

pub fn validate_type_system(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_name_uniqueness());

    diagnostics.extend(db.validate_schema_definition(db.type_system_definitions().schema.clone()));

    diagnostics.extend(db.validate_scalar_definitions());
    diagnostics.extend(db.validate_enum_definitions());
    diagnostics.extend(db.validate_union_definitions());

    diagnostics.extend(db.validate_interface_definitions());
    diagnostics.extend(db.validate_directive_definitions());
    diagnostics.extend(db.validate_input_object_definitions());
    diagnostics.extend(db.validate_object_type_definitions());

    diagnostics.extend(db.validate_extensions());

    diagnostics
}

pub fn validate_executable(db: &dyn ValidationDatabase, file_id: FileId) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    if db.source_type(file_id).is_executable() {
        let document = db.cst(file_id).document();
        for def in document.definitions() {
            if !def.is_executable_definition() {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        (file_id, def.syntax().text_range()).into(),
                        DiagnosticData::ExecutableDefinition { kind: def.kind() },
                    )
                    .label(Label::new(
                        (file_id, def.syntax().text_range()),
                        "not supported in executable documents",
                    )),
                );
            }
        }
    }

    diagnostics.extend(db.validate_operation_definitions(file_id));
    for def in db.fragments(file_id).values() {
        diagnostics.extend(db.validate_fragment_used(Arc::clone(def), file_id));
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::ValidationDatabase;
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn executable_and_type_system_definitions() {
        let input_type_system = r#"
type Query {
    name: String
}
"#;
        let input_executable = r#"
fragment q on Query { name }
query {
    ...q
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input_type_system, "schema.graphql");
        compiler.add_executable(input_executable, "query.graphql");

        let diagnostics = compiler.validate();
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn executable_definition_does_not_contain_type_system_definitions() {
        let input_type_system = r#"
type Query {
    name: String
}
"#;
        let input_executable = r#"
type Object {
    notAllowed: Boolean!
}
fragment q on Query { name }
query {
    ...q
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input_type_system, "schema.graphql");
        compiler.add_executable(input_executable, "query.graphql");

        let diagnostics = compiler.validate();
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].data.to_string(),
            "executable documents must not contain ObjectTypeDefinition"
        );
    }

    #[test]
    fn executable_definition_with_cycles_do_not_overflow_stack() {
        let input_type_system = r#"
type Query {
    name: String
}
"#;

        let input_executable = r#"
{
    ...q
}
fragment q on Query {
    ...q
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input_type_system, "schema.graphql");
        compiler.add_executable(input_executable, "query.graphql");

        let diagnostics = compiler.validate();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].data.to_string(),
            "`q` fragment cannot reference itself"
        );
    }

    #[test]
    fn executable_definition_with_nested_cycles_do_not_overflow_stack() {
        let input_type_system = r#"
type Query {
    obj: TestObject
}

type TestObject {
    name: String
}
"#;

        let input_executable = r#"
{
    obj {
        ...q
    }
}

fragment q on TestObject {
    ...q
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(input_type_system, "schema.graphql");
        compiler.add_executable(input_executable, "query.graphql");

        let diagnostics = compiler.validate();

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].data.to_string(),
            "`q` fragment cannot reference itself"
        );
    }

    #[test]
    fn validation_with_type_system_hir() {
        let input_type_system = r#"
type Query {
    obj: TestObject
}

type TestObject {
    name: String
}
"#;

        let input_executable = r#"
{
    obj {
        name
        nickname
    }
}
"#;

        let mut root_compiler = ApolloCompiler::new();
        root_compiler.add_type_system(input_type_system, "schema.graphql");
        assert!(root_compiler.validate().is_empty());

        let mut child_compiler = ApolloCompiler::new();
        child_compiler.set_type_system_hir(root_compiler.db.type_system());
        let executable_id = child_compiler.add_executable(input_executable, "query.graphql");
        let diagnostics = child_compiler.db.validate_executable(executable_id);

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(
            diagnostics[0].data.to_string(),
            "cannot query field `nickname` on type `TestObject`"
        );
    }
}
