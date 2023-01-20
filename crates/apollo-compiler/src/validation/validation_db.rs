use std::sync::Arc;

use crate::{
    database::db::Upcast,
    hir,
    validation::{
        arguments, directive, enum_, input_object, interface, object, operation, scalar, schema,
        subscription, union_, unused_variable,
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

    fn check_directive_definition(
        &self,
        def: Arc<hir::DirectiveDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_object_type_definition(
        &self,
        def: Arc<hir::ObjectTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_interface_type_definition(
        &self,
        def: Arc<hir::InterfaceTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_union_type_definition(
        &self,
        def: Arc<hir::UnionTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_selection_set(&self, selection_set: hir::SelectionSet) -> Vec<ApolloDiagnostic>;
    fn check_selection(&self, selection: Vec<hir::Selection>) -> Vec<ApolloDiagnostic>;
    fn check_field_definition(&self, field: hir::FieldDefinition) -> Vec<ApolloDiagnostic>;
    fn check_field(&self, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic>;
    fn check_variable_definitions(
        &self,
        variables: Arc<Vec<hir::VariableDefinition>>,
    ) -> Vec<ApolloDiagnostic>;
}

pub fn check_directive_definition(
    db: &dyn ValidationDatabase,
    directive: Arc<hir::DirectiveDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_arguments_definition(directive.arguments.clone()));

    diagnostics
}

pub fn check_object_type_definition(
    db: &dyn ValidationDatabase,
    object_type: Arc<hir::ObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // TODO: validate extensions
    for field in object_type.fields_definition() {
        diagnostics.extend(db.check_field_definition(field.clone()));
    }

    diagnostics
}

pub fn check_interface_type_definition(
    db: &dyn ValidationDatabase,
    interface_type: Arc<hir::InterfaceTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // TODO: validate extensions
    for field in interface_type.fields_definition() {
        diagnostics.extend(db.check_field_definition(field.clone()));
    }

    diagnostics
}

pub fn check_union_type_definition(
    _db: &dyn ValidationDatabase,
    _union_type: Arc<hir::UnionTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    // TODO: validate extensions
    vec![]
}

pub fn check_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    db.check_selection((*selection_set.selection).clone())
}

pub fn check_selection(
    db: &dyn ValidationDatabase,
    selection: Vec<hir::Selection>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection {
        match sel {
            hir::Selection::Field(field) => {
                if !field.selection_set().selection().is_empty() {
                    diagnostics
                        .extend(db.check_selection((*field.selection_set().selection).clone()))
                }
                diagnostics.extend(db.check_field(field));
            }
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(directive::validate_usage(
                db,
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => {
                diagnostics.extend(directive::validate_usage(
                    db,
                    inline.directives().to_vec(),
                    hir::DirectiveLocation::InlineFragment,
                ))
            }
        }
    }

    diagnostics
}

pub fn validate_arguments_definition(
    db: &dyn ValidationDatabase,
    args_def: hir::ArgumentsDefinition,
) -> Vec<ApolloDiagnostic> {
    arguments::validate(db, args_def, hir::DirectiveLocation::ArgumentDefinition)
}

pub fn check_field_definition(
    db: &dyn ValidationDatabase,
    field: hir::FieldDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate_usage(
        db,
        field.directives().to_vec(),
        hir::DirectiveLocation::FieldDefinition,
    ));

    diagnostics.extend(db.validate_arguments_definition(field.arguments));

    diagnostics
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

pub fn check_field(db: &dyn ValidationDatabase, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(directive::validate_usage(
        db,
        field.directives().to_vec(),
        hir::DirectiveLocation::Field,
    ));
    diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));

    diagnostics
}

pub fn validate_arguments(
    db: &dyn ValidationDatabase,
    args: Vec<hir::Argument>,
) -> Vec<ApolloDiagnostic> {
    arguments::validate_usage(db, args)
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

    for def in db.all_operations().iter() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            def.operation_ty().into(),
        ));
        diagnostics.extend(db.check_variable_definitions(def.variables.clone()));
        diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
    }
    for def in db.all_fragments().values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::FragmentDefinition,
        ));
        diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
    }
    for def in directives.values() {
        diagnostics.extend(db.check_directive_definition(def.clone()));
    }
    for def in objects.values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::Object,
        ));
        diagnostics.extend(db.check_object_type_definition(def.clone()));
    }
    for def in interfaces.values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::Interface,
        ));
        diagnostics.extend(db.check_interface_type_definition(def.clone()));
        // TODO: validate extensions
    }
    for def in unions.values() {
        diagnostics.extend(directive::validate_usage(
            db,
            def.directives().to_vec(),
            hir::DirectiveLocation::Union,
        ));
        diagnostics.extend(db.check_union_type_definition(def.clone()));
        // TODO: validate extensions
    }
    for def in input_objects.values() {
        // TODO: validate extensions
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

    let enums = &db.type_system_definitions().enums;
    for enum_def in enums.values() {
        diagnostics.extend(enum_::validate(db, enum_def.clone()));
    }

    diagnostics
}

pub fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    union_::check(db)
}

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    interface::check(db)
}

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    directive::validate(db)
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
    object::check(db)
}

pub fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    operation::check(db, file_id)
}

pub fn validate_unused_variable(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    unused_variable::check(db, file_id)
}
