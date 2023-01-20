use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use miette::SourceSpan;

use crate::{
    database::db::Upcast,
    diagnostics::{UndefinedDefinition, UniqueArgument, UnsupportedLocation},
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
    fn validate_subscription_operations(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;
    fn validate_unused_variable(&self, file_id: FileId) -> Vec<ApolloDiagnostic>;

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
    fn check_scalar_type_definition(
        &self,
        def: Arc<hir::ScalarTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_union_type_definition(
        &self,
        def: Arc<hir::UnionTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_enum_type_definition(
        &self,
        def: Arc<hir::EnumTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_enum_value(&self, enum_val: hir::EnumValueDefinition) -> Vec<ApolloDiagnostic>;
    fn check_input_object_type_definition(
        &self,
        def: Arc<hir::InputObjectTypeDefinition>,
    ) -> Vec<ApolloDiagnostic>;
    fn check_schema_definition(&self, def: Arc<hir::SchemaDefinition>) -> Vec<ApolloDiagnostic>;
    fn check_selection_set(&self, selection_set: hir::SelectionSet) -> Vec<ApolloDiagnostic>;
    fn check_selection(&self, selection: Vec<hir::Selection>) -> Vec<ApolloDiagnostic>;
    fn validate_arguments_definition(
        &self,
        arguments_def: hir::ArgumentsDefinition,
    ) -> Vec<ApolloDiagnostic>;
    fn validate_arguments(&self, schema: Vec<hir::Argument>) -> Vec<ApolloDiagnostic>;
    fn check_field_definition(&self, field: hir::FieldDefinition) -> Vec<ApolloDiagnostic>;
    fn check_input_values(
        &self,
        input_values: Arc<Vec<hir::InputValueDefinition>>,
        dir_loc: hir::DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;
    fn check_db_definitions(&self) -> Vec<ApolloDiagnostic>;
    fn check_directives(
        &self,
        dirs: Vec<hir::Directive>,
        dir_loc: hir::DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;
    fn check_directive(
        &self,
        schema: hir::Directive,
        dir_loc: hir::DirectiveLocation,
    ) -> Vec<ApolloDiagnostic>;
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

pub fn check_scalar_type_definition(
    _db: &dyn ValidationDatabase,
    _union_type: Arc<hir::ScalarTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    // TODO: validate extensions
    vec![]
}

pub fn check_union_type_definition(
    _db: &dyn ValidationDatabase,
    _union_type: Arc<hir::UnionTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    // TODO: validate extensions
    vec![]
}

pub fn check_enum_type_definition(
    db: &dyn ValidationDatabase,
    enum_def: Arc<hir::EnumTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for val in enum_def.enum_values_definition() {
        diagnostics.extend(db.check_enum_value(val.clone()))
    }

    diagnostics
}

pub fn check_enum_value(
    db: &dyn ValidationDatabase,
    enum_val: hir::EnumValueDefinition,
) -> Vec<ApolloDiagnostic> {
    db.check_directives(
        enum_val.directives().to_vec(),
        hir::DirectiveLocation::EnumValue,
    )
}

pub fn check_input_object_type_definition(
    db: &dyn ValidationDatabase,
    input_obj: Arc<hir::InputObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.check_input_values(
        input_obj.input_fields_definition.clone(),
        hir::DirectiveLocation::InputFieldDefinition,
    ));

    diagnostics
}

pub fn check_schema_definition(
    db: &dyn ValidationDatabase,
    schema_def: Arc<hir::SchemaDefinition>,
) -> Vec<ApolloDiagnostic> {
    db.check_directives(
        schema_def.directives().to_vec(),
        hir::DirectiveLocation::Schema,
    )
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
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(db.check_directives(
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => diagnostics.extend(db.check_directives(
                inline.directives().to_vec(),
                hir::DirectiveLocation::InlineFragment,
            )),
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

    diagnostics.extend(db.check_directives(
        field.directives().to_vec(),
        hir::DirectiveLocation::FieldDefinition,
    ));

    diagnostics.extend(db.validate_arguments_definition(field.arguments));

    diagnostics
}

pub fn check_input_values(
    db: &dyn ValidationDatabase,
    input_values: Arc<Vec<hir::InputValueDefinition>>,
    // directive location depends on parent node location, so we pass this down from parent
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::InputValueDefinition> = HashMap::new();

    for input_value in input_values.iter() {
        diagnostics.extend(db.check_directives(input_value.directives().to_vec(), dir_loc));
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

pub fn check_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: Arc<Vec<hir::VariableDefinition>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for variable in variables.iter() {
        diagnostics.extend(db.check_directives(
            variable.directives().to_vec(),
            hir::DirectiveLocation::VariableDefinition,
        ));
    }

    diagnostics
}

pub fn check_db_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
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
        diagnostics
            .extend(db.check_directives(def.directives().to_vec(), def.operation_ty().into()));
        diagnostics.extend(db.check_variable_definitions(def.variables.clone()));
        diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
    }
    for def in db.all_fragments().values() {
        diagnostics.extend(db.check_directives(
            def.directives().to_vec(),
            hir::DirectiveLocation::FragmentDefinition,
        ));
        diagnostics.extend(db.check_selection_set(def.selection_set().clone()));
    }
    for def in directives.values() {
        diagnostics.extend(db.check_directive_definition(def.clone()));
    }
    for def in scalars.values() {
        diagnostics
            .extend(db.check_directives(def.directives().to_vec(), hir::DirectiveLocation::Scalar));
        diagnostics.extend(db.check_scalar_type_definition(def.clone()));
    }
    for def in objects.values() {
        diagnostics
            .extend(db.check_directives(def.directives().to_vec(), hir::DirectiveLocation::Object));
        diagnostics.extend(db.check_object_type_definition(def.clone()));
    }
    for def in interfaces.values() {
        diagnostics.extend(
            db.check_directives(def.directives().to_vec(), hir::DirectiveLocation::Interface),
        );
        diagnostics.extend(db.check_interface_type_definition(def.clone()));
        // TODO: validate extensions
    }
    for def in unions.values() {
        diagnostics
            .extend(db.check_directives(def.directives().to_vec(), hir::DirectiveLocation::Union));
        diagnostics.extend(db.check_union_type_definition(def.clone()));
        // TODO: validate extensions
    }
    for def in enums.values() {
        diagnostics
            .extend(db.check_directives(def.directives().to_vec(), hir::DirectiveLocation::Enum));
        diagnostics.extend(db.check_enum_type_definition(def.clone()));
        // TODO: validate extensions
    }
    for def in input_objects.values() {
        diagnostics.extend(db.check_directives(
            def.directives().to_vec(),
            hir::DirectiveLocation::InputObject,
        ));
        diagnostics.extend(db.check_input_object_type_definition(def.clone()));
        // TODO: validate extensions
    }
    diagnostics.extend(db.check_schema_definition(schema.clone()));

    diagnostics
}

pub fn check_field(db: &dyn ValidationDatabase, field: Arc<hir::Field>) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics
        .extend(db.check_directives(field.directives().to_vec(), hir::DirectiveLocation::Field));
    diagnostics.extend(db.validate_arguments(field.arguments().to_vec()));

    diagnostics
}

pub fn check_directives(
    db: &dyn ValidationDatabase,
    dirs: Vec<hir::Directive>,
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for dir in dirs {
        diagnostics.extend(db.check_directive(dir.clone(), dir_loc));
    }
    diagnostics
}

pub fn check_directive(
    db: &dyn ValidationDatabase,
    directive: hir::Directive,
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_arguments(directive.arguments().to_vec()));

    let name = directive.name();
    let loc = directive.loc();
    let offset = loc.offset();
    let len = loc.node_len();

    if let Some(directive) = db.find_directive_definition_by_name(name.into()) {
        let directive_def_loc = directive
            .loc
            .map(|loc| SourceSpan::new(loc.offset().into(), loc.node_len().into()));
        let allowed_loc: HashSet<hir::DirectiveLocation> =
            HashSet::from_iter(directive.directive_locations().iter().cloned());
        if !allowed_loc.contains(&dir_loc) {
            diagnostics.push(ApolloDiagnostic::UnsupportedLocation(UnsupportedLocation {
                ty: name.into(),
                dir_loc,
                src: db.source_code(loc.file_id()),
                directive: (offset, len).into(),
                directive_def: directive_def_loc,
                help: Some("the directive must be used in a location that the service has declared support for".into()),
            }))
        }
    } else {
        diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
            ty: name.into(),
            src: db.source_code(loc.file_id()),
            definition: (offset, len).into(),
        }))
    }

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

    diagnostics.extend(db.validate_schema());
    diagnostics.extend(db.check_db_definitions());

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
    diagnostics.extend(db.validate_subscription_operations(file_id));

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

pub fn validate_subscription_operations(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    subscription::check(db, file_id)
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
