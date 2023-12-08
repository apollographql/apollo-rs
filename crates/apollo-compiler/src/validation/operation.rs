use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::FileId;
use crate::{ast, name, Node, ValidationDatabase};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct OperationValidationConfig<'vars> {
    /// When false, rules that require a schema to validate are disabled.
    pub has_schema: bool,
    /// The variables defined for this operation.
    pub variables: &'vars [Node<ast::VariableDefinition>],
}

pub(crate) fn validate_operation(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    operation: Node<ast::OperationDefinition>,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    let config = OperationValidationConfig {
        has_schema,
        variables: &operation.variables,
    };

    let schema = db.schema();
    let against_type = schema.root_operation(operation.operation_type);

    let named_fragments = db.ast_named_fragments(file_id);
    let q = name!("Query");

    if operation.operation_type == ast::OperationType::Subscription {
        let fields = super::selection::operation_fields(
            &named_fragments,
            against_type.unwrap_or(&q),
            &operation.selection_set,
        );

        if fields.len() > 1 {
            diagnostics.push(ValidationError::new(
                operation.location(),
                DiagnosticData::SingleRootField {
                    fields: fields
                        .iter()
                        .map(|field| field.field.name.clone())
                        .collect(),
                },
            ));
        }

        let has_introspection_fields = fields
            .iter()
            .find(|field| {
                matches!(
                    field.field.name.as_str(),
                    "__type" | "__schema" | "__typename"
                )
            })
            .map(|field| field.field);
        if let Some(field) = has_introspection_fields {
            diagnostics.push(ValidationError::new(
                field.location(),
                DiagnosticData::IntrospectionField {
                    field: field.name.to_string(),
                },
            ));
        }
    }

    diagnostics.extend(super::directive::validate_directives(
        db,
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
    ));
    diagnostics.extend(super::variable::validate_variable_definitions(
        db,
        &operation.variables,
        config.has_schema,
    ));

    diagnostics.extend(super::variable::validate_unused_variables(
        db,
        file_id,
        operation.clone(),
    ));
    diagnostics.extend(super::selection::validate_selection_set(
        db,
        file_id,
        against_type,
        &operation.selection_set,
        config,
    ));

    diagnostics
}

pub(crate) fn validate_operation_definitions_inner(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();
    let document = db.ast(file_id);

    for definition in &document.definitions {
        if let ast::Definition::OperationDefinition(operation) = definition {
            diagnostics.extend(validate_operation(
                db,
                file_id,
                operation.clone(),
                has_schema,
            ));
        }
    }

    diagnostics
}

pub(crate) fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ValidationError> {
    validate_operation_definitions_inner(db, file_id, false)
}
