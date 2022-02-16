use arbitrary::Result;

use crate::{
    directive::Directive, name::Name, selection_set::SelectionSet, ty::Ty, DocumentBuilder,
};

/// The __fragmentDef type represents a fragment definition
///
/// *FragmentDefinition*:
///     fragment FragmentName TypeCondition Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentDefinition).
#[derive(Debug)]
pub struct FragmentDef {
    pub(crate) name: Name,
    pub(crate) type_condition: TypeCondition,
    pub(crate) directives: Vec<Directive>,
    pub(crate) selection_set: SelectionSet,
}

impl From<FragmentDef> for apollo_encoder::FragmentDefinition {
    fn from(frag_def: FragmentDef) -> Self {
        let mut new_frag_def = apollo_encoder::FragmentDefinition::new(
            frag_def.name.into(),
            frag_def.type_condition.into(),
            frag_def.selection_set.into(),
        );
        frag_def
            .directives
            .into_iter()
            .for_each(|directive| new_frag_def.directive(directive.into()));

        new_frag_def
    }
}

/// The __fragmentSpread type represents a named fragment used in a selection set.
///
/// *FragmentSpread*:
///     ... FragmentName Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentSpread).
#[derive(Debug)]
pub struct FragmentSpread {
    pub(crate) name: Name,
    pub(crate) directives: Vec<Directive>,
}

impl From<FragmentSpread> for apollo_encoder::FragmentSpread {
    fn from(fragment_spread: FragmentSpread) -> Self {
        let mut new_fragment_spread = Self::new(fragment_spread.name.into());
        fragment_spread
            .directives
            .into_iter()
            .for_each(|directive| new_fragment_spread.directive(directive.into()));

        new_fragment_spread
    }
}

/// The __inlineFragment type represents an inline fragment in a selection set that could be used as a field
///
/// *InlineFragment*:
///     ... TypeCondition? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Inline-Fragments).
#[derive(Debug)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<TypeCondition>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) selection_set: SelectionSet,
}

impl From<InlineFragment> for apollo_encoder::InlineFragment {
    fn from(inline_fragment: InlineFragment) -> Self {
        let mut new_inline_fragment = Self::new(inline_fragment.selection_set.into());
        new_inline_fragment.type_condition(inline_fragment.type_condition.map(Into::into));
        inline_fragment
            .directives
            .into_iter()
            .for_each(|directive| new_inline_fragment.directive(directive.into()));

        new_inline_fragment
    }
}

/// The __typeCondition type represents where a fragment could be applied
///
/// *TypeCondition*:
///     on NamedType
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#TypeCondition).
#[derive(Debug)]
pub struct TypeCondition {
    name: Name,
}

impl From<TypeCondition> for apollo_encoder::TypeCondition {
    fn from(ty_cond: TypeCondition) -> Self {
        Self::new(ty_cond.name.into())
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `FragmentDef`
    pub fn fragment_definition(&mut self) -> Result<FragmentDef> {
        let name = self.type_name()?;
        let directives = self.directives()?;
        let selection_set = self.selection_set()?;
        let type_condition = self.type_condition()?;

        Ok(FragmentDef {
            name,
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Create an arbitrary `FragmentSpread`, returns `None` if no fragment definition was previously created
    pub fn fragment_spread(&mut self, excludes: &mut Vec<Name>) -> Result<Option<FragmentSpread>> {
        let available_fragment: Vec<&FragmentDef> = self
            .fragment_defs
            .iter()
            .filter(|f| excludes.contains(&f.name))
            .collect();

        let name = if !available_fragment.is_empty() {
            return Ok(None);
        } else {
            self.u.choose(&available_fragment)?.name.clone()
        };
        let directives = self.directives()?;
        excludes.push(name.clone());

        Ok(Some(FragmentSpread { name, directives }))
    }

    /// Create an arbitrary `InlineFragment`
    pub fn inline_fragment(&mut self) -> Result<InlineFragment> {
        let type_condition = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.type_condition())
            .transpose()?;
        let selection_set = self.selection_set()?;
        let directives = self.directives()?;

        Ok(InlineFragment {
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Create an arbitrary `TypeCondition`
    pub fn type_condition(&mut self) -> Result<TypeCondition> {
        let named_types: Vec<Ty> = self
            .list_existing_object_types()
            .into_iter()
            .filter(Ty::is_named)
            .collect();

        Ok(TypeCondition {
            name: self.choose_named_ty(&named_types)?.name().clone(),
        })
    }
}
