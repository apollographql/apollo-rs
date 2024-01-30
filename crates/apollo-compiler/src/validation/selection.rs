use super::operation::OperationValidationConfig;
use crate::validation::diagnostics::{DiagnosticData, ValidationError};
use crate::validation::{CycleError, FileId, RecursionGuard, RecursionStack, ValidationDatabase};
use crate::{ast, executable, schema, Node};
use indexmap::IndexMap;
use std::collections::{hash_map::Entry, HashMap};
use std::collections::{HashSet, VecDeque};

/// Return all possible unordered combinations of 2 elements from a slice.
fn pair_combinations<T>(slice: &[T]) -> impl Iterator<Item = (&T, &T)> {
    slice
        .iter()
        .enumerate()
        // Final element will zip with the empty slice and produce no result.
        .flat_map(|(index, element)| std::iter::repeat(element).zip(&slice[index + 1..]))
}

/// A field and the type it selects from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct FieldAgainstType<'a> {
    pub against_type: &'a ast::NamedType,
    pub field: &'a Node<ast::Field>,
}

// TODO(@goto-bus-stop) remove intermediate allocations
pub(crate) fn operation_fields<'a>(
    named_fragments: &'a HashMap<ast::Name, Node<ast::FragmentDefinition>>,
    against_type: &'a ast::NamedType,
    selections: &'a [ast::Selection],
) -> Vec<FieldAgainstType<'a>> {
    fn inner<'a>(
        named_fragments: &'a HashMap<ast::Name, Node<ast::FragmentDefinition>>,
        seen: &mut std::collections::HashSet<ast::Name>,
        against_type: &'a ast::NamedType,
        selections: &'a [ast::Selection],
    ) -> Vec<FieldAgainstType<'a>> {
        selections
            .iter()
            .flat_map(|selection| match selection {
                ast::Selection::Field(field) => vec![FieldAgainstType {
                    against_type,
                    field,
                }],
                ast::Selection::InlineFragment(inline) => inner(
                    named_fragments,
                    seen,
                    inline.type_condition.as_ref().unwrap_or(against_type),
                    &inline.selection_set,
                ),
                ast::Selection::FragmentSpread(spread) => {
                    if seen.contains(&spread.fragment_name) {
                        return vec![];
                    }
                    seen.insert(spread.fragment_name.clone());

                    named_fragments
                        .get(&spread.fragment_name)
                        .map(|fragment| {
                            inner(
                                named_fragments,
                                seen,
                                &fragment.type_condition,
                                &fragment.selection_set,
                            )
                        })
                        .unwrap_or_default()
                }
            })
            .collect()
    }
    inner(
        named_fragments,
        &mut Default::default(),
        against_type,
        selections,
    )
}

fn is_composite(ty: &schema::ExtendedType) -> bool {
    use schema::ExtendedType::*;
    matches!(ty, Object(_) | Interface(_) | Union(_))
}

/// Check if a selection set contains a fragment cycle, meaning we can't run recursive
/// validations on it.
fn contains_any_fragment_cycle(
    fragments: &IndexMap<ast::Name, Node<executable::Fragment>>,
    selection_set: &executable::SelectionSet,
) -> bool {
    let mut visited = RecursionStack::new().with_limit(100);

    return detect_fragment_cycle_inner(fragments, selection_set, &mut visited.guard()).is_err();

    fn detect_fragment_cycle_inner(
        fragments: &IndexMap<ast::Name, Node<executable::Fragment>>,
        selection_set: &executable::SelectionSet,
        visited: &mut RecursionGuard<'_>,
    ) -> Result<(), CycleError<()>> {
        for selection in &selection_set.selections {
            match selection {
                executable::Selection::FragmentSpread(spread) => {
                    if visited.contains(&spread.fragment_name) {
                        if visited.first() == Some(&spread.fragment_name) {
                            return Err(CycleError::Recursed(vec![]));
                        }
                        continue;
                    }

                    if let Some(fragment) = fragments.get(&spread.fragment_name) {
                        detect_fragment_cycle_inner(
                            fragments,
                            &fragment.selection_set,
                            &mut visited.push(&fragment.name)?,
                        )?;
                    }
                }
                executable::Selection::InlineFragment(inline) => {
                    detect_fragment_cycle_inner(fragments, &inline.selection_set, visited)?;
                }
                executable::Selection::Field(field) => {
                    detect_fragment_cycle_inner(fragments, &field.selection_set, visited)?;
                }
            }
        }
        Ok(())
    }
}

