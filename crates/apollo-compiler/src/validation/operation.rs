use crate::collections::HashSet;
use crate::executable;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DepthCounter;
use crate::validation::DepthGuard;
use crate::validation::DiagnosticList;
use crate::validation::ExecutableValidationContext;
use crate::validation::RecursionLimitError;
use crate::ExecutableDocument;
use crate::Name;
use crate::Node;

/// Iterate all selections in the selection set.
///
/// This includes fields, fragment spreads, and inline fragments. For fragments, both the spread
/// and the fragment's nested selections are reported.
///
/// Does not recurse into nested fields.
fn walk_selections<'doc>(
    document: &'doc ExecutableDocument,
    selections: &'doc executable::SelectionSet,
    mut f: impl FnMut(&'doc executable::Selection),
) -> Result<(), RecursionLimitError> {
    fn walk_selections_inner<'doc>(
        document: &'doc ExecutableDocument,
        selection_set: &'doc executable::SelectionSet,
        seen: &mut HashSet<&'doc Name>,
        mut guard: DepthGuard<'_>,
        f: &mut dyn FnMut(&'doc executable::Selection),
    ) -> Result<(), RecursionLimitError> {
        for selection in &selection_set.selections {
            f(selection);
            match selection {
                executable::Selection::Field(_) => {
                    // Nothing to do
                }
                executable::Selection::FragmentSpread(fragment) => {
                    let new = seen.insert(&fragment.fragment_name);
                    if !new {
                        continue;
                    }

                    // If the fragment doesn't exist, that error is reported elsewhere.
                    if let Some(fragment_definition) =
                        document.fragments.get(&fragment.fragment_name)
                    {
                        walk_selections_inner(
                            document,
                            &fragment_definition.selection_set,
                            seen,
                            guard.increment()?,
                            f,
                        )?;
                    }
                }
                executable::Selection::InlineFragment(fragment) => {
                    walk_selections_inner(
                        document,
                        &fragment.selection_set,
                        seen,
                        guard.increment()?,
                        f,
                    )?;
                }
            }
        }
        Ok(())
    }

    // This has a much higher limit than comparable recursive walks, like the one in
    // `validate_fragment_cycles`, despite doing similar work. This is because this limit
    // was introduced later and should not break (reasonable) existing queries that are
    // under that pre-existing limit. Luckily the existing limit was very conservative.
    let mut depth = DepthCounter::new().with_limit(500);

    walk_selections_inner(
        document,
        selections,
        &mut HashSet::default(),
        depth.guard(),
        &mut f,
    )
}

pub(crate) fn validate_subscription(
    document: &executable::ExecutableDocument,
    operation: &Node<executable::Operation>,
    diagnostics: &mut DiagnosticList,
) {
    if !operation.is_subscription() {
        return;
    }

    let mut field_names = vec![];

    let walked = walk_selections(document, &operation.selection_set, |selection| {
        if let executable::Selection::Field(field) = selection {
            field_names.push(field.name.clone());
            if matches!(field.name.as_str(), "__type" | "__schema" | "__typename") {
                diagnostics.push(
                    field.location(),
                    executable::BuildError::SubscriptionUsesIntrospection {
                        name: operation.name.clone(),
                        field: field.name.clone(),
                    },
                );
            }
        }

        if let Some(conditional_directive) = selection
            .directives()
            .iter()
            .find(|d| matches!(d.name.as_str(), "skip" | "include"))
        {
            diagnostics.push(
                conditional_directive.location(),
                executable::BuildError::SubscriptionUsesConditionalSelection {
                    name: operation.name.clone(),
                },
            );
        }
    });

    if walked.is_err() {
        diagnostics.push(None, DiagnosticData::RecursionError {});
        return;
    }

    if field_names.len() > 1 {
        diagnostics.push(
            operation.location(),
            executable::BuildError::SubscriptionUsesMultipleFields {
                name: operation.name.clone(),
                fields: field_names,
            },
        );
    }
}

pub(crate) fn validate_operation(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    operation: &executable::Operation,
    context: &ExecutableValidationContext<'_>,
) {
    let against_type = if let Some(schema) = context.schema() {
        schema
            .root_operation(operation.operation_type)
            .map(|ty| (schema, ty))
    } else {
        None
    };

    super::directive::validate_directives(
        diagnostics,
        context.schema(),
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
    );
    super::variable::validate_variable_definitions(
        diagnostics,
        context.schema(),
        &operation.variables,
    );

    super::variable::validate_unused_variables(diagnostics, document, operation);
    super::selection::validate_selection_set(
        diagnostics,
        document,
        against_type,
        &operation.selection_set,
        &mut context.operation_context(&operation.variables),
    );
}

pub(crate) fn validate_operation_definitions(
    diagnostics: &mut DiagnosticList,
    document: &ExecutableDocument,
    context: &ExecutableValidationContext<'_>,
) {
    for operation in document.operations.iter() {
        validate_operation(diagnostics, document, operation, context);
    }
}
