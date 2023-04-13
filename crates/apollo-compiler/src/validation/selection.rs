use indexmap::IndexMap;
use std::sync::Arc;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationDatabase,
};

/// Check if two fields will output the same type.
///
/// Spec: https://spec.graphql.org/October2021/#SameResponseShape()
pub(crate) fn same_response_shape(
    db: &dyn ValidationDatabase,
    field_a: Arc<hir::Field>,
    field_b: Arc<hir::Field>,
) -> Result<(), ApolloDiagnostic> {
    // 1. Let typeA be the return type of fieldA.
    let Some(full_type_a) = field_a.ty(db.upcast()) else {
        return Ok(()); // Can't do much if we don't know the type
    };
    // 2. Let typeB be the return type of fieldB.
    let Some(full_type_b) = field_b.ty(db.upcast()) else {
        return Ok(()); // Can't do much if we don't know the type
    };

    let mut type_a = &full_type_a;
    let mut type_b = &full_type_b;

    let mismatching_type_diagnostic = || {
        ApolloDiagnostic::new(
            db,
            field_b.loc().into(),
            DiagnosticData::ConflictingField {
                field: field_a.name().to_string(),
                original_selection: field_a.loc().into(),
                redefined_selection: field_b.loc().into(),
            },
        )
        .label(Label::new(
            field_a.loc(),
            format!(
                "`{}` has type `{}` here",
                field_a.response_name(),
                full_type_a
            ),
        ))
        .label(Label::new(
            field_b.loc(),
            format!("but the same field name has type `{full_type_b}` here"),
        ))
    };

    while !type_a.is_named() || !type_b.is_named() {
        // 3. If typeA or typeB is Non-Null.
        // 3a. If typeA or typeB is nullable, return false.
        // 3b. Let typeA be the nullable type of typeA
        // 3c. Let typeB be the nullable type of typeB
        (type_a, type_b) = match (type_a, type_b) {
            (hir::Type::NonNull { ty: type_a, .. }, hir::Type::NonNull { ty: type_b, .. }) => {
                (type_a.as_ref(), type_b.as_ref())
            }
            (hir::Type::NonNull { .. }, _) | (_, hir::Type::NonNull { .. }) => {
                return Err(mismatching_type_diagnostic())
            }
            (type_a, type_b) => (type_a, type_b),
        };

        // 4. If typeA or typeB is List.
        // 4a. If typeA or typeB is not List, return false.
        // 4b. Let typeA be the item type of typeA
        // 4c. Let typeB be the item type of typeB
        (type_a, type_b) = match (type_a, type_b) {
            (hir::Type::List { ty: type_a, .. }, hir::Type::List { ty: type_b, .. }) => {
                (type_a.as_ref(), type_b.as_ref())
            }
            (hir::Type::List { .. }, _) | (_, hir::Type::List { .. }) => {
                return Err(mismatching_type_diagnostic())
            }
            (type_a, type_b) => (type_a, type_b),
        };

        // 4d. Repeat from step 3.
    }

    let (Some(def_a), Some(def_b)) = (type_a.type_def(db.upcast()), type_b.type_def(db.upcast())) else {
        return Ok(()); // Cannot do much if we don't know the type
    };

    match (def_a, def_b) {
        // 5. If typeA or typeB is Scalar or Enum.
        (
            def_a @ (hir::TypeDefinition::ScalarTypeDefinition(_)
            | hir::TypeDefinition::EnumTypeDefinition(_)),
            def_b @ (hir::TypeDefinition::ScalarTypeDefinition(_)
            | hir::TypeDefinition::EnumTypeDefinition(_)),
        ) => {
            // 5a. If typeA and typeB are the same type return true, otherwise return false.
            if def_a == def_b {
                Ok(())
            } else {
                Err(mismatching_type_diagnostic())
            }
        }
        // 6. Assert: typeA and typeB are both composite types.
        (def_a, def_b) if def_a.is_composite_definition() && def_b.is_composite_definition() => {
            let merged_set = field_a.selection_set().merge(field_b.selection_set());
            let fields = db.flattened_operation_fields(merged_set);
            let grouped_by_name = group_fields_by_name(fields);

            for (_, fields_for_name) in grouped_by_name {
                // 9. Given each pair of members subfieldA and subfieldB in fieldsForName:
                let Some((subfield_a, rest)) = fields_for_name.split_first() else {
                    continue;
                };
                for subfield_b in rest {
                    // 9a. If SameResponseShape(subfieldA, subfieldB) is false, return false.
                    db.same_response_shape(Arc::clone(subfield_a), Arc::clone(subfield_b))?;
                }
            }

            Ok(())
        }
        (_, _) => Ok(()),
    }
}

