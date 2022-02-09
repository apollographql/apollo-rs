use arbitrary::Result;

use crate::{
    field::Field,
    fragment::{FragmentSpread, InlineFragment},
    name::Name,
    DocumentBuilder,
};

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

#[derive(Debug)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `SelectionSet`
    pub fn selection_set(&mut self) -> Result<SelectionSet> {
        let mut exclude_names = Vec::new();
        let selections = (0..self.u.int_in_range(2..=7usize)?)
            .map(|i| self.selection(i, &mut exclude_names)) // TODO do not generate duplication variable name
            .collect::<Result<Vec<_>>>()?;
        Ok(SelectionSet { selections })
    }
    /// Create an arbitrary `Selection`
    pub fn selection(&mut self, index: usize, excludes: &mut Vec<Name>) -> Result<Selection> {
        let selection = match self.u.int_in_range(0..=2usize)? {
            0 => Selection::Field(self.field_with_index(index)?),
            1 => match self.fragment_spread(excludes)? {
                Some(frag_spread) => Selection::FragmentSpread(frag_spread),
                None => Selection::Field(self.field_with_index(index)?),
            },
            2 => Selection::InlineFragment(self.inline_fragment()?),
            _ => unreachable!(),
        };

        Ok(selection)
    }
}
