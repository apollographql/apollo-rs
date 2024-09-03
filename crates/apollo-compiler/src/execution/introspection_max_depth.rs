use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::execution::introspection_split::get_fragment;
use crate::execution::SchemaIntrospectionError;
use crate::validation::Valid;
use crate::ExecutableDocument;

const MAX_LISTS_DEPTH: u32 = 3;

pub(crate) fn check_document(
    document: &Valid<ExecutableDocument>,
) -> Result<(), SchemaIntrospectionError> {
    for operation in document.operations.iter() {
        let initial_depth = 0;
        check_selection_set(document, initial_depth, &operation.selection_set)?;
    }
    Ok(())
}

fn check_selection_set(
    document: &Valid<ExecutableDocument>,
    depth_so_far: u32,
    selection_set: &SelectionSet,
) -> Result<(), SchemaIntrospectionError> {
    for selection in &selection_set.selections {
        match selection {
            Selection::InlineFragment(inline) => {
                check_selection_set(document, depth_so_far, &inline.selection_set)?
            }
            Selection::FragmentSpread(spread) => {
                // Validation ensures that `Valid<ExecutableDocument>` does not contain fragment cycles
                let def = get_fragment(document, &spread.fragment_name)?;
                check_selection_set(document, depth_so_far, &def.selection_set)?
            }
            Selection::Field(field) => {
                let mut depth = depth_so_far;
                if matches!(
                    field.name.as_str(),
                    "fields" | "interfaces" | "possibleTypes" | "inputFields"
                ) {
                    depth += 1;
                    if depth >= MAX_LISTS_DEPTH {
                        return Err(SchemaIntrospectionError::DeeplyNestedIntrospectionList(
                            field.name.location(),
                        ));
                    }
                }
                check_selection_set(document, depth, &field.selection_set)?
            }
        }
    }
    Ok(())
}
