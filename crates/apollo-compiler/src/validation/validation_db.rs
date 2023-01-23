use std::sync::Arc;

use crate::{
    database::db::Upcast,
    hir,
    validation::{
        argument, directive, enum_, input_object, interface, object, operation, scalar, schema,
        selection_set, subscription, union_, variable,
    },
    ApolloDiagnostic, AstDatabase, FileId, HirDatabase, InputDatabase,
};

#[salsa::query_group(ValidationStorage)]
pub trait ValidationDatabase:
    Upcast<dyn HirDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    /// Validate all documents.
    fn validate(&self) -> Vec<ApolloDiagnostic>;

    /// Validate the type system, combined of all type system documents known to the compiler.
    fn validate_type_system(&self) -> Vec<ApolloDiagnostic>;
    fn validate_schema_definition(&self) -> Vec<ApolloDiagnostic>;
    fn validate_scalar_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_enum_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_union_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_interface_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_directive_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_input_object_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn validate_object_type_definitions(&self) -> Vec<ApolloDiagnostic>;

    /// Validate an executable document.
    fn validate_executable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
    fn validate_operation_definitions(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
    fn validate_subscription_operations(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
    fn validate_unused_variable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
    fn validate_arguments_definition(
        &self,
        args_def: hir::ArgumentsDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn validate_arguments(&self, arg: Vec<hir::Argument>) -> Vec<ApolloDiagnostic>;
    fn check_variable_definitions(
        &self,
        variables: Arc<Vec<hir::VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;
}

pub fn validate_arguments_definition(
    db: &dyn ValidationDatabase,
    args_def: hir::ArgumentsDefinition,
) -> Vec<ApolloDiagnostic> {
    argument::validate(db, args_def, hir::DirectiveLocation::ArgumentDefinition)
}

pub fn check_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for variable in variables.iter() {
        diagnostics.extend(directive::validate_usage(
            db,
            variable.directives().to_vec(),
            hir::DirectiveLocation::VariableDefinition,
        ));
    }

    diagnostics
}

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    args: Vec<hir::Argument>,
) -> Vec<ApolloDiagnostic> {
    argument::validate_usage(db, args)
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

    let type_system = db.type_system_definitions();
    let hir::TypeSystemDefinitions {
        schema,
        scalars,
        objects,
        interfaces,
        unions,
        enums,
        input_objects,
        directives,
    } = &*type_system;

    for def in db.all_operations().iter() {}
    for def in db.all_fragments().values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::FragmentDefinition,
        ));
        diagnostics.extend(selection_set::validate(db, def.selection_set().clone()));
    }

    diagnostics.extend(db.validate_schema_definition());

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
    diagnostics.extend(db.validate_unused_variable(file_id));
    diagnostics.extend(db.validate_subscription_operations(file_id));

    diagnostics
}

pub fn validate_schema_definition(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    schema::validate(db, db.type_system_definitions().schema.clone())
}

pub fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    scalar::validate(db)
}

pub fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().enums;
    for def in defs.values() {
        diagnostics.extend(enum_::validate(db, def.clone()));
    }

    diagnostics
}

pub fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().unions;
    for def in defs.values() {
        diagnostics.extend(union_::validate(db, def.clone()));
    }

    diagnostics
}

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().interfaces;
    for def in defs.values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::Interface,
        ));
        interface::validate(db, def.clone());
    }

    diagnostics
}

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate(db));

    let defs = &db.type_system_definitions().directives;
    for def in defs.values() {
        diagnostics.extend(db.validate_arguments_definition(def.arguments.clone()));
    }

    diagnostics
}

pub fn validate_input_object_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().input_objects;
    for def in defs.values() {
        input_object::validate(db, def.clone());
    }

    diagnostics
}

pub fn validate_object_type_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().objects;
    for def in defs.values() {
        diagnostics.extend(object::validate(db, def.clone()))
    }

    diagnostics
}

pub fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in db.all_operations().iter() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            def.operation_ty().into(),
        ));
        diagnostics.extend(variable::validate(db, def.variables.clone()));
        diagnostics.extend(selection_set::validate(db, def.selection_set().clone()));
    }
    operation::check(db, file_id)
}

pub fn validate_unused_variable(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    variable::check(db, file_id)
}
