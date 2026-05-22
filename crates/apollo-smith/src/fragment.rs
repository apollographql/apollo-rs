use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::operation::OperationDef;
use crate::selection_set::SelectionSet;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

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
            .filter(|f| !excludes.contains(&f.name))
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

/// Compute the set of fragment names reachable from `operations`, walking
/// through `fragments` transitively when one fragment spreads another.
///
/// A fragment is reachable iff some operation spreads it directly, or spreads
/// some other reachable fragment whose chain leads to it. Chains like
/// `A -> B` with no operation referencing A produce no reachable names, even
/// though `A` syntactically references `B`.
pub(crate) fn reachable_fragment_names(
    operations: &[OperationDef],
    fragments: &[FragmentDef],
) -> IndexSet<Name> {
    let mut reachable: IndexSet<Name> = IndexSet::new();
    for op in operations {
        op.selection_set.collect_fragment_spreads(&mut reachable);
    }
    let mut frontier: Vec<Name> = reachable.iter().cloned().collect();
    while let Some(name) = frontier.pop() {
        if let Some(frag) = fragments.iter().find(|f| f.name == name) {
            let mut nested: IndexSet<Name> = IndexSet::new();
            frag.selection_set.collect_fragment_spreads(&mut nested);
            for n in nested {
                if reachable.insert(n.clone()) {
                    frontier.push(n);
                }
            }
        }
    }
    reachable
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::OperationType;
    use crate::selection_set::Selection;
    use crate::selection_set::SelectionSet;

    fn n(s: &str) -> Name {
        Name::new(s.to_string())
    }

    fn spread(target: &str) -> Selection {
        Selection::FragmentSpread(FragmentSpread {
            name: n(target),
            directives: IndexMap::new(),
        })
    }

    fn sset(selections: Vec<Selection>) -> SelectionSet {
        SelectionSet { selections }
    }

    fn names(items: &[&str]) -> IndexSet<Name> {
        items.iter().map(|s| n(s)).collect()
    }

    fn op(spreads: Vec<&str>) -> OperationDef {
        OperationDef {
            operation_type: OperationType::Query,
            name: None,
            variable_definitions: vec![],
            directives: IndexMap::new(),
            selection_set: sset(spreads.into_iter().map(spread).collect()),
        }
    }

    fn frag(name: &str, on: &str, spreads: Vec<&str>) -> FragmentDef {
        FragmentDef {
            name: n(name),
            type_condition: TypeCondition { name: n(on) },
            directives: IndexMap::new(),
            selection_set: sset(spreads.into_iter().map(spread).collect()),
        }
    }

    #[test]
    fn no_operations_means_nothing_reachable() {
        let frags = vec![frag("A", "T", vec![])];
        let result = reachable_fragment_names(&[], &frags);
        assert!(result.is_empty());
    }

    #[test]
    fn direct_spread_is_reachable() {
        let ops = vec![op(vec!["A"])];
        let frags = vec![frag("A", "T", vec![])];
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A"]));
    }

    #[test]
    fn transitive_chain_is_reachable() {
        // op -> A -> B -> C : all three reachable
        let ops = vec![op(vec!["A"])];
        let frags = vec![
            frag("A", "T", vec!["B"]),
            frag("B", "T", vec!["C"]),
            frag("C", "T", vec![]),
        ];
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A", "B", "C"]));
    }

    #[test]
    fn orphan_chain_is_not_reachable() {
        // A -> B with no operation referencing A: neither retained
        let ops: Vec<OperationDef> = vec![];
        let frags = vec![frag("A", "T", vec!["B"]), frag("B", "T", vec![])];
        let result = reachable_fragment_names(&ops, &frags);
        assert!(result.is_empty());
    }

    #[test]
    fn unreferenced_fragment_among_used_ones_is_pruned() {
        // op -> A, but B exists in isolation -> only A reachable
        let ops = vec![op(vec!["A"])];
        let frags = vec![frag("A", "T", vec![]), frag("B", "T", vec![])];
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A"]));
    }

    #[test]
    fn cycle_terminates() {
        // op -> A -> B -> A : both reachable, no infinite loop
        let ops = vec![op(vec!["A"])];
        let frags = vec![frag("A", "T", vec!["B"]), frag("B", "T", vec!["A"])];
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A", "B"]));
    }
}
