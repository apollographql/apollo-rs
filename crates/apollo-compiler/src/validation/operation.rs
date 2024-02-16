use crate::validation::DiagnosticList;
use crate::{ast, executable, ExecutableDocument, Node, Schema};

#[derive(Debug, Clone)]
pub(crate) struct OperationValidationConfig<'a> {
    /// When None, rules that require a schema to validate are disabled.
    pub schema: Option<&'a crate::Schema>,
    /// The variables defined for this operation.
    pub variables: &'a [Node<ast::VariableDefinition>],
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
    diagnostics: &mut DiagnosticList,
    schema: Option<&Schema>,
    document: &ExecutableDocument,
    operation: &executable::Operation,
) {
    let config = OperationValidationConfig {
        schema,
        variables: &operation.variables,
    };

    let against_type = if let Some(schema) = schema {
        schema
            .root_operation(operation.operation_type)
            .map(|ty| (schema, ty))
    } else {
        None
    };

    super::directive::validate_directives(
        diagnostics,
        schema,
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
    );
    super::variable::validate_variable_definitions(
        diagnostics,
        config.schema,
        &operation.variables,
    );

    super::variable::validate_unused_variables(diagnostics, document, operation);
    super::selection::validate_selection_set(
        diagnostics,
        document,
        against_type,
        &operation.selection_set,
        config,
    );
}

pub(crate) fn validate_operation_definitions(
    diagnostics: &mut DiagnosticList,
    schema: Option<&Schema>,
    document: &ExecutableDocument,
) {
    for operation in document.all_operations() {
        validate_operation(diagnostics, schema, document, operation);
    }
}
