use std::sync::Arc;
use multimap::MultiMap;

use crate::{
    hir,
    validation::ValidationDatabase,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
};

fn same_response_shape(db: &dyn ValidationDatabase, field_a: &hir::Field, field_b: &hir::Field) -> bool {
    // 1. Let typeA be the return type of fieldA.
    let Some(mut type_a) = field_a.ty(db.upcast()) else {
        return false;
    };
    // 2. Let typeB be the return type of fieldB.
    let Some(mut type_b) = field_b.ty(db.upcast()) else {
        return false;
    };

    while !type_a.is_named() || !type_b.is_named() {
        // 3. If typeA or typeB is Non-Null.
        // 3a. If typeA or typeB is nullable, return false.
        // 3b. Let typeA be the nullable type of typeA
        // 3c. Let typeB be the nullable type of typeB
        (type_a, type_b) = match (type_a, type_b) {
            (hir::Type::NonNull { ty: type_a, .. }, hir::Type::NonNull { ty: type_b, .. }) => (*type_a, *type_b),
            (hir::Type::NonNull { .. }, _) | (_, hir::Type::NonNull { .. }) => return false,
            (type_a, type_b) => (type_a, type_b),
        };

        // 4. If typeA or typeB is List.
        // 4a. If typeA or typeB is not List, return false.
        // 4b. Let typeA be the item type of typeA
        // 4c. Let typeB be the item type of typeB
        // 4d. Repeat from step 3.
        (type_a, type_b) = match (type_a, type_b) {
            (hir::Type::List { ty: type_a, .. }, hir::Type::List { ty: type_b, .. }) => (*type_a, *type_b),
            (hir::Type::List { .. }, _) | (_, hir::Type::List { .. }) => return false,
            (type_a, type_b) => (type_a, type_b),
        };
    }

    let (Some(def_a), Some(def_b)) = (type_a.type_def(db.upcast()), type_b.type_def(db.upcast())) else {
        return false;
    };

    match (def_a, def_b) {
        // 5. If typeA or typeB is Scalar or Enum.
        (
            def_a @ (hir::TypeDefinition::ScalarTypeDefinition(_) | hir::TypeDefinition::EnumTypeDefinition(_)),
            def_b @ (hir::TypeDefinition::ScalarTypeDefinition(_) | hir::TypeDefinition::EnumTypeDefinition(_)),
        ) => {
            // 5a. If typeA and typeB are the same type return true, otherwise return false.
            return def_a == def_b;
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
                    if !same_response_shape(db, subfield_a, subfield_b) {
                        return false;
                    }
                }
            }
        }
        (_, _) => return false,
    }

    true
}

fn group_fields_by_name(fields: Vec<Arc<hir::Field>>) -> MultiMap<String, Arc<hir::Field>> {
    fields
        .into_iter()
        .map(|field| (field.response_name().to_string(), field))
        .collect()
}

fn identical_arguments(field_a: &hir::Field, field_b: &hir::Field) -> bool {
    // TODO(@goto-bus-stop) This should probably "resolve" arguments? eg. fill in default values…
    let args_a = field_a.arguments();
    let args_b = field_b.arguments();
    // Check if fieldB provides the same argument names and values as fieldA (order-independent).
    for arg in args_a {
        let Some(other_arg) = args_b.iter().find(|other_arg| other_arg.name() == arg.name()) else {
            return false;
        };

        if other_arg.value != arg.value {
            return false;
        }
    }
    // Check if fieldB provides any arguments that fieldA does not provide.
    for arg in args_b {
        if !args_a.iter().any(|other_arg| other_arg.name() == arg.name()) {
            return false;
        };
    }

    return true;
}

fn fields_in_set_can_merge(db: &dyn ValidationDatabase, selection_set: &hir::SelectionSet) -> bool {
    // 1. Let `fieldsForName` be the set of selections with a given response name in set including visiting fragments and inline fragments.
    let fields = db.flattened_operation_fields(selection_set.clone());
    let grouped_by_name = group_fields_by_name(fields);

    for (_, fields_for_name) in grouped_by_name {
        // 2. Given each pair of members fieldA and fieldB in fieldsForName:
        let Some((field_a, rest)) = fields_for_name.split_first() else {
            continue;
        };
        let Some(parent_a) = field_a.parent_type(db.upcast()) else {
            return false;
        };
        for field_b in rest {
            // 2a. SameResponseShape(fieldA, fieldB) must be true.
            if !same_response_shape(db, field_a, field_b) {
                return false;
            }
            // 2b. If the parent types of fieldA and fieldB are equal or if either is not an Object Type:
            let Some(parent_b) = field_b.parent_type(db.upcast()) else {
                return false;
            };
            if parent_a == parent_b {
                // 2bi. fieldA and fieldB must have identical field names.
                if field_a.name() != field_b.name() {
                    return false;
                }
                // 2bii. fieldA and fieldB must have identical sets of arguments.
                if !identical_arguments(field_a, field_b) {
                    return false;
                }
                // 2biii. Let mergedSet be the result of adding the selection set of fieldA and the selection set of fieldB.
                let merged_set = field_a.selection_set().merge(field_b.selection_set());
                // 2biv. FieldsInSetCanMerge(mergedSet) must be true.
                if !fields_in_set_can_merge(db, &merged_set) {
                    return false;
                }
            }
        }
    }

    return true;
}

pub fn validate_selection(
    db: &dyn ValidationDatabase,
    selection: Arc<Vec<hir::Selection>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection.iter() {
        match sel {
            hir::Selection::Field(field) => {
                diagnostics.extend(db.validate_field(field.clone()));
            }

            // TODO handle fragment spreads on invalid parent types
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(db.validate_directives(
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => diagnostics.extend(db.validate_directives(
                inline.directives().to_vec(),
                hir::DirectiveLocation::InlineFragment,
            )),
        }
    }

    diagnostics
}

pub fn validate_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // TODO handle Enums, Scalar (no selection set content allowed), Unions

    if !fields_in_set_can_merge(db, &selection_set) {
        diagnostics.push(ApolloDiagnostic::new(db, selection_set.selection[0].loc().into(), DiagnosticData::SyntaxError {
            message: "Cannot merge".to_string(),
        }).label(Label::new(selection_set.selection[0].loc(), "the error is not actually here")));
    }

    diagnostics.extend(db.validate_selection(selection_set.selection));

    diagnostics
}