/// https://tech.new-work.se/graphql-overlapping-fields-can-be-merged-fast-ea6e92e0a01
pub(crate) fn fields_in_set_can_merge(
    schema: &schema::Schema,
    document: &executable::ExecutableDocument,
    selection_set: &executable::SelectionSet,
    diagnostics: &mut Vec<ValidationError>,
) {
    if contains_any_fragment_cycle(&document.fragments, selection_set) {
        return;
    }

    let fields = expand_selections(&document.fragments, &[selection_set]);

    same_response_shape_by_name(schema, &document.fragments, &fields, diagnostics);
    same_for_common_parents_by_name(schema, &document.fragments, &fields, diagnostics);

    /// Represents a field selected against a parent type.
    #[derive(Debug, Clone)]
    struct FieldSelection {
        /// The type of the selection set this field selection is part of.
        parent_type: ast::NamedType,
        field: Node<executable::Field>,
    }

    fn same_response_shape_by_name(
        schema: &schema::Schema,
        fragments: &IndexMap<ast::Name, Node<executable::Fragment>>,
        fields: &[FieldSelection],
        diagnostics: &mut Vec<ValidationError>,
    ) {
        for fields_for_name in group_selections_by_output_name(fields.iter().cloned()).values() {
            for (field_a, field_b) in pair_combinations(fields_for_name) {
                // Covers steps 3-5 of the spec algorithm.
                if let Err(err) = same_output_type_shape(&schema, field_a.clone(), field_b.clone())
                {
                    diagnostics.push(err);
                    continue;
                }

                let merged_set = expand_selections(
                    fragments,
                    &[&field_a.field.selection_set, &field_b.field.selection_set],
                );
                same_response_shape_by_name(schema, fragments, &merged_set, diagnostics);
            }
        }
    }

    fn same_for_common_parents_by_name(
        schema: &schema::Schema,
        fragments: &IndexMap<ast::Name, Node<executable::Fragment>>,
        fields: &[FieldSelection],
        diagnostics: &mut Vec<ValidationError>,
    ) {
        for fields_for_name in group_selections_by_output_name(fields.iter().cloned()).values() {
            for (field_a, field_b) in pair_combinations(fields_for_name) {
                if field_a.parent_type == field_b.parent_type {
                    // 2bi. fieldA and fieldB must have identical field names.
                    // 2bii. fieldA and fieldB must have identical sets of arguments.
                    if let Err(diagnostic) = same_field_selection(field_a, field_b) {
                        diagnostics.push(diagnostic);
                        continue;
                    }

                    let merged_set = expand_selections(
                        fragments,
                        &[&field_a.field.selection_set, &field_b.field.selection_set],
                    );
                    same_for_common_parents_by_name(schema, fragments, &merged_set, diagnostics);
                }
            }
        }
    }

    /// Expand one or more selection sets to a list of all fields selected.
    fn expand_selections(
        fragments: &IndexMap<ast::Name, Node<executable::Fragment>>,
        selection_sets: &[&executable::SelectionSet],
    ) -> Vec<FieldSelection> {
        let mut selections = vec![];
        let mut queue: VecDeque<&executable::SelectionSet> =
            selection_sets.iter().copied().collect();
        let mut seen_fragments = HashSet::new();

        while let Some(next_set) = queue.pop_front() {
            for selection in &next_set.selections {
                match selection {
                    executable::Selection::Field(field) => selections.push(FieldSelection {
                        parent_type: next_set.ty.clone(),
                        field: field.clone(),
                    }),
                    executable::Selection::InlineFragment(spread) => {
                        queue.push_back(&spread.selection_set)
                    }
                    executable::Selection::FragmentSpread(spread)
                        if !seen_fragments.contains(&spread.fragment_name) =>
                    {
                        seen_fragments.insert(&spread.fragment_name);
                        if let Some(fragment) = fragments.get(&spread.fragment_name) {
                            queue.push_back(&fragment.selection_set);
                        }
                    }
                    executable::Selection::FragmentSpread(_) => {
                        // Already seen
                    }
                }
            }
        }

        selections
    }

    fn group_selections_by_output_name(
        selections: impl Iterator<Item = FieldSelection>,
    ) -> HashMap<schema::Name, Vec<FieldSelection>> {
        let mut map = HashMap::new();
        for selection in selections {
            match map.entry(selection.field.response_key().clone()) {
                Entry::Vacant(entry) => {
                    entry.insert(vec![selection]);
                }
                Entry::Occupied(mut entry) => {
                    entry.get_mut().push(selection);
                }
            }
        }
        map
    }

    fn same_output_type_shape(
        schema: &schema::Schema,
        selection_a: FieldSelection,
        selection_b: FieldSelection,
    ) -> Result<(), ValidationError> {
        let field_a = &selection_a.field.definition;
        let field_b = &selection_b.field.definition;

        let mut type_a = &field_a.ty;
        let mut type_b = &field_b.ty;

        let mismatching_type_diagnostic = || {
            ValidationError::new(
                selection_b.field.location(),
                DiagnosticData::ConflictingFieldType {
                    field: selection_a.field.response_key().clone(),
                    original_selection: selection_a.field.location(),
                    original_type: field_a.ty.clone(),
                    redefined_selection: selection_b.field.location(),
                    redefined_type: field_b.ty.clone(),
                },
            )
        };

        // Steps 3 and 4 of the spec text unwrap both types simultaneously down to the named type.
        // The apollo-rs representation of NonNull and Lists makes it tricky to follow the steps
        // exactly.
        //
        // Instead we unwrap lists and non-null lists first, which leaves just a named type or a
        // non-null named type...
        while !type_a.is_named() || !type_b.is_named() {
            // 4. If typeA or typeB is List.
            // 4a. If typeA or typeB is not List, return false.
            // 4b. Let typeA be the item type of typeA
            // 4c. Let typeB be the item type of typeB
            (type_a, type_b) = match (type_a, type_b) {
                (ast::Type::List(type_a), ast::Type::List(type_b))
                | (ast::Type::NonNullList(type_a), ast::Type::NonNullList(type_b)) => {
                    (type_a.as_ref(), type_b.as_ref())
                }
                (ast::Type::List(_), _)
                | (_, ast::Type::List(_))
                | (ast::Type::NonNullList(_), _)
                | (_, ast::Type::NonNullList(_)) => return Err(mismatching_type_diagnostic()),
                // Now it's a named type.
                (type_a, type_b) => (type_a, type_b),
            };
        }

        // Now we are down to two named types, we can check that they have the same nullability...
        let (type_a, type_b) = match (type_a, type_b) {
            (ast::Type::NonNullNamed(a), ast::Type::NonNullNamed(b)) => (a, b),
            (ast::Type::Named(a), ast::Type::Named(b)) => (a, b),
            _ => return Err(mismatching_type_diagnostic()),
        };

        let (Some(def_a), Some(def_b)) = (schema.types.get(type_a), schema.types.get(type_b))
        else {
            return Ok(()); // Cannot do much if we don't know the type
        };

        match (def_a, def_b) {
            // 5. If typeA or typeB is Scalar or Enum.
            (
                def_a @ (schema::ExtendedType::Scalar(_) | schema::ExtendedType::Enum(_)),
                def_b @ (schema::ExtendedType::Scalar(_) | schema::ExtendedType::Enum(_)),
            ) => {
                // 5a. If typeA and typeB are the same type return true, otherwise return false.
                if def_a == def_b {
                    Ok(())
                } else {
                    Err(mismatching_type_diagnostic())
                }
            }
            // 6. If typeA or typeB is not a composite type, return false.
            (def_a, def_b) if is_composite(&def_a) && is_composite(&def_b) => Ok(()),
            _ => Err(mismatching_type_diagnostic()),
        }
    }

    /// Check if two field selections from the same type are the same, so the fields can be merged.
    fn same_field_selection(
        field_a: &FieldSelection,
        field_b: &FieldSelection,
    ) -> Result<(), ValidationError> {
        debug_assert_eq!(field_a.parent_type, field_b.parent_type);

        // 2bi. fieldA and fieldB must have identical field names.
        if field_a.field.name != field_b.field.name {
            return Err(ValidationError::new(
                field_b.field.location(),
                DiagnosticData::ConflictingFieldName {
                    field: field_a.field.response_key().clone(),
                    original_selection: field_a.field.location(),
                    original_name: field_a.field.name.clone(),
                    redefined_selection: field_b.field.location(),
                    redefined_name: field_b.field.name.clone(),
                },
            ));
        }

        // 2bii. fieldA and fieldB must have identical sets of arguments.
        let args_a = &field_a.field.arguments;
        let args_b = &field_b.field.arguments;

        let conflicting_field_argument =
            |original_arg: Option<&Node<ast::Argument>>,
             redefined_arg: Option<&Node<ast::Argument>>| {
                debug_assert!(
                    original_arg.is_some() || redefined_arg.is_some(),
                    "a conflicting field argument error can only exist when at least one field has the argument",
                );

                // We can take the name from either one of the arguments as they are necessarily the same.
                let arg = original_arg.or(redefined_arg).unwrap();

                let data = DiagnosticData::ConflictingFieldArgument {
                    // field_a and field_b have the same name so we can use either one.
                    field: field_b.field.name.clone(),
                    argument: arg.name.clone(),
                    original_selection: field_a.field.location(),
                    original_value: original_arg.map(|arg| (*arg.value).clone()),
                    redefined_selection: field_b.field.location(),
                    redefined_value: redefined_arg.map(|arg| (*arg.value).clone()),
                };
                ValidationError::new(field_b.field.location(), data)
            };

        // Check if fieldB provides the same argument names and values as fieldA (order-independent).
        for arg in args_a {
            let Some(other_arg) = args_b.iter().find(|other_arg| other_arg.name == arg.name) else {
                return Err(conflicting_field_argument(Some(arg), None));
            };

            if other_arg.value != arg.value {
                return Err(conflicting_field_argument(Some(arg), Some(other_arg)));
            }
        }
        // Check if fieldB provides any arguments that fieldA does not provide.
        for arg in args_b {
            if !args_a.iter().any(|other_arg| other_arg.name == arg.name) {
                return Err(conflicting_field_argument(None, Some(arg)));
            };
        }

        Ok(())
    }
}

