use crate::{
    database::db::Upcast,
    hir::*,
    validation::{
        argument, directive, enum_, fragment, input_object, interface, object, operation, scalar,
        schema, selection, union_, variable,
    },
    ApolloDiagnostic, AstDatabase, FileId, HirDatabase, InputDatabase,
};

use super::field;

#[salsa::query_group(ValidationStorage)]
pub trait ValidationDatabase:
    Upcast<dyn HirDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    /// Validate all documents.
    fn validate(&self) -> Vec<ApolloDiagnostic>;

    /// Validate the type system, combined of all type system documents known to
    /// the compiler.
    fn validate_type_system(&self) -> Vec<ApolloDiagnostic>;

    /// Validate the corresonding executable document.
    fn validate_executable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(schema::validate_schema_definition)]
    fn validate_schema_definition(&self, def: SchemaDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(schema::validate_root_operation_definitions)]
    fn validate_root_operation_definitions(
        &self,
        defs: Vec<RootOperationTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(scalar::validate_scalar_definitions)]
    fn validate_scalar_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(scalar::validate_scalar_definition)]
    fn validate_scalar_definition(&self, scalar_def: ScalarTypeDefinition)
        -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_definitions)]
    fn validate_enum_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_definition)]
    fn validate_enum_definition(&self, def: EnumTypeDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(enum_::validate_enum_value)]
    fn validate_enum_value(&self, def: EnumValueDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(union_::validate_union_definitions)]
    fn validate_union_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(union_::validate_union_definition)]
    fn validate_union_definition(&self, def: UnionTypeDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_interface_definitions)]
    fn validate_interface_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_interface_definition)]
    fn validate_interface_definition(&self, def: InterfaceTypeDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(interface::validate_implements_interfaces)]
    fn validate_implements_interfaces(
        &self,
        impl_interfaces: Vec<ImplementsInterface>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(directive::validate_directive_definitions)]
    fn validate_directive_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(directive::validate_directives)]
    fn validate_directives(
        &self,
        dirs: Vec<Directive>,
        loc: DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_object_definitions)]
    fn validate_input_object_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_object_definition)]
    fn validate_input_object_definition(
        &self,
        def: InputObjectTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(input_object::validate_input_values)]
    fn validate_input_values(
        &self,
        vals: Vec<InputValueDefinition>,
        dir_loc: DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(object::validate_object_type_definitions)]
    fn validate_object_type_definitions(&self) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_subscription_operations)]
    fn validate_subscription_operations(
        &self,
        defs: Vec<OperationDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_query_operations)]
    fn validate_query_operations(&self, defs: Vec<OperationDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(operation::validate_mutation_operations)]
    fn validate_mutation_operations(
        &self,
        mutations: Vec<OperationDefinition>,
    ) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(object::validate_object_type_definition)]
    fn validate_object_type_definition(&self, def: ObjectTypeDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field_definitions)]
    fn validate_field_definitions(&self, fields: Vec<FieldDefinition>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field_definition)]
    fn validate_field_definition(&self, field: FieldDefinition) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(field::validate_field)]
    fn validate_field(&self, field: Field) -> Vec<ApolloDiagnostic>;

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

    #[salsa::invoke(fragment::validate_fragment_definitions)]
    fn validate_fragment_definitions(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(selection::validate_selection_set)]
    fn validate_selection_set(&self, sel_set: SelectionSet) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(selection::validate_selection)]
    fn validate_selection(&self, sel: Vec<Selection>) -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(variable::validate_variable_definitions)]
    fn validate_variable_definitions(&self, defs: Vec<VariableDefinition>)
        -> Vec<ApolloDiagnostic>;

    #[salsa::invoke(variable::validate_unused_variables)]
    fn validate_unused_variable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
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

pub fn validate_type_system(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(
        db.validate_schema_definition(db.type_system_definitions().schema.as_ref().clone()),
    );

    diagnostics.extend(db.validate_scalar_definitions());
    diagnostics.extend(db.validate_enum_definitions());
    diagnostics.extend(db.validate_union_definitions());

    diagnostics.extend(db.validate_interface_definitions());
    diagnostics.extend(db.validate_directive_definitions());
    diagnostics.extend(db.validate_input_object_definitions());
    diagnostics.extend(db.validate_object_type_definitions());

    diagnostics
}

pub fn validate_executable(db: &dyn ValidationDatabase, file_id: FileId) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_operation_definitions(file_id));
    diagnostics.extend(db.validate_fragment_definitions(file_id));
    diagnostics.extend(db.validate_unused_variable(file_id));

    diagnostics
}
