use crate::collections::HashMap;
use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::request::RequestError;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Name;

const MAX_LISTS_DEPTH: u32 = 3;

pub(super) fn check_selection_set<'doc>(
    document: &'doc Valid<ExecutableDocument>,
    fragment_depths: &mut HashMap<&'doc Name, u32>,
    depth_so_far: u32,
    selection_set: &'doc SelectionSet,
) -> Result<u32, RequestError> {
    let mut max_depth = depth_so_far;
    for selection in &selection_set.selections {
        match selection {
            Selection::InlineFragment(inline) => {
                max_depth = max_depth.max(check_selection_set(
                    document,
                    fragment_depths,
                    depth_so_far,
                    &inline.selection_set,
                )?)
            }
            Selection::FragmentSpread(spread) => {
                let Some(def) = document.fragments.get(&spread.fragment_name) else {
                    continue;
                };
                // Avoiding the entry API because we may have to modify the map in-between this `.get()`
                // and the `.insert()`.
                if let Some(fragment_depth) = fragment_depths.get(&spread.fragment_name) {
                    if depth_so_far + *fragment_depth > MAX_LISTS_DEPTH {
                        return Err(RequestError {
                            message: "Maximum introspection depth exceeded".into(),
                            location: spread.location(),
                            is_suspected_validation_bug: false,
                        });
                    }
                } else {
                    // Recursing without marking our fragment spread as used is fine,
                    // because validation guarantees that we do not have a self-referential
                    // fragment chain.
                    let post_fragment_depth = check_selection_set(
                        document,
                        fragment_depths,
                        depth_so_far,
                        &def.selection_set,
                    )?;
                    fragment_depths
                        .insert(&spread.fragment_name, post_fragment_depth - depth_so_far);
                    max_depth = max_depth.max(post_fragment_depth);
                }
            }
            Selection::Field(field) => {
                let mut depth = depth_so_far;
                if matches!(
                    field.name.as_str(),
                    "fields" | "interfaces" | "possibleTypes" | "inputFields"
                ) {
                    depth += 1;
                    if depth >= MAX_LISTS_DEPTH {
                        return Err(RequestError {
                            message: "Maximum introspection depth exceeded".into(),
                            location: field.name.location(),
                            is_suspected_validation_bug: false,
                        });
                    }
                }
                max_depth = max_depth.max(check_selection_set(
                    document,
                    fragment_depths,
                    depth,
                    &field.selection_set,
                )?)
            }
        }
    }
    Ok(max_depth)
}
