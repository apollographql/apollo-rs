use arbitrary::Result;

use crate::{
    field::Field,
    fragment::{FragmentSpread, InlineFragment},
    name::Name,
    DocumentBuilder,
};

/// The __selectionSet type represents a selection_set type in a fragment spread, an operation or a field
///
/// *SelectionSet*:
///     Selection*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Selection-Sets).
#[derive(Debug)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl From<SelectionSet> for apollo_encoder::SelectionSet {
    fn from(sel_set: SelectionSet) -> Self {
        let mut new_sel_set = Self::new();
        sel_set
            .selections
            .into_iter()
            .for_each(|selection| new_sel_set.selection(selection.into()));

        new_sel_set
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::SelectionSet> for SelectionSet {
    type Error = crate::FromError;

    fn try_from(selection_set: apollo_parser::ast::SelectionSet) -> Result<Self, Self::Error> {
        Ok(Self {
            selections: selection_set
                .selections()
                .map(Selection::try_from)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// The __selection type represents a selection in a selection set
/// *Selection*:
///     Field | FragmentSpread | InlineFragment
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#Selection).
#[derive(Debug)]
pub enum Selection {
    /// Represents a field
    Field(Field),
    /// Represents a fragment spread
    FragmentSpread(FragmentSpread),
    /// Represents an inline fragment
    InlineFragment(InlineFragment),
}

impl From<Selection> for apollo_encoder::Selection {
    fn from(selection: Selection) -> Self {
        match selection {
            Selection::Field(field) => Self::Field(field.into()),
            Selection::FragmentSpread(fragment_spread) => {
                Self::FragmentSpread(fragment_spread.into())
            }
            Selection::InlineFragment(inline_fragment) => {
                Self::InlineFragment(inline_fragment.into())
            }
        }
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Selection> for Selection {
    type Error = crate::FromError;

    fn try_from(selection: apollo_parser::ast::Selection) -> Result<Self, Self::Error> {
        match selection {
            apollo_parser::ast::Selection::Field(field) => field.try_into().map(Self::Field),
            apollo_parser::ast::Selection::FragmentSpread(fragment_spread) => {
                fragment_spread.try_into().map(Self::FragmentSpread)
            }
            apollo_parser::ast::Selection::InlineFragment(inline_fragment) => {
                inline_fragment.try_into().map(Self::InlineFragment)
            }
        }
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `SelectionSet`
    pub fn selection_set(&mut self) -> Result<SelectionSet> {
        let mut exclude_names = Vec::new();
        let selection_nb = std::cmp::max(
            self.stack.last().map(|o| o.fields_def().len()).unwrap_or(7),
            3,
        );
        let selections = (0..self.u.int_in_range(2..=selection_nb)?)
            .map(|index| self.selection(index, &mut exclude_names)) // TODO do not generate duplication variable name
            .collect::<Result<Vec<_>>>()?;
        Ok(SelectionSet { selections })
    }

    /// Create an arbitrary `Selection`
    pub fn selection(&mut self, index: usize, excludes: &mut Vec<Name>) -> Result<Selection> {
        let selection = match self.u.int_in_range(0..=2usize)? {
            0 => Selection::Field(self.field(index)?),
            1 => match self.fragment_spread(excludes)? {
                Some(frag_spread) => Selection::FragmentSpread(frag_spread),
                None => Selection::Field(self.field(index)?),
            },
            2 => Selection::InlineFragment(self.inline_fragment()?),
            _ => unreachable!(),
        };

        Ok(selection)
    }
}
