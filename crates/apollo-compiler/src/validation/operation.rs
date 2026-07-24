use crate::ast;
use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::executable;
use crate::parser::SourceSpan;
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

/// Validate `@defer` directive usage per the GraphQL Defer & Stream spec
/// (PR <https://github.com/graphql/graphql-spec/pull/1110>):
///
/// 1. `@defer(label:)` values must be unique across the document, and the
///    `label` argument must not be a variable.
/// 2. `@defer` is not allowed on root selections of `mutation` or
///    `subscription` operations (recursing through fragment spreads/inline
///    fragments at the root level).
/// 3. In a `subscription` operation, every `@defer` directive that is not
///    statically skipped via `@skip`/`@include` must be disabled via an
///    `if` argument set to `false` or to a variable.
pub(crate) fn validate_defer(document: &ExecutableDocument, diagnostics: &mut DiagnosticList) {
    validate_defer_labels(document, diagnostics);

    for operation in document.operations.iter() {
        let op_kind = match operation.operation_type {
            ast::OperationType::Query => continue,
            ast::OperationType::Mutation => "mutation",
            ast::OperationType::Subscription => "subscription",
        };
        let _ = forbid_defer_on_root(
            document,
            &operation.selection_set,
            op_kind,
            diagnostics,
            &mut HashSet::default(),
            DepthCounter::new().with_limit(500).guard(),
        );
        if operation.is_subscription() {
            let _ = forbid_unconditional_defer(
                document,
                &operation.selection_set,
                diagnostics,
                &mut HashSet::default(),
                DepthCounter::new().with_limit(500).guard(),
            );
        }
    }
}

fn validate_defer_labels(document: &ExecutableDocument, diagnostics: &mut DiagnosticList) {
    let mut seen: HashMap<String, Option<SourceSpan>> = HashMap::default();

    let walk = |selection_set: &executable::SelectionSet,
                diagnostics: &mut DiagnosticList,
                seen: &mut HashMap<String, Option<SourceSpan>>| {
        let _ = walk_defers_in_selection_set(
            selection_set,
            &mut |directive| check_defer_label(directive, diagnostics, seen),
            DepthCounter::new().with_limit(500).guard(),
        );
    };
    for operation in document.operations.iter() {
        walk(&operation.selection_set, diagnostics, &mut seen);
    }
    for fragment in document.fragments.values() {
        walk(&fragment.selection_set, diagnostics, &mut seen);
    }
}

fn check_defer_label(
    directive: &Node<executable::Directive>,
    diagnostics: &mut DiagnosticList,
    seen: &mut HashMap<String, Option<SourceSpan>>,
) {
    let Some(label_arg) = directive.specified_argument_by_name("label") else {
        return;
    };
    match label_arg.as_ref() {
        ast::Value::Variable(_) => {
            diagnostics.push(
                label_arg.location(),
                executable::BuildError::DeferLabelMustNotBeVariable,
            );
        }
        ast::Value::String(label) => {
            if let Some(&prev) = seen.get(label) {
                diagnostics.push(
                    label_arg.location(),
                    executable::BuildError::DuplicateDeferLabel {
                        label: label.clone(),
                        original_location: prev,
                    },
                );
            } else {
                seen.insert(label.clone(), label_arg.location());
            }
        }
        // Non-string, non-variable values are rejected by argument coercion validation.
        _ => {}
    }
}

