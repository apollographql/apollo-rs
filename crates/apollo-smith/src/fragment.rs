use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::selection_set::SelectionSet;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// The __fragmentDef type represents a fragment definition
///
/// *FragmentDefinition*:
///     fragment FragmentName TypeCondition Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentDefinition).
#[derive(Debug, Clone)]
pub struct FragmentDef {
    pub(crate) name: Name,
    pub(crate) type_condition: TypeCondition,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) selection_set: SelectionSet,
}

impl From<FragmentDef> for ast::Definition {
    fn from(x: FragmentDef) -> Self {
        ast::FragmentDefinition {
            description: None, // TODO(@goto-bus-stop): represent description
            name: x.name.into(),
            type_condition: x.type_condition.name.into(),
            directives: Directive::to_ast(x.directives),
            selection_set: x.selection_set.into(),
        }
        .into()
    }
}

impl TryFrom<apollo_parser::cst::FragmentDefinition> for FragmentDef {
    type Error = crate::FromError;

    fn try_from(fragment_def: apollo_parser::cst::FragmentDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            name: fragment_def.fragment_name().unwrap().name().unwrap().into(),
            directives: fragment_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            type_condition: fragment_def.type_condition().unwrap().into(),
            selection_set: fragment_def.selection_set().unwrap().try_into()?,
        })
    }
}

/// The __fragmentSpread type represents a named fragment used in a selection set.
///
/// *FragmentSpread*:
///     ... FragmentName Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentSpread).
#[derive(Debug, Clone)]
pub struct FragmentSpread {
    pub(crate) name: Name,
    pub(crate) directives: IndexMap<Name, Directive>,
}

impl From<FragmentSpread> for ast::FragmentSpread {
    fn from(x: FragmentSpread) -> Self {
        Self {
            fragment_name: x.name.into(),
            directives: Directive::to_ast(x.directives),
        }
    }
}

impl TryFrom<apollo_parser::cst::FragmentSpread> for FragmentSpread {
    type Error = crate::FromError;

    fn try_from(fragment_spread: apollo_parser::cst::FragmentSpread) -> Result<Self, Self::Error> {
        Ok(Self {
            name: fragment_spread
                .fragment_name()
                .unwrap()
                .name()
                .unwrap()
                .into(),
            directives: fragment_spread
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

/// The __inlineFragment type represents an inline fragment in a selection set that could be used as a field
///
/// *InlineFragment*:
///     ... TypeCondition? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Inline-Fragments).
#[derive(Debug, Clone)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<TypeCondition>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) selection_set: SelectionSet,
}

impl From<InlineFragment> for ast::InlineFragment {
    fn from(x: InlineFragment) -> Self {
        Self {
            type_condition: x.type_condition.map(|t| t.name.into()),
            directives: Directive::to_ast(x.directives),
            selection_set: x.selection_set.into(),
        }
    }
}

impl TryFrom<apollo_parser::cst::InlineFragment> for InlineFragment {
    type Error = crate::FromError;

    fn try_from(inline_fragment: apollo_parser::cst::InlineFragment) -> Result<Self, Self::Error> {
        Ok(Self {
            directives: inline_fragment
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            selection_set: inline_fragment.selection_set().unwrap().try_into()?,
            type_condition: inline_fragment.type_condition().map(TypeCondition::from),
        })
    }
}

/// The __typeCondition type represents where a fragment could be applied
///
/// *TypeCondition*:
///     on NamedType
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#TypeCondition).
#[derive(Debug, Clone)]
pub struct TypeCondition {
    name: Name,
}

impl From<apollo_parser::cst::TypeCondition> for TypeCondition {
    fn from(type_condition: apollo_parser::cst::TypeCondition) -> Self {
        Self {
            name: type_condition.named_type().unwrap().name().unwrap().into(),
        }
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `FragmentDef`
    pub fn fragment_definition(&mut self) -> ArbitraryResult<FragmentDef> {
        // TODO: also choose between enum/scalars/object
        let selected_object_type_name = self.u.choose(&self.object_type_defs)?.name.clone();
        let _ = self.stack_ty(&Ty::Named(selected_object_type_name));
        let name = self.type_name()?;
        let directives = self.directives(DirectiveLocation::FragmentDefinition)?;
        let selection_set = self.selection_set()?;
        let type_condition = self.type_condition()?;
        self.stack.pop();

        Ok(FragmentDef {
            name,
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Create an arbitrary `FragmentSpread`, returns `None` if no fragment definition was previously created
    pub fn fragment_spread(
        &mut self,
        excludes: &mut Vec<Name>,
    ) -> ArbitraryResult<Option<FragmentSpread>> {
        let available_fragment: Vec<&FragmentDef> = self
            .fragment_defs
            .iter()
            .filter(|f| excludes.contains(&f.name))
            .collect();

        let name = if available_fragment.is_empty() {
            return Ok(None);
        } else {
            self.u.choose(&available_fragment)?.name.clone()
        };
        let directives = self.directives(DirectiveLocation::FragmentSpread)?;
        excludes.push(name.clone());

        Ok(Some(FragmentSpread { name, directives }))
    }

    /// Create an arbitrary `InlineFragment`
    pub fn inline_fragment(&mut self) -> ArbitraryResult<InlineFragment> {
        let type_condition = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.type_condition())
            .transpose()?;
        let selection_set = self.selection_set()?;
        let directives = self.directives(DirectiveLocation::InlineFragment)?;

        Ok(InlineFragment {
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Create an arbitrary `TypeCondition`
    pub fn type_condition(&mut self) -> ArbitraryResult<TypeCondition> {
        let last_element = self.stack.last();
        match last_element {
            Some(last_element) => Ok(TypeCondition {
                name: last_element.name().clone(),
            }),
            None => {
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
    }
}
