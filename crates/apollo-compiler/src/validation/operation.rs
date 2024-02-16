use crate::validation::diagnostics::ValidationError;
use crate::validation::DiagnosticList;
use crate::{ast, executable, ExecutableDocument, Node, Schema, ValidationDatabase};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct OperationValidationConfig<'vars> {
    /// When false, rules that require a schema to validate are disabled.
    pub has_schema: bool,
    /// The variables defined for this operation.
    pub variables: &'vars [Node<ast::VariableDefinition>],
}

pub(crate) fn validate_subscription(
    document: &executable::ExecutableDocument,
    operation: &Node<executable::Operation>,
    diagnostics: &mut DiagnosticList,
) {
    if operation.is_subscription() {
        let fields = super::selection::expand_selections(
            &document.fragments,
            std::iter::once(&operation.selection_set),
        );

        if fields.len() > 1 {
            diagnostics.push(
                operation.location(),
                executable::BuildError::SubscriptionUsesMultipleFields {
                    name: operation.name.clone(),
                    fields: fields
                        .iter()
                        .map(|field| field.field.name.clone())
                        .collect(),
                },
            );
        }

        let has_introspection_fields = fields
            .iter()
            .find(|field| {
                matches!(
                    field.field.name.as_str(),
                    "__type" | "__schema" | "__typename"
                )
            })
            .map(|field| &field.field);
        if let Some(field) = has_introspection_fields {
            diagnostics.push(
                field.location(),
                executable::BuildError::SubscriptionUsesIntrospection {
                    name: operation.name.clone(),
                    field: field.name.clone(),
                },
            );
        }
    }
}

pub(crate) fn validate_operation(
    db: &dyn ValidationDatabase,
    document: &ExecutableDocument,
    schema: Option<&Schema>,
    operation: &executable::Operation,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    let config = OperationValidationConfig {
        has_schema: schema.is_some(),
        variables: &operation.variables,
    };

    let against_type = if let Some(schema) = schema {
        schema.root_operation(operation.operation_type)
    } else {
        None
    };

    diagnostics.extend(super::directive::validate_directives(
        db,
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
        schema.is_some(),
    ));
    diagnostics.extend(super::variable::validate_variable_definitions(
        db,
        &operation.variables,
        config.has_schema,
    ));

    diagnostics.extend(super::variable::validate_unused_variables(
        document, operation,
    ));
    diagnostics.extend(super::selection::validate_selection_set(
        db,
        document,
        against_type,
        &operation.selection_set,
        config,
    ));

    diagnostics
}

pub(crate) fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    document: &ExecutableDocument,
    schema: Option<&Schema>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for operation in document.all_operations() {
        diagnostics.extend(validate_operation(db, document, schema, operation));
    }

    diagnostics
}
