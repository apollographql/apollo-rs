use std::fmt::{self, Write as _};

use crate::{Field, FragmentSpread, InlineFragment};

/// The SelectionSet type represents a selection_set type in a fragment spread,
/// an operation or a field
///
/// *SelectionSet*:
///     Selection*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Selection-Sets).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Field, FragmentSpread, Selection, SelectionSet, TypeCondition};
/// use indoc::indoc;
///
/// let selections = vec![
///     Selection::Field(Field::new(String::from("myField"))),
///     Selection::FragmentSpread(FragmentSpread::new(String::from("myFragment"))),
/// ];
/// let mut selection_set = SelectionSet::new();
/// selections
///     .into_iter()
///     .for_each(|s| selection_set.selection(s));
///
/// assert_eq!(
///     selection_set.to_string(),
///     indoc! {r#"
///         {
///           myField
///           ...myFragment
///         }
///     "#}
/// )
/// ```
#[derive(Debug, PartialEq, Clone, Default)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl SelectionSet {
    /// Create an instance of SelectionSet
    pub fn new() -> Self {
        Self::default()
    }

    /// Create an instance of SelectionSet given its selections
    pub fn with_selections(selections: Vec<Selection>) -> Self {
        Self { selections }
    }

    /// Add a selection in the SelectionSet
    pub fn selection(&mut self, selection: Selection) {
        self.selections.push(selection);
    }

    /// Should be used everywhere in this crate isntead of the Display implementation
    /// Display implementation is only useful as a public api
    pub(crate) fn format_with_indent(&self, mut indent_level: usize) -> String {
        let mut text = String::from("{\n");
        indent_level += 1;
        let indent = "  ".repeat(indent_level);
        for sel in &self.selections {
            let _ = writeln!(text, "{}{}", indent, sel.format_with_indent(indent_level));
        }
        if indent_level <= 1 {
            text.push_str("}\n");
        } else {
            let _ = write!(text, "{}}}", "  ".repeat(indent_level - 1));
        }

        text
    }
}

// This impl is only useful when we generate only a SelectionSet
// If it's used from a parent element, we call `format_with_indent`
impl fmt::Display for SelectionSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 1;
        writeln!(f, "{{")?;
        let indent = "  ".repeat(indent_level);
        for sel in &self.selections {
            writeln!(f, "{}{}", indent, sel.format_with_indent(indent_level))?;
        }
        writeln!(f, "}}")?;

        Ok(())
    }
}

/// The Selection type represents a selection in a selection set
/// *Selection*:
///     Field | FragmentSpread | InlineFragment
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#Selection).
#[derive(Debug, Clone, PartialEq)]
pub enum Selection {
    /// Represents a field
    Field(Field),
    /// Represents a fragment spread
    FragmentSpread(FragmentSpread),
    /// Represents an inline fragment
    InlineFragment(InlineFragment),
}

impl Selection {
    pub(crate) fn format_with_indent(&self, indent_level: usize) -> String {
        match self {
            Selection::Field(field) => field.format_with_indent(indent_level),
            Selection::FragmentSpread(frag_spread) => frag_spread.to_string(),
            Selection::InlineFragment(inline_frag) => inline_frag.format_with_indent(indent_level),
        }
    }
}

impl fmt::Display for Selection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 0;
        match self {
            Selection::Field(field) => write!(f, "{}", field.format_with_indent(indent_level)),
            Selection::FragmentSpread(fragment_spread) => write!(f, "{fragment_spread}"),
            Selection::InlineFragment(inline_fragment) => {
                write!(f, "{}", inline_fragment.format_with_indent(indent_level))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

    #[test]
    fn it_encodes_selection_set() {
        let mut aliased_field = Field::new(String::from("myField"));
        aliased_field.alias(Some(String::from("myAlias")));
        let selections = vec![
            Selection::Field(aliased_field),
            Selection::FragmentSpread(FragmentSpread::new(String::from("myFragment"))),
        ];
        let mut selection_set = SelectionSet::new();
        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));

        assert_eq!(
            selection_set.to_string(),
            indoc! {r#"
                {
                  myAlias: myField
                  ...myFragment
                }
            "#}
        )
    }

    #[test]
    fn it_encodes_deeper_selection_set() {
        let fourth_field = Field::new("fourth".to_string());
        let mut third_field = Field::new("third".to_string());
        third_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            fourth_field,
        )])));
        let mut second_field = Field::new("second".to_string());
        second_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            third_field,
        )])));

        let mut first_field = Field::new("first".to_string());
        first_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            second_field,
        )])));

        let selections = vec![
            Selection::Field(first_field),
            Selection::FragmentSpread(FragmentSpread::new(String::from("myFragment"))),
        ];
        let mut selection_set = SelectionSet::new();

        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));

        assert_eq!(
            selection_set.to_string(),
            indoc! {r#"
                {
                  first {
                    second {
                      third {
                        fourth
                      }
                    }
                  }
                  ...myFragment
                }
            "#}
        )
    }
}
