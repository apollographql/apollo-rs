use crate::validation::diagnostics::ValidationError;
use crate::validation::DiagnosticList;
use crate::validation::FileId;
use crate::{ast, executable, Node, ValidationDatabase};

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
    file_id: FileId,
    operation: &Node<ast::OperationDefinition>,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    let config = OperationValidationConfig {
        has_schema,
        variables: &operation.variables,
    };

    let schema;
    let against_type = if has_schema {
        schema = db.schema();
        schema.root_operation(operation.operation_type)
    } else {
        None
    };

    diagnostics.extend(super::directive::validate_directives(
        db,
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
        has_schema,
    ));
    diagnostics.extend(super::variable::validate_variable_definitions(
        db,
        &operation.variables,
        config.has_schema,
    ));

    diagnostics.extend(super::variable::validate_unused_variables(db, operation));
    diagnostics.extend(super::selection::validate_selection_set(
        db,
        file_id,
        against_type,
        &operation.selection_set,
        config,
    ));

    diagnostics
}

pub(crate) fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    has_schema: bool,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();
    let document = db.executable_ast();

    for definition in &document.definitions {
        if let ast::Definition::OperationDefinition(operation) = definition {
            diagnostics.extend(validate_operation(db, file_id, operation, has_schema));
        }
    }

    diagnostics
}