/// Given a list of fields, group them by response name.
fn group_fields_by_name(fields: Vec<Arc<hir::Field>>) -> IndexMap<String, Vec<Arc<hir::Field>>> {
    let mut map = IndexMap::<String, Vec<Arc<hir::Field>>>::new();
    for field in fields {
        match map.entry(field.response_name().to_string()) {
            indexmap::map::Entry::Occupied(mut entry) => {
                entry.get_mut().push(field);
            }
            indexmap::map::Entry::Vacant(entry) => {
                entry.insert(vec![field]);
            }
        }
    }
    map
}

/// Check if the arguments provided to two fields are the same, so the fields can be merged.
fn identical_arguments(
    db: &dyn ValidationDatabase,
    field_a: &hir::Field,
    field_b: &hir::Field,
) -> Result<(), ApolloDiagnostic> {
    let args_a = field_a.arguments();
    let args_b = field_b.arguments();
    // Check if fieldB provides the same argument names and values as fieldA (order-independent).
    for arg in args_a {
        let Some(other_arg) = args_b.iter().find(|other_arg| other_arg.name() == arg.name()) else {
            return Err(
                ApolloDiagnostic::new(
                    db,
                    field_b.loc().into(),
                    DiagnosticData::ConflictingField {
                        field: field_a.name().to_string(),
                        original_selection: field_a.loc().into(),
                        redefined_selection: field_b.loc().into(),
                    },
                )
                .label(Label::new(arg.loc(), format!("field `{}` is selected with argument `{}` here", field_a.name(), arg.name())))
                .label(Label::new(field_b.loc(), format!("but argument `{}` is not provided here", arg.name())))
                .help("Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.")
            );
        };

        if !other_arg.value.is_same_value(&arg.value) {
            return Err(
                ApolloDiagnostic::new(
                    db,
                    field_b.loc().into(),
                    DiagnosticData::ConflictingField {
                        field: field_a.name().to_string(),
                        original_selection: field_a.loc().into(),
                        redefined_selection: field_b.loc().into(),
                    },
                )
                .label(Label::new(arg.loc(), format!("field `{}` provides one argument value here", field_a.name())))
                .label(Label::new(other_arg.loc(), "but a different value here"))
                .help("Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.")
            );
        }
    }
    // Check if fieldB provides any arguments that fieldA does not provide.
    for arg in args_b {
        if !args_a
            .iter()
            .any(|other_arg| other_arg.name() == arg.name())
        {
            return Err(
                ApolloDiagnostic::new(
                    db,
                    field_b.loc().into(),
                    DiagnosticData::ConflictingField {
                        field: field_a.name().to_string(),
                        original_selection: field_a.loc().into(),
                        redefined_selection: field_b.loc().into(),
                    },
                )
                .label(Label::new(arg.loc(), format!("field `{}` is selected with argument `{}` here", field_b.name(), arg.name())))
                .label(Label::new(field_a.loc(), format!("but argument `{}` is not provided here", arg.name())))
                .help("Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.")
            );
        };
    }

    Ok(())
}

