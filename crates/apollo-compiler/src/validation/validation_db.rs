use crate::validation::diagnostics::ValidationError;
use crate::validation::directive::validate_directive_definitions;
use crate::validation::enum_::validate_enum_definitions;
use crate::validation::input_object::validate_input_object_definitions;
use crate::validation::interface::validate_interface_definitions;
use crate::validation::object::validate_object_type_definitions;
use crate::validation::scalar::validate_scalar_definitions;
use crate::validation::schema::validate_schema_definition;
use crate::validation::union_::validate_union_definitions;
use crate::{ast, Node, ReprDatabase};
use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(ValidationStorage)]
pub(crate) trait ValidationDatabase: ReprDatabase {
    fn ast_named_fragments(&self) -> Arc<HashMap<ast::Name, Node<ast::FragmentDefinition>>>;
}

pub(crate) fn ast_named_fragments(
    db: &dyn ValidationDatabase,
) -> Arc<HashMap<ast::Name, Node<ast::FragmentDefinition>>> {
    let document = db.executable_ast();
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

    diagnostics.extend(validate_schema_definition(db));

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
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(super::operation::validate_operation_definitions(
        db, has_schema,
    ));
    for def in db.ast_named_fragments().values() {
        diagnostics.extend(super::fragment::validate_fragment_used(db, def));
    }

    diagnostics
}

pub(crate) fn validate_standalone_executable(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    validate_executable_inner(db, false)
}

pub(crate) fn validate_executable(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    validate_executable_inner(db, true)
}
