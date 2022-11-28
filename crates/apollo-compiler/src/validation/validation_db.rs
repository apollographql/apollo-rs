use std::{collections::HashMap, sync::Arc};

use crate::{
    database::db::Upcast,
    diagnostics::UniqueArgument,
    hir,
    validation::{
        directive, enum_, input_object, interface, object, operation, scalar, schema, union_,
        unused_variable,
    },
    ApolloDiagnostic, AstDatabase, DocumentDatabase, FileId, HirDatabase, InputDatabase,
};

#[salsa::query_group(ValidationStorage)]
pub trait ValidationDatabase:
    Upcast<dyn DocumentDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    /// Validate all documents.
    fn validate(&self) -> Vec<ApolloDiagnostic>;

    /// Validate the schema, combined of all schema documents known to the compiler.
    fn validate_schema(&self) -> Vec<ApolloDiagnostic>;
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
    fn validate_unused_variable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

    fn check_directive_definition(
        &self,
        directive: hir::DirectiveDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_object_type_definition(
        &self,
        object_type: hir::ObjectTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_interface_type_definition(
        &self,
        object_type: hir::InterfaceTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_union_type_definition(
        &self,
        object_type: hir::UnionTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_enum_type_definition(
        &self,
        object_type: hir::EnumTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_input_object_type_definition(
        &self,
        object_type: hir::InputObjectTypeDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_schema_definition(&self, object_type: hir::SchemaDefinition) -> Vec<ApolloDiagnostic>;
    fn check_selection_set(&self, selection_set: hir::SelectionSet) -> Vec<ApolloDiagnostic>;
    fn check_arguments_definition(
        &self,
        arguments_def: hir::ArgumentsDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn check_field_definition(&self, field: hir::FieldDefinition) -> Vec<ApolloDiagnostic>;
    fn check_input_values(
        &self,
        input_values: Arc<Vec<hir::InputValueDefinition>>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_db_definitions(&self, definitions: Arc<Vec<hir::Definition>>)
        -> Vec<ApolloDiagnostic>;
    fn check_directive(&self, schema: hir::Directive) -> Vec<ApolloDiagnostic>;
    fn check_arguments(&self, schema: Vec<hir::Argument>) -> Vec<ApolloDiagnostic>;
    fn check_field(&self, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic>;
}

pub fn check_directive_definition(
    db: &dyn ValidationDatabase,
    directive: hir::DirectiveDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.check_arguments_definition(directive.arguments));

    diagnostics
}

pub fn check_object_type_definition(
    db: &dyn ValidationDatabase,
    object_type: hir::ObjectTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for field in object_type.fields_definition() {
        diagnostics.extend(db.check_field_definition(field.clone()));
    }

    diagnostics
}

pub fn check_interface_type_definition(
    db: &dyn ValidationDatabase,
    interface_type: hir::InterfaceTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for field in interface_type.fields_definition() {
        diagnostics.extend(db.check_field_definition(field.clone()));
    }

    diagnostics
}

pub fn check_union_type_definition(
    _db: &dyn ValidationDatabase,
    _union_type: hir::UnionTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    vec![]
}

pub fn check_enum_type_definition(
    _db: &dyn ValidationDatabase,
    _enum_type: hir::EnumTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    vec![]
}

pub fn check_input_object_type_definition(
    _db: &dyn ValidationDatabase,
    _input_object_type: hir::InputObjectTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    // Not checking the `input_values` here as those are checked as fields elsewhere.
    vec![]
}

pub fn check_schema_definition(
    db: &dyn ValidationDatabase,
    schema_def: hir::SchemaDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for directive in schema_def.directives() {
        diagnostics.extend(db.check_directive(directive.clone()));
    }

    diagnostics
}

pub fn check_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for selection in selection_set.selection.iter() {
        match selection {
            hir::Selection::Field(field) => {
                diagnostics.extend(db.check_field(Arc::clone(field)));
            }
            hir::Selection::FragmentSpread(_) | hir::Selection::InlineFragment(_) => {
                // no diagnostics yet
            }
        }
    }

    diagnostics
}

pub fn check_arguments_definition(
    db: &dyn ValidationDatabase,
    arguments_def: hir::ArgumentsDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.check_input_values(arguments_def.input_values));

    diagnostics
}

pub fn check_field_definition(
    db: &dyn ValidationDatabase,
    field: hir::FieldDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for directive in field.directives() {
        diagnostics.extend(db.check_directive(directive.clone()));
    }

    diagnostics.extend(db.check_arguments_definition(field.arguments));

    diagnostics
}

pub fn check_input_values(
    db: &dyn ValidationDatabase,
    input_values: Arc<Vec<hir::InputValueDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::InputValueDefinition> = HashMap::new();

    for input_value in input_values.iter() {
        let name = input_value.name();
        if let Some(prev_arg) = seen.get(name) {
            let prev_offset = prev_arg.loc().unwrap().offset();
            let prev_node_len = prev_arg.loc().unwrap().node_len();

            let current_offset = input_value.loc().unwrap().offset();
            let current_node_len = input_value.loc().unwrap().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueArgument(UniqueArgument {
                name: name.into(),
                src: db.source_code(prev_arg.loc().unwrap().file_id()),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!("`{name}` argument must only be defined once.")),
            }));
        } else {
            seen.insert(name, input_value);
        }
    }

    diagnostics
}

pub fn check_db_definitions(
    db: &dyn ValidationDatabase,
    definitions: Arc<Vec<hir::Definition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for definition in definitions.iter() {
        for directive in definition.directives() {
            diagnostics.extend(db.check_directive(directive.clone()));
        }

        use hir::Definition::*;
        match definition {
            OperationDefinition(def) => {
                diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
            }
            FragmentDefinition(def) => {
                diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
            }
            DirectiveDefinition(def) => {
                diagnostics.extend(db.check_directive_definition(def.clone()));
            }
            ScalarTypeDefinition(_def) => {}
            ObjectTypeDefinition(def) => {
                diagnostics.extend(db.check_object_type_definition(def.clone()));
            }
            InterfaceTypeDefinition(def) => {
                diagnostics.extend(db.check_interface_type_definition(def.clone()));
            }
            UnionTypeDefinition(def) => {
                diagnostics.extend(db.check_union_type_definition(def.clone()));
            }
            EnumTypeDefinition(def) => {
                diagnostics.extend(db.check_enum_type_definition(def.clone()));
            }
            InputObjectTypeDefinition(def) => {
                diagnostics.extend(db.check_input_object_type_definition(def.clone()));
            }
            SchemaDefinition(def) => {
                diagnostics.extend(db.check_schema_definition(def.clone()));
            }
            // FIXME: what validation is needed for extensions?
            SchemaExtension(_) => {}
            ScalarTypeExtension(_) => {}
            ObjectTypeExtension(_) => {}
            InterfaceTypeExtension(_) => {}
            UnionTypeExtension(_) => {}
            EnumTypeExtension(_) => {}
            InputObjectTypeExtension(_) => {}
        }
    }

    diagnostics
}

pub fn check_field(db: &dyn ValidationDatabase, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for directive in field.directives.iter() {
        diagnostics.extend(db.check_directive(directive.clone()));
    }
    diagnostics.extend(db.check_arguments(field.arguments().to_vec()));

    diagnostics
}

pub fn check_directive(
    db: &dyn ValidationDatabase,
    directive: hir::Directive,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.check_arguments(directive.arguments().to_vec()));

    diagnostics
}

pub fn check_arguments(
    db: &dyn ValidationDatabase,
    arguments: Vec<hir::Argument>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::Argument> = HashMap::new();

    for argument in &arguments {
        let name = argument.name();
        if let Some(prev_arg) = seen.get(name) {
            let prev_offset = prev_arg.loc().offset();
            let prev_node_len = prev_arg.loc().node_len();

            let current_offset = argument.loc().offset();
            let current_node_len = argument.loc().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueArgument(UniqueArgument {
                name: name.into(),
                src: db.source_code(prev_arg.loc().file_id()),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!("`{name}` argument must only be provided once.")),
            }));
        } else {
            seen.insert(name, argument);
        }
    }

    diagnostics
}

pub fn validate(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(db.syntax_errors());

    diagnostics.extend(db.validate_schema());
    diagnostics.extend(db.check_db_definitions(db.db_definitions()));

    for file_id in db.executable_definition_files() {
        diagnostics.extend(db.validate_executable(file_id));
    }

    diagnostics
}

pub fn validate_schema(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

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

    diagnostics
}

pub fn validate_schema_definition(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    schema::check(db)
}

pub fn validate_scalar_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    scalar::check(db)
}

pub fn validate_enum_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    enum_::check(db)
}

pub fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    union_::check(db)
}

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    interface::check(db)
}

pub fn validate_directive_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    directive::check(db)
}

pub fn validate_input_object_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    input_object::check(db)
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

// #[salsa::query_group(ValidationStorage)]
// pub trait Validation: Document + Inputs + DocumentParser + Definitions {
//     fn validate(&self) -> Arc<Vec<ApolloDiagnostic>>;
// }
//
// pub fn validate(db: &dyn Validation) -> Arc<Vec<ApolloDiagnostic>> {
//     let mut diagnostics = Vec::new();
//     diagnostics.extend(schema::check(db));
//
//     Arc::new(diagnostics)
// }
