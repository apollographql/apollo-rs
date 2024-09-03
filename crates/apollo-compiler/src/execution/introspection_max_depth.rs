use crate::executable::Operation;
use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::parser::SourceSpan;
use crate::validation::Valid;
use crate::ExecutableDocument;

const MAX_LISTS_DEPTH: u32 = 3;

#[derive(Debug)]
pub struct DeeplyNestedIntrospectionListError {
    pub location: Option<SourceSpan>,
}

pub fn check_introspection_max_depth(
    document: &Valid<ExecutableDocument>,
    operation: &Operation,
) -> Result<(), DeeplyNestedIntrospectionListError> {
    let initial_depth = 0;
    check_selection_set(document, initial_depth, &operation.selection_set)
}

fn check_selection_set(
    document: &Valid<ExecutableDocument>,
    depth_so_far: u32,
    selection_set: &SelectionSet,
) -> Result<(), DeeplyNestedIntrospectionListError> {
    for selection in &selection_set.selections {
        match selection {
            Selection::InlineFragment(inline) => {
                check_selection_set(document, depth_so_far, &inline.selection_set)?
            }
            Selection::FragmentSpread(spread) => {
                // Validation ensures that `Valid<ExecutableDocument>` does not contain fragment cycles
                if let Some(def) = document.fragments.get(&spread.fragment_name) {
                    check_selection_set(document, depth_so_far, &def.selection_set)?
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
                        return Err(DeeplyNestedIntrospectionListError {
                            location: field.name.location(),
                        });
                    }
                }
                check_selection_set(document, depth, &field.selection_set)?
            }
        }
    }
    Ok(())
}
