use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::FragmentDefinition;
use apollo_compiler::ast::FragmentSpread;
use apollo_compiler::ast::InlineFragment;
use apollo_compiler::ast::OperationDefinition;
use apollo_compiler::ast::Type;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;
use itertools::Itertools;

impl DocumentBuilder<'_> {
    /// Create an arbitrary `FragmentDefinition`
    pub fn fragment_definition(&mut self) -> ArbitraryResult<FragmentDefinition> {
        let type_conditions = self.fragment_type_conditions();
        let type_condition = self.u.choose(&type_conditions)?.clone();
        let _ = self.stack_ty(&Type::Named(type_condition));
        let name = self.type_name()?;
        let directives = self.directives(DirectiveLocation::FragmentDefinition)?;
        let (selection_set, selections) = self.tracking_merges(|b| b.selection_set());
        let selection_set = selection_set?;
        self.fragment_selections.insert(name.clone(), selections);
        if let Some(depth) = self.nesting_depth(&selection_set) {
            self.fragment_depths.insert(name.clone(), depth);
        }
        let type_condition = self.type_condition()?;
        self.stack.pop();

        Ok(FragmentDefinition {
            description: None,
            name,
            type_condition,
            directives,
            selection_set,
        })
    }

    /// Types a fragment can be defined on. Fragments select fields, so leaf
    /// types are left out.
    pub(crate) fn fragment_type_conditions(&self) -> Vec<Name> {
        let objects = self.object_type_defs.iter().map(|o| o.type_name());
        let interfaces = self.interface_type_defs.iter().map(|i| i.type_name());
        let unions = self.union_type_defs.iter().map(|u| u.type_name());
        objects
            .chain(interfaces)
            .chain(unions)
            .unique()
            .cloned()
            .collect()
    }

    /// Create an arbitrary `FragmentSpread`, returns `None` if no fragment definition was previously created
    pub fn fragment_spread(
        &mut self,
        excludes: &mut Vec<Name>,
    ) -> ArbitraryResult<Option<FragmentSpread>> {
        let current_type = self.stack.last().map(|e| e.type_name().clone());
        let available_fragment: Vec<&Node<FragmentDefinition>> = self
            .fragment_defs
            .iter()
            .filter(|f| {
                !excludes.contains(&f.name)
                    && self.fragment_spread_possible(&f.type_condition, current_type.as_ref())
                    && self.can_merge_fragment(&f.name)
                    && self.fragment_fits_depth(&f.name)
            })
            .collect();

        let name = if available_fragment.is_empty() {
            return Ok(None);
        } else {
            self.u.choose(&available_fragment)?.name.clone()
        };
        let directives = self.selection_directives(DirectiveLocation::FragmentSpread, true)?;
        self.record_fragment(&name);
        excludes.push(name.clone());

        Ok(Some(FragmentSpread {
            fragment_name: name,
            directives,
        }))
    }

    /// Create an arbitrary `InlineFragment`
    pub fn inline_fragment(&mut self) -> ArbitraryResult<InlineFragment> {
        if let Some(member) = self.choose_union_member()? {
            let type_condition = member.clone();
            self.stack_ty(&Type::Named(member));
            let selection_set = self.selection_set();
            self.stack.pop();
            let directives = self.selection_directives(DirectiveLocation::InlineFragment, true)?;
            return Ok(InlineFragment {
                type_condition: Some(type_condition),
                directives,
                selection_set: selection_set?,
            });
        }

        let type_condition = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.type_condition())
            .transpose()?;
        let selection_set = self.selection_set()?;
        let directives = self.selection_directives(DirectiveLocation::InlineFragment, true)?;

        Ok(InlineFragment {
            type_condition,
            directives,
            selection_set,
        })
    }

    /// When selecting on a union, usually pick one of its members to narrow
    /// to, since `__typename` is the only field a union itself offers.
    fn choose_union_member(&mut self) -> ArbitraryResult<Option<Name>> {
        let Some(current) = self.stack.last().map(|e| e.type_name().clone()) else {
            return Ok(None);
        };
        if !self
            .union_type_defs
            .iter()
            .any(|u| u.type_name() == &current)
        {
            return Ok(None);
        }
        let members: Vec<Name> = self.union_members(&current).into_iter().collect();
        if members.is_empty() || self.u.ratio(1, 4)? {
            return Ok(None);
        }
        Ok(Some(self.u.choose(&members)?.clone()))
    }

    /// Whether a fragment with `fragment_type` can be spread inside a
    /// selection set for `current_type`. The two types must share at
    /// least one possible object type.
    ///
    /// See <https://spec.graphql.org/September2025/#sec-Fragment-Spread-Is-Possible>.
    fn fragment_spread_possible(&self, fragment_type: &Name, current_type: Option<&Name>) -> bool {
        let Some(current) = current_type else {
            return true;
        };
        let current_objects = self.possible_object_types(current);
        let fragment_objects = self.possible_object_types(fragment_type);
        current_objects.iter().any(|o| fragment_objects.contains(o))
    }

    /// The set of object types that `type_name` can resolve to at runtime.
    fn possible_object_types(&self, type_name: &Name) -> IndexSet<Name> {
        if self
            .object_type_defs
            .iter()
            .any(|o| o.type_name() == type_name)
        {
            return IndexSet::from([type_name.clone()]);
        }
        if self
            .union_type_defs
            .iter()
            .any(|u| u.type_name() == type_name)
        {
            return self.union_members(type_name);
        }
        // Interface: collect every object whose implements closure includes it
        self.object_type_defs
            .iter()
            .filter(|o| {
                self.implements_graph
                    .closure(o.type_name())
                    .contains(type_name)
            })
            .map(|o| o.type_name().clone())
            .collect()
    }

    /// Choose a type condition for the current selection.
    pub fn type_condition(&mut self) -> ArbitraryResult<Name> {
        match self.stack.last() {
            Some(parent) => Ok(parent.type_name().clone()),
            None => Ok(self
                .choose_named_ty(&self.list_existing_object_types())?
                .inner_named_type()
                .clone()),
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
    operations: &[Node<OperationDefinition>],
    fragments: &[Node<FragmentDefinition>],
) -> IndexSet<Name> {
    let mut reachable: IndexSet<Name> = IndexSet::new();
    for op in operations {
        crate::selection_set::collect_fragment_spreads(&op.selection_set, &mut reachable);
    }
    let mut frontier: Vec<Name> = reachable.iter().cloned().collect();
    while let Some(name) = frontier.pop() {
        if let Some(frag) = fragments.iter().find(|f| f.name == name) {
            let mut nested: IndexSet<Name> = IndexSet::new();
            crate::selection_set::collect_fragment_spreads(&frag.selection_set, &mut nested);
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
    use crate::test_support::generated;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    fn parse(
        src: &str,
    ) -> (
        Vec<Node<OperationDefinition>>,
        Vec<Node<FragmentDefinition>>,
    ) {
        let document = apollo_compiler::ast::Document::parse(src, "fragments.graphql")
            .expect("parse fragments");
        let mut ops = Vec::new();
        let mut frags = Vec::new();
        for definition in document.definitions {
            match definition {
                apollo_compiler::ast::Definition::OperationDefinition(operation) => {
                    ops.push(operation)
                }
                apollo_compiler::ast::Definition::FragmentDefinition(fragment) => {
                    frags.push(fragment)
                }
                _ => panic!("unexpected definition in test input"),
            }
        }
        (ops, frags)
    }

    fn names(items: &[&str]) -> IndexSet<Name> {
        items
            .iter()
            .map(|s| Name::new(s).expect("valid generated name"))
            .collect()
    }

    #[test]
    fn no_operations_means_nothing_reachable() {
        let (ops, frags) = parse("fragment A on T { __typename }");
        let result = reachable_fragment_names(&ops, &frags);
        assert!(result.is_empty());
    }

    #[test]
    fn direct_spread_is_reachable() {
        let (ops, frags) = parse(
            "
            query { ...A }
            fragment A on T { __typename }
            ",
        );
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A"]));
    }

    #[test]
    fn transitive_chain_is_reachable() {
        let (ops, frags) = parse(
            "
            query { ...A }
            fragment A on T { ...B }
            fragment B on T { ...C }
            fragment C on T { __typename }
            ",
        );
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A", "B", "C"]));
    }

    #[test]
    fn orphan_chain_is_not_reachable() {
        // A -> B with no operation referencing A: neither retained
        let (ops, frags) = parse(
            "
            fragment A on T { ...B }
            fragment B on T { __typename }
            ",
        );
        let result = reachable_fragment_names(&ops, &frags);
        assert!(result.is_empty());
    }

    #[test]
    fn unreferenced_fragment_among_used_ones_is_pruned() {
        let (ops, frags) = parse(
            "
            query { ...A }
            fragment A on T { __typename }
            fragment B on T { __typename }
            ",
        );
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A"]));
    }

    #[test]
    fn cycle_terminates() {
        // op -> A -> B -> A : both reachable, no infinite loop
        let (ops, frags) = parse(
            "
            query { ...A }
            fragment A on T { ...B }
            fragment B on T { ...A }
            ",
        );
        let result = reachable_fragment_names(&ops, &frags);
        assert_eq!(result, names(&["A", "B"]));
    }

    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!) on FRAGMENT_DEFINITION | FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT
        type Query { i: I, u: U }
        interface I { x: Int }
        type A implements I { x: Int, a: A }
        type B { b: Int }
        union U = A | B
        fragment FA on A { x }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.fragment_definition().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }

    #[test]
    fn fragments_can_be_defined_on_objects_interfaces_and_unions() {
        let schema = apollo_compiler::ast::Document::parse(SCHEMA, "schema.graphql")
            .expect("parse compiler document");
        let mut u = arbitrary::Unstructured::new(&[]);
        let builder =
            crate::DocumentBuilder::with_document(&mut u, schema).expect("builder from schema");
        let candidates = builder.fragment_type_conditions();
        let names: Vec<&str> = candidates.iter().map(|n| n.as_str()).collect();
        assert_eq!(names, ["Query", "A", "B", "I", "U"]);
    }
}
