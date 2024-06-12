use crate::{
    field::Field,
    fragment::{FragmentSpread, InlineFragment},
    name::Name,
    DocumentBuilder,
};
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;

/// The __selectionSet type represents a selection_set type in a fragment spread, an operation or a field
///
/// *SelectionSet*:
///     Selection*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Selection-Sets).
#[derive(Debug, Clone)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

impl From<SelectionSet> for Vec<ast::Selection> {
    fn from(sel_set: SelectionSet) -> Self {
        sel_set.selections.into_iter().map(Into::into).collect()
    }
}

impl TryFrom<apollo_parser::cst::SelectionSet> for SelectionSet {
    type Error = crate::FromError;

    fn try_from(selection_set: apollo_parser::cst::SelectionSet) -> Result<Self, Self::Error> {
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
#[derive(Debug, Clone)]
pub enum Selection {
    /// Represents a field
    Field(Field),
    /// Represents a fragment spread
    FragmentSpread(FragmentSpread),
    /// Represents an inline fragment
    InlineFragment(InlineFragment),
}

impl From<Selection> for ast::Selection {
    fn from(selection: Selection) -> Self {
        match selection {
            Selection::Field(field) => Self::Field(Node::new(field.into())),
            Selection::FragmentSpread(fragment_spread) => {
                Self::FragmentSpread(Node::new(fragment_spread.into()))
            }
            Selection::InlineFragment(inline_fragment) => {
                Self::InlineFragment(Node::new(inline_fragment.into()))
            }
        }
    }
}

impl TryFrom<apollo_parser::cst::Selection> for Selection {
    type Error = crate::FromError;

    fn try_from(selection: apollo_parser::cst::Selection) -> Result<Self, Self::Error> {
        match selection {
            apollo_parser::cst::Selection::Field(field) => field.try_into().map(Self::Field),
            apollo_parser::cst::Selection::FragmentSpread(fragment_spread) => {
                fragment_spread.try_into().map(Self::FragmentSpread)
            }
            apollo_parser::cst::Selection::InlineFragment(inline_fragment) => {
                inline_fragment.try_into().map(Self::InlineFragment)
            }
        }
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `SelectionSet`
    pub fn selection_set(&mut self) -> ArbitraryResult<SelectionSet> {
        let mut exclude_names = Vec::new();
        let selection_nb = self.stack.last().map(|o| o.fields_def().len()).unwrap_or(0);

        let selections = (0..self.u.int_in_range(1..=5)?)
            .map(|_| {
                let index = self.u.int_in_range(0..=selection_nb)?;
                self.selection(index, &mut exclude_names)
            }) // TODO do not generate duplication variable name
            .collect::<ArbitraryResult<Vec<_>>>()?;
        Ok(SelectionSet { selections })
    }

    /// Create an arbitrary `Selection`
    pub fn selection(
        &mut self,
        index: usize,
        excludes: &mut Vec<Name>,
    ) -> ArbitraryResult<Selection> {
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
