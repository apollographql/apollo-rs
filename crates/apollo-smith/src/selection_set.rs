use crate::argument::Argument;
use crate::field::Field;
use crate::fragment::FragmentDef;
use crate::fragment::FragmentSpread;
use crate::fragment::InlineFragment;
use crate::name::Name;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

/// The __selectionSet type represents a selection_set type in a fragment spread, an operation or a field
///
/// *SelectionSet*:
///     Selection*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Selection-Sets).
#[derive(Debug, Clone)]
pub struct SelectionSet {
    pub(crate) selections: Vec<Selection>,
}

impl From<SelectionSet> for Vec<ast::Selection> {
    fn from(sel_set: SelectionSet) -> Self {
        sel_set.selections.into_iter().map(Into::into).collect()
    }
}

impl SelectionSet {
    pub(crate) fn collect_fragment_spreads(&self, into: &mut IndexSet<Name>) {
        for selection in &self.selections {
            match selection {
                Selection::Field(field) => {
                    if let Some(inner) = &field.selection_set {
                        inner.collect_fragment_spreads(into);
                    }
                }
                Selection::FragmentSpread(spread) => {
                    into.insert(spread.name.clone());
                }
                Selection::InlineFragment(inline) => {
                    inline.selection_set.collect_fragment_spreads(into);
                }
            }
        }
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
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#Selection).
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

impl DocumentBuilder<'_> {
    /// Create an arbitrary `SelectionSet`
    pub fn selection_set(&mut self) -> ArbitraryResult<SelectionSet> {
        if self.selection_depth >= MAX_SELECTION_DEPTH {
            return Ok(SelectionSet {
                selections: Vec::new(),
            });
        }
        self.selection_depth += 1;
        let selection_set = self.selections();
        self.selection_depth -= 1;
        selection_set
    }

    fn selections(&mut self) -> ArbitraryResult<SelectionSet> {
        let mut exclude_names = Vec::new();
        let selection_nb = self.stack.last().map(|o| o.fields_def().len()).unwrap_or(0);

        let selections = (0..self.u.int_in_range(0..=5)?)
            .map(|_| {
                let index = self.u.int_in_range(0..=selection_nb)?;
                self.selection(index, &mut exclude_names)
            })
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

/// A field selected under some response name, with what the merging rules
/// need to compare it against other selections of that name.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SelectedField {
    pub(crate) parent: Name,
    pub(crate) field_name: Name,
    pub(crate) ty: Ty,
    pub(crate) args: Vec<Argument>,
}

/// The fields already selected in a selection set, grouped by the response
/// name they appear under, so that new selections can be checked against
/// the field selection merging rules before they're added.
///
/// GraphQL merges every selection with the same response name, including
/// their subselections. Given
///
/// ```graphql
/// { u { ... on A { shared } }  u { ... on B { shared } } }
/// ```
///
/// both `shared` fields end up in one merged `u`, so they must be
/// compatible even though they're in different `u` selections. Each
/// response name therefore keeps one record of its merged subselections,
/// shared by every field selected under that name.
#[derive(Debug, Clone, Default)]
pub(crate) struct SelectionsByResponseName {
    by_name: IndexMap<Name, SelectionsUnderName>,
}

#[derive(Debug, Clone, Default)]
struct SelectionsUnderName {
    fields: Vec<SelectedField>,
    merged_subselections: SelectionsByResponseName,
}

impl SelectionsByResponseName {
    fn at(&self, response_path: &[Name]) -> Option<&SelectionsByResponseName> {
        response_path.iter().try_fold(self, |selections, name| {
            selections
                .by_name
                .get(name)
                .map(|under_name| &under_name.merged_subselections)
        })
    }

    fn at_mut(&mut self, response_path: &[Name]) -> &mut SelectionsByResponseName {
        response_path.iter().fold(self, |selections, name| {
            &mut selections
                .by_name
                .entry(name.clone())
                .or_default()
                .merged_subselections
        })
    }

    fn merge(&mut self, other: &SelectionsByResponseName) {
        for (name, other_under_name) in &other.by_name {
            let under_name = self.by_name.entry(name.clone()).or_default();
            for field in &other_under_name.fields {
                under_name.add(field.clone());
            }
            under_name
                .merged_subselections
                .merge(&other_under_name.merged_subselections);
        }
    }
}

impl SelectionsUnderName {
    // Merge checks compare against every field under a name, so repeats of
    // the same field would only make them slower.
    fn add(&mut self, field: SelectedField) {
        if !self.fields.contains(&field) {
            self.fields.push(field);
        }
    }
}

/// What one operation or fragment definition has selected so far, and the
/// response names leading from its root to the selection set currently
/// being generated.
#[derive(Debug, Default)]
pub(crate) struct MergeTracking {
    selections: SelectionsByResponseName,
    response_path: Vec<Name>,
}

impl DocumentBuilder<'_> {
    /// Run `f` while recording what it selects, and return that record.
    /// Operations and fragment definitions each start their own, since they
    /// are validated independently.
    pub(crate) fn tracking_merges<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> (T, SelectionsByResponseName) {
        self.merge_tracking.push(MergeTracking::default());
        let res = f(self);
        let tracking = self
            .merge_tracking
            .pop()
            .expect("the tracking pushed above is still on the stack");
        (res, tracking.selections)
    }

    /// Run `f` to generate the subselection of a field selected as
    /// `response_name`. Its selections are checked against, and recorded
    /// with, the subselections of every other field selected under that
    /// name here, since GraphQL merges them.
    pub(crate) fn in_merged_subselection<T>(
        &mut self,
        response_name: &Name,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if let Some(tracking) = self.merge_tracking.last_mut() {
            tracking.response_path.push(response_name.clone());
        }
        let res = f(self);
        if let Some(tracking) = self.merge_tracking.last_mut() {
            tracking.response_path.pop();
        }
        res
    }

    /// What's already selected in the selection set being generated.
    fn selections_here(&self) -> Option<&SelectionsByResponseName> {
        let tracking = self.merge_tracking.last()?;
        tracking.selections.at(&tracking.response_path)
    }

    /// Whether `field`, with its arguments chosen, can be selected as
    /// `response_name` alongside what's already selected here.
    pub(crate) fn can_merge(&self, response_name: &Name, field: &SelectedField) -> bool {
        self.selections_here()
            .is_none_or(|selections| self.can_merge_into(selections, response_name, field))
    }

    /// Requiring identical types is stricter than the spec's
    /// SameResponseShape, but it keeps the check local to one level, with
    /// subselections checked through their merged record. Unless the
    /// parents are distinct object types, the fields must also be the same
    /// field with the same arguments, which only aliases can break.
    ///
    /// See <https://spec.graphql.org/September2025/#sec-Field-Selection-Merging>.
    fn can_merge_into(
        &self,
        selections: &SelectionsByResponseName,
        response_name: &Name,
        field: &SelectedField,
    ) -> bool {
        let Some(under_name) = selections.by_name.get(response_name) else {
            return true;
        };
        under_name.fields.iter().all(|other| {
            self.compatible(
                other,
                &field.parent,
                &field.field_name,
                &field.ty,
                Some(&field.args),
            )
        })
    }

    /// Like `can_merge`, for a field whose arguments aren't chosen yet.
    pub(crate) fn can_merge_ignoring_args(
        &self,
        response_name: &Name,
        parent: &Name,
        field_name: &Name,
        ty: &Ty,
    ) -> bool {
        self.selections_here()
            .and_then(|selections| selections.by_name.get(response_name))
            .is_none_or(|under_name| {
                under_name
                    .fields
                    .iter()
                    .all(|other| self.compatible(other, parent, field_name, ty, None))
            })
    }

    /// Whether a field can share a response name with `other`, comparing
    /// arguments only when they're known.
    fn compatible(
        &self,
        other: &SelectedField,
        parent: &Name,
        field_name: &Name,
        ty: &Ty,
        args: Option<&[Argument]>,
    ) -> bool {
        let distinct_objects = &other.parent != parent
            && self.is_object_type(&other.parent)
            && self.is_object_type(parent);
        &other.ty == ty
            && (distinct_objects
                || (&other.field_name == field_name && args.is_none_or(|args| other.args == args)))
    }

    /// Whether `response_name` is already selected where the current
    /// selection is being generated.
    pub(crate) fn has_response_name(&self, response_name: &Name) -> bool {
        self.selections_here()
            .is_some_and(|selections| selections.by_name.contains_key(response_name))
    }

    /// Response names already selected where the current selection is
    /// being generated, leaving out reserved introspection names.
    pub(crate) fn current_response_names(&self) -> Vec<Name> {
        self.selections_here()
            .map(|selections| {
                selections
                    .by_name
                    .keys()
                    .filter(|name| !name.name.starts_with("__"))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn record_selection(&mut self, response_name: Name, field: SelectedField) {
        if let Some(tracking) = self.merge_tracking.last_mut() {
            let selections = tracking.selections.at_mut(&tracking.response_path);
            selections
                .by_name
                .entry(response_name)
                .or_default()
                .add(field);
        }
    }

    /// Whether everything a fragment selects, at every depth, can merge with
    /// what's already selected here.
    pub(crate) fn can_merge_fragment(&self, fragment_name: &Name) -> bool {
        let Some(fragment_selections) = self.fragment_selections.get(fragment_name) else {
            return false;
        };
        self.selections_here()
            .is_none_or(|selections| self.can_merge_selections(selections, fragment_selections))
    }

    fn can_merge_selections(
        &self,
        selections: &SelectionsByResponseName,
        incoming: &SelectionsByResponseName,
    ) -> bool {
        incoming.by_name.iter().all(|(name, incoming_under_name)| {
            incoming_under_name
                .fields
                .iter()
                .all(|f| self.can_merge_into(selections, name, f))
                && selections.by_name.get(name).is_none_or(|under_name| {
                    self.can_merge_selections(
                        &under_name.merged_subselections,
                        &incoming_under_name.merged_subselections,
                    )
                })
        })
    }

    pub(crate) fn record_fragment(&mut self, fragment_name: &Name) {
        let Some(fragment_selections) = self.fragment_selections.get(fragment_name) else {
            return;
        };
        if let Some(tracking) = self.merge_tracking.last_mut() {
            tracking
                .selections
                .at_mut(&tracking.response_path)
                .merge(fragment_selections);
        }
    }

    /// How many selection sets deep `selection_set` nests, counting itself
    /// and those of the fragments it spreads. `None` if it spreads a
    /// fragment whose depth isn't known yet.
    pub(crate) fn nesting_depth(&self, selection_set: &SelectionSet) -> Option<usize> {
        let mut deepest = 0;
        for selection in &selection_set.selections {
            let depth = match selection {
                Selection::Field(field) => match &field.selection_set {
                    Some(inner) => self.nesting_depth(inner)?,
                    None => 0,
                },
                Selection::InlineFragment(inline) => self.nesting_depth(&inline.selection_set)?,
                Selection::FragmentSpread(spread) => *self.fragment_depths.get(&spread.name)?,
            };
            deepest = deepest.max(depth);
        }
        Some(deepest + 1)
    }

    /// Whether spreading the fragment here keeps selections within
    /// `MAX_SELECTION_DEPTH`.
    pub(crate) fn fragment_fits_depth(&self, fragment_name: &Name) -> bool {
        self.fragment_depths
            .get(fragment_name)
            .is_some_and(|depth| self.selection_depth + depth <= MAX_SELECTION_DEPTH)
    }

    /// Fragments generated by the builder record their selections as they
    /// are generated. This derives them for fragments passed to
    /// `with_document`, returning `None` until the selections of fragments
    /// it spreads are known.
    pub(crate) fn selections_of_fragment(
        &self,
        fragment: &FragmentDef,
    ) -> Option<SelectionsByResponseName> {
        self.selections_of(&fragment.type_condition.name, &fragment.selection_set)
    }

    fn selections_of(
        &self,
        parent: &Name,
        selection_set: &SelectionSet,
    ) -> Option<SelectionsByResponseName> {
        let mut selections = SelectionsByResponseName::default();
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let ty = self.output_field_ty(parent, &field.name)?;
                    let subselections = match &field.selection_set {
                        Some(inner) => self.selections_of(ty.name(), inner)?,
                        None => SelectionsByResponseName::default(),
                    };
                    let response_name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                    let under_name = selections.by_name.entry(response_name).or_default();
                    under_name.add(SelectedField {
                        parent: parent.clone(),
                        field_name: field.name.clone(),
                        ty,
                        args: field.args.clone(),
                    });
                    under_name.merged_subselections.merge(&subselections);
                }
                Selection::InlineFragment(inline) => {
                    let parent = inline.type_condition.as_ref().map_or(parent, |tc| &tc.name);
                    selections.merge(&self.selections_of(parent, &inline.selection_set)?);
                }
                Selection::FragmentSpread(spread) => {
                    selections.merge(self.fragment_selections.get(&spread.name)?);
                }
            }
        }
        Some(selections)
    }

    fn output_field_ty(&self, parent: &Name, field_name: &Name) -> Option<Ty> {
        if field_name.name == TYPENAME {
            return Some(typename_ty());
        }
        let object_fields = self
            .object_type_defs
            .iter()
            .filter(|o| &o.name == parent)
            .flat_map(|o| &o.fields_def);
        let interface_fields = self
            .interface_type_defs
            .iter()
            .filter(|i| &i.name == parent)
            .flat_map(|i| &i.fields_def);
        object_fields
            .chain(interface_fields)
            .find(|f| &f.name == field_name)
            .map(|f| f.ty.clone())
    }

    fn is_object_type(&self, name: &Name) -> bool {
        self.object_type_defs.iter().any(|o| &o.name == name)
    }

    /// A `__typename` selection, valid on any composite type and always
    /// mergeable with other `__typename` selections.
    pub(crate) fn typename_field(&mut self) -> Field {
        let name = Name::new(TYPENAME.to_string());
        let parent = self
            .stack
            .last()
            .expect("a composite type must be on the stack")
            .name()
            .clone();
        self.record_selection(
            name.clone(),
            SelectedField {
                parent,
                field_name: name.clone(),
                ty: typename_ty(),
                args: Vec::new(),
            },
        );
        Field {
            alias: None,
            name,
            args: Vec::new(),
            directives: IndexMap::new(),
            selection_set: None,
        }
    }
}

const TYPENAME: &str = "__typename";

// apollo-compiler reports "too much recursion" when checking field merging
// for selections nested more than 128 fields below the operation, so this
// counts the operation's own selection set plus 128.
const MAX_SELECTION_DEPTH: usize = 129;

fn typename_ty() -> Ty {
    Ty::NonNull(Box::new(Ty::Named(Name::new("String".to_string()))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::generate;
    use crate::test_support::generated_against_any;
    use crate::test_support::generated_from;
    use crate::test_support::prop_assert_valid;
    use apollo_compiler::ast;
    use arbitrary::Unstructured;
    use proptest::prelude::*;

    #[test]
    fn repeated_selections_are_recorded_once() {
        let mut u = Unstructured::new(&[]);
        let mut builder = DocumentBuilder::new(&mut u);
        let name = Name::new(String::from("f"));
        let field = SelectedField {
            parent: Name::new(String::from("T")),
            field_name: name.clone(),
            ty: Ty::Named(Name::new(String::from("Int"))),
            args: Vec::new(),
        };
        let (_, selections) = builder.tracking_merges(|b| {
            for _ in 0..3 {
                b.record_selection(name.clone(), field.clone());
            }
        });
        assert_eq!(selections.by_name[&name].fields, vec![field]);
    }

    // Exhausted input picks the minimum selection count, which is zero.
    #[test]
    fn exhausted_input_generates_valid_empty_selection_sets() {
        let schema = "schema { query: Query } type Query { id: ID }";
        let doc =
            generate(schema, &[], |b| b.operation_definition()).expect("schema has a query root");
        let ast = ast::Document::parse(&doc, "smith.graphql").expect("smith output parses");
        let op = ast
            .definitions
            .iter()
            .find_map(|d| d.as_operation_definition())
            .expect("document contains the generated operation");
        assert!(op.selection_set.is_empty(), "{doc}");
        ast.to_mixed_validate()
            .expect("empty selection set validates");
    }

    // An implementer's fragment selects `f` with a fixed argument, while the
    // generator picks its own arguments for `I.f`. Selecting both in one
    // interface selection set would conflict, since one parent isn't an
    // object.
    const INTERFACE_FRAGMENT: &str = "schema { query: Query }
        type Query { i: I, j: I }
        interface I { f(x: Int): Int, g: Int }
        type A implements I { f(x: Int): Int, g: Int }
        fragment FA on A { f(x: 1) g }";

    // Members share a field name with different types.
    const UNION: &str = "schema { query: Query }
        type Query { u: U }
        union U = A | B
        type A { a: Int, shared: Int }
        type B { b: String, shared: String }";

    // A member added by an extension, with a fragment on it.
    const EXTENDED_UNION: &str = "schema { query: Query }
        type Query { u: U }
        union U = A
        extend union U = B
        type A { a: Int }
        type B { b: Int }
        fragment FB on B { b }";

    const CUSTOM_SCALAR: &str = "schema { query: Query }
        type Query { s: Custom, i: Int }
        scalar Custom";

    // Members reuse field names with different types, so some fields can't
    // merge.
    const CONFLICTING_MEMBERS: &str = "schema { query: Query }
        type Query { u: U }
        union U = A | B
        type A { x: Int, y: Int }
        type B { x: String, y: Int }";

    #[test]
    fn max_selection_depth_is_the_deepest_that_validates() {
        let schema = "schema { query: Query } type Query { a: A } type A { x: Int, a: A }";
        let validates = |selection_sets: usize| {
            let fields = selection_sets - 1;
            let operation = format!("{{ {}x{} }}", "a { ".repeat(fields), " }".repeat(fields));
            ast::Document::parse(format!("{schema}\n{operation}"), "smith.graphql")
                .expect("operation parses")
                .to_mixed_validate()
                .is_ok()
        };
        assert!(validates(MAX_SELECTION_DEPTH));
        assert!(!validates(MAX_SELECTION_DEPTH + 1));
    }

    // Fields of their own type let selections nest as deeply as the input
    // allows, through fields and through fragments.
    const RECURSIVE: &str = "schema { query: Query }
        type Query { a: A }
        type A { x: Int, a: A, b: [A] }
        fragment FA on A { a { a { x } } }";

    proptest! {
        #[test]
        fn recursive_selections_stay_within_validation_depth(
            case in generated_from(
                prop::collection::vec(any::<u8>(), 0..=16384),
                RECURSIVE,
                |b| b.operation_definition(),
            ),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }

    proptest! {
        #[test]
        fn selections_stay_mergeable(
            case in generated_against_any(
                &[INTERFACE_FRAGMENT, UNION, EXTENDED_UNION, CUSTOM_SCALAR, CONFLICTING_MEMBERS],
                |b| b.operation_definition(),
            ),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
