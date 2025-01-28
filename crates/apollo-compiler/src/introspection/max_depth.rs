use crate::collections::HashSet;
use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::request::RequestError;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Name;

const MAX_LISTS_DEPTH: u32 = 3;

pub(super) fn check_selection_set<'doc>(
    document: &'doc Valid<ExecutableDocument>,
    fragments_visited: &mut HashSet<&'doc Name>,
    depth_so_far: u32,
    selection_set: &'doc SelectionSet,
) -> Result<(), RequestError> {
    for selection in &selection_set.selections {
        match selection {
            Selection::InlineFragment(inline) => check_selection_set(
                document,
                fragments_visited,
                depth_so_far,
                &inline.selection_set,
            )?,
            Selection::FragmentSpread(spread) => {
                if let Some(def) = document.fragments.get(&spread.fragment_name) {
                    let new = fragments_visited.insert(&spread.fragment_name);
                    if new {
                        check_selection_set(
                            document,
                            fragments_visited,
                            depth_so_far,
                            &def.selection_set,
                        )?
                    }
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
                check_selection_set(document, fragments_visited, depth, &field.selection_set)?
            }
        }
    }
    Ok(())
}