/// Check if the fields in a given selection set can be merged.
///
/// Spec: https://spec.graphql.org/October2021/#FieldsInSetCanMerge()
pub(crate) fn fields_in_set_can_merge(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Result<(), Vec<ApolloDiagnostic>> {
    // 1. Let `fieldsForName` be the set of selections with a given response name in set including visiting fragments and inline fragments.
    let fields = db.flattened_operation_fields(selection_set);
    let grouped_by_name = group_fields_by_name(fields);

    let mut diagnostics = vec![];

    for (_, fields_for_name) in grouped_by_name {
        let Some((field_a, rest)) = fields_for_name.split_first() else {
            continue; // Nothing to merge
        };
        let Some(parent_a) = field_a.parent_type(db.upcast()) else {
            continue; // We can't find the type
        };

        // 2. Given each pair of members fieldA and fieldB in fieldsForName:
        for field_b in rest {
            // 2a. SameResponseShape(fieldA, fieldB) must be true.
            if let Err(diagnostic) =
                db.same_response_shape(Arc::clone(field_a), Arc::clone(field_b))
            {
                diagnostics.push(diagnostic);
                continue;
            }
            // 2b. If the parent types of fieldA and fieldB are equal or if either is not an Object Type:
            let Some(parent_b) = field_b.parent_type(db.upcast()) else {
                continue; // We can't find the type
            };
            if parent_a == parent_b {
                // 2bi. fieldA and fieldB must have identical field names.
                if field_a.name() != field_b.name() {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            field_b.loc().into(),
                            DiagnosticData::ConflictingField {
                                field: field_b.name().to_string(),
                                original_selection: field_a.loc().into(),
                                redefined_selection: field_b.loc().into(),
                            },
                        )
                        .label(Label::new(
                            field_a.loc(),
                            format!(
                                "field `{}` is selected from field `{}` here",
                                field_a.response_name(),
                                field_a.name()
                            ),
                        ))
                        .label(Label::new(
                            field_b.loc(),
                            format!(
                                "but the same field `{}` is also selected from field `{}` here",
                                field_b.response_name(),
                                field_b.name()
                            ),
                        ))
                        .help("Alias is already used for a different field"),
                    );
                    continue;
                }
                // 2bii. fieldA and fieldB must have identical sets of arguments.
                if let Err(diagnostic) = identical_arguments(db, field_a, field_b) {
                    diagnostics.push(diagnostic);
                    continue;
                }
                // 2biii. Let mergedSet be the result of adding the selection set of fieldA and the selection set of fieldB.
                let merged_set = field_a.selection_set().merge(field_b.selection_set());
                // 2biv. FieldsInSetCanMerge(mergedSet) must be true.
                if let Err(sub_diagnostics) = db.fields_in_set_can_merge(merged_set) {
                    diagnostics.extend(sub_diagnostics);
                    continue;
                }
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics)
    }
}

pub fn validate_selection(
    db: &dyn ValidationDatabase,
    selection: Arc<Vec<hir::Selection>>,
    parent_op: Option<hir::Name>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection.iter() {
        match sel {
            hir::Selection::Field(field) => {
                diagnostics.extend(db.validate_field(field.clone(), parent_op.clone()));
            }
            // TODO handle fragment spreads on invalid parent types
            hir::Selection::FragmentSpread(frag) => {
                diagnostics
                    .extend(db.validate_fragment_spread(Arc::clone(frag), parent_op.clone()));
                diagnostics.extend(db.validate_directives(
                    frag.directives().to_vec(),
                    hir::DirectiveLocation::FragmentSpread,
                    parent_op.clone(),
                ));
            }
            hir::Selection::InlineFragment(inline) => {
                diagnostics.extend(db.validate_directives(
                    inline.directives().to_vec(),
                    hir::DirectiveLocation::InlineFragment,
                    parent_op.clone(),
                ));
                diagnostics.extend(db.validate_fragment_type_condition(
                    inline.type_condition().map(|t| t.to_string()),
                    inline.loc(),
                ));
            }
        }
    }

    diagnostics
}

pub fn validate_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
    parent_op: Option<hir::Name>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    if let Err(diagnostic) = db.fields_in_set_can_merge(selection_set.clone()) {
        diagnostics.extend(diagnostic);
    }

    diagnostics.extend(db.validate_selection(selection_set.selection, parent_op));

    diagnostics
}