pub(crate) fn validate_selection_set(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    selection_set: &[ast::Selection],
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    diagnostics.extend(validate_selections(
        db,
        file_id,
        against_type,
        selection_set,
        context,
    ));

    diagnostics
}

pub(crate) fn validate_selections(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    selection_set: &[ast::Selection],
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    for selection in selection_set {
        match selection {
            ast::Selection::Field(field) => diagnostics.extend(super::field::validate_field(
                db,
                file_id,
                against_type,
                field.clone(),
                context.clone(),
            )),
            ast::Selection::FragmentSpread(fragment) => {
                diagnostics.extend(super::fragment::validate_fragment_spread(
                    db,
                    file_id,
                    against_type,
                    fragment.clone(),
                    context.clone(),
                ))
            }
            ast::Selection::InlineFragment(inline) => {
                diagnostics.extend(super::fragment::validate_inline_fragment(
                    db,
                    file_id,
                    against_type,
                    inline.clone(),
                    context.clone(),
                ))
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_combinations_test() {
        let pairs = pair_combinations::<i64>(&[1, 2, 3, 4]).collect::<Vec<_>>();
        assert_eq!(
            pairs,
            &[(&1, &2), (&1, &3), (&1, &4), (&2, &3), (&2, &4), (&3, &4)]
        );
        let pairs = pair_combinations(&["a", "a", "a"]).collect::<Vec<_>>();
        assert_eq!(pairs, &[(&"a", &"a"), (&"a", &"a"), (&"a", &"a")]);
    }
}