/// Walks the selection set, invoking `f` on every `@defer` directive attached
/// to a selection. Does not follow fragment spreads — callers must iterate
/// `document.fragments` separately to visit directives inside fragment
/// definitions.
fn walk_defers_in_selection_set<'doc, F>(
    selection_set: &'doc executable::SelectionSet,
    f: &mut F,
    mut guard: DepthGuard<'_>,
) -> Result<(), RecursionLimitError>
where
    F: FnMut(&'doc Node<executable::Directive>),
{
    for selection in &selection_set.selections {
        for directive in selection.directives().iter() {
            if directive.name == "defer" {
                f(directive);
            }
        }
        match selection {
            executable::Selection::Field(field) => {
                walk_defers_in_selection_set(&field.selection_set, f, guard.increment()?)?;
            }
            executable::Selection::InlineFragment(frag) => {
                walk_defers_in_selection_set(&frag.selection_set, f, guard.increment()?)?;
            }
            executable::Selection::FragmentSpread(_) => {}
        }
    }
    Ok(())
}

fn forbid_defer_on_root<'doc>(
    document: &'doc ExecutableDocument,
    selection_set: &'doc executable::SelectionSet,
    operation_type: &'static str,
    diagnostics: &mut DiagnosticList,
    visited_fragments: &mut HashSet<&'doc Name>,
    mut guard: DepthGuard<'_>,
) -> Result<(), RecursionLimitError> {
    for selection in &selection_set.selections {
        match selection {
            // Spec ForbidDeferStream only checks @stream on Fields; @stream is not
            // covered by this @defer-focused validation.
            executable::Selection::Field(_) => {}
            executable::Selection::InlineFragment(inline) => {
                report_root_defer(&inline.directives, operation_type, diagnostics);
                forbid_defer_on_root(
                    document,
                    &inline.selection_set,
                    operation_type,
                    diagnostics,
                    visited_fragments,
                    guard.increment()?,
                )?;
            }
            executable::Selection::FragmentSpread(spread) => {
                report_root_defer(&spread.directives, operation_type, diagnostics);
                if !visited_fragments.insert(&spread.fragment_name) {
                    continue;
                }
                if let Some(fragment) = document.fragments.get(&spread.fragment_name) {
                    forbid_defer_on_root(
                        document,
                        &fragment.selection_set,
                        operation_type,
                        diagnostics,
                        visited_fragments,
                        guard.increment()?,
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn report_root_defer(
    directives: &executable::DirectiveList,
    operation_type: &'static str,
    diagnostics: &mut DiagnosticList,
) {
    for directive in directives.iter() {
        if directive.name == "defer" {
            diagnostics.push(
                directive.location(),
                executable::BuildError::DeferOnRootMutationOrSubscriptionField { operation_type },
            );
        }
    }
}

fn forbid_unconditional_defer<'doc>(
    document: &'doc ExecutableDocument,
    selection_set: &'doc executable::SelectionSet,
    diagnostics: &mut DiagnosticList,
    visited_fragments: &mut HashSet<&'doc Name>,
    mut guard: DepthGuard<'_>,
) -> Result<(), RecursionLimitError> {
    for selection in &selection_set.selections {
        if selection_may_be_excluded(selection.directives()) {
            continue;
        }
        for directive in selection.directives().iter() {
            if directive.name == "defer" && !defer_can_be_disabled(directive) {
                diagnostics.push(
                    directive.location(),
                    executable::BuildError::DeferInSubscriptionMustBeConditional,
                );
            }
        }
        match selection {
            executable::Selection::Field(field) => {
                forbid_unconditional_defer(
                    document,
                    &field.selection_set,
                    diagnostics,
                    visited_fragments,
                    guard.increment()?,
                )?;
            }
            executable::Selection::InlineFragment(inline) => {
                forbid_unconditional_defer(
                    document,
                    &inline.selection_set,
                    diagnostics,
                    visited_fragments,
                    guard.increment()?,
                )?;
            }
            executable::Selection::FragmentSpread(spread) => {
                if !visited_fragments.insert(&spread.fragment_name) {
                    continue;
                }
                if let Some(fragment) = document.fragments.get(&spread.fragment_name) {
                    forbid_unconditional_defer(
                        document,
                        &fragment.selection_set,
                        diagnostics,
                        visited_fragments,
                        guard.increment()?,
                    )?;
                }
            }
        }
    }
    Ok(())
}

/// True when the selection may be excluded by `@skip` or `@include` — and so
/// we should not flag any `@defer` inside it. Per spec, `@skip(if:)` excludes
/// the selection when its value is anything other than literal `false`, and
/// `@include(if:)` excludes the selection when its value is anything other
/// than literal `true`. A missing or variable `if` argument is conservatively
/// treated as "may be excluded" — matching graphql-js, this avoids
/// double-reporting with the `ProvidedRequiredArguments` rule and avoids
/// false positives on runtime-conditional selections.
fn selection_may_be_excluded(directives: &executable::DirectiveList) -> bool {
    for directive in directives.iter() {
        if directive.name == "skip" {
            match directive
                .specified_argument_by_name("if")
                .map(|a| a.as_ref())
            {
                Some(ast::Value::Boolean(false)) => {}
                _ => return true,
            }
        } else if directive.name == "include" {
            match directive
                .specified_argument_by_name("if")
                .map(|a| a.as_ref())
            {
                Some(ast::Value::Boolean(true)) => {}
                _ => return true,
            }
        }
    }
    false
}

/// True when `@defer`'s `if` argument is `false` or a variable — i.e. the
/// directive can be disabled at runtime. A missing or literal-`true` `if`
/// argument means the directive is unconditionally active.
fn defer_can_be_disabled(directive: &executable::Directive) -> bool {
    let Some(arg) = directive.specified_argument_by_name("if") else {
        return false;
    };
    matches!(
        arg.as_ref(),
        ast::Value::Boolean(false) | ast::Value::Variable(_)
    )
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
