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
        let mut exclude_names = Vec::new();
        let selection_nb = self.stack.last().map(|o| o.fields_def().len()).unwrap_or(0);

        let selections = (0..self.u.int_in_range(0..=5)?)
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

/// A field selected under some response name, with what the merging rules
/// need to compare it against other selections of that name.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SelectedField {
    pub(crate) parent: Name,
    pub(crate) ty: Ty,
    pub(crate) args: Vec<Argument>,
}

/// Every field selected in one selection set, keyed by response name.
/// Same-named fields merge their subselections, so each name also keeps
/// one child scope shared by all of their subselections.
#[derive(Debug, Clone, Default)]
pub(crate) struct ResponseScope {
    entries: IndexMap<Name, ResponseEntry>,
}

#[derive(Debug, Clone, Default)]
struct ResponseEntry {
    fields: Vec<SelectedField>,
    children: ResponseScope,
}

impl ResponseScope {
    fn descend(&self, path: &[Name]) -> Option<&ResponseScope> {
        path.iter().try_fold(self, |scope, name| {
            scope.entries.get(name).map(|e| &e.children)
        })
    }

    fn descend_mut(&mut self, path: &[Name]) -> &mut ResponseScope {
        path.iter().fold(self, |scope, name| {
            &mut scope.entries.entry(name.clone()).or_default().children
        })
    }

    fn merge(&mut self, other: &ResponseScope) {
        for (name, entry) in &other.entries {
            let dst = self.entries.entry(name.clone()).or_default();
            for field in &entry.fields {
                dst.add(field.clone());
            }
            dst.children.merge(&entry.children);
        }
    }
}

impl ResponseEntry {
    // Merge checks compare against every field under a name, so repeats of
    // the same field would only make them slower.
    fn add(&mut self, field: SelectedField) {
        if !self.fields.contains(&field) {
            self.fields.push(field);
        }
    }
}

/// The scope tree for one operation or fragment definition, plus the path
/// of response names leading to the selection set being generated.
#[derive(Debug, Default)]
pub(crate) struct ResponseScopeFrame {
    root: ResponseScope,
    path: Vec<Name>,
}

impl DocumentBuilder<'_> {
    /// Run `f` with a fresh scope tree and return what it selected.
    /// Operations and fragment definitions each start one, since they are
    /// validated independently.
    pub(crate) fn with_response_scope<T>(
        &mut self,
        f: impl FnOnce(&mut Self) -> T,
    ) -> (T, ResponseScope) {
        self.response_scopes.push(ResponseScopeFrame::default());
        let res = f(self);
        let frame = self
            .response_scopes
            .pop()
            .expect("the frame pushed above is still on the stack");
        (res, frame.root)
    }

    /// Run `f` in the scope shared by subselections of every field selected
    /// as `response_name` in the current scope.
    pub(crate) fn with_child_scope<T>(
        &mut self,
        response_name: &Name,
        f: impl FnOnce(&mut Self) -> T,
    ) -> T {
        if let Some(frame) = self.response_scopes.last_mut() {
            frame.path.push(response_name.clone());
        }
        let res = f(self);
        if let Some(frame) = self.response_scopes.last_mut() {
            frame.path.pop();
        }
        res
    }

    fn current_scope(&self) -> Option<&ResponseScope> {
        let frame = self.response_scopes.last()?;
        frame.root.descend(&frame.path)
    }

    /// Whether `field` can share `response_name` with every field already
    /// selected under that name in the current scope.
    pub(crate) fn can_merge(&self, response_name: &Name, field: &SelectedField) -> bool {
        self.current_scope()
            .is_none_or(|scope| self.can_merge_into(scope, response_name, field))
    }

    /// Requiring identical types is stricter than the spec's
    /// SameResponseShape, but it keeps the check local to one level, with
    /// subselections checked through the shared child scope. Arguments must
    /// match unless the parents are distinct object types.
    ///
    /// See <https://spec.graphql.org/September2025/#sec-Field-Selection-Merging>.
    fn can_merge_into(
        &self,
        scope: &ResponseScope,
        response_name: &Name,
        field: &SelectedField,
    ) -> bool {
        let Some(entry) = scope.entries.get(response_name) else {
            return true;
        };
        entry.fields.iter().all(|other| {
            let distinct_objects = other.parent != field.parent
                && self.is_object_type(&other.parent)
                && self.is_object_type(&field.parent);
            other.ty == field.ty && (distinct_objects || other.args == field.args)
        })
    }

    /// Like `can_merge`, for a field whose arguments aren't chosen yet.
    pub(crate) fn can_merge_ignoring_args(&self, response_name: &Name, ty: &Ty) -> bool {
        self.current_scope()
            .and_then(|scope| scope.entries.get(response_name))
            .is_none_or(|entry| entry.fields.iter().all(|other| &other.ty == ty))
    }

    pub(crate) fn record_selection(&mut self, response_name: Name, field: SelectedField) {
        if let Some(frame) = self.response_scopes.last_mut() {
            let scope = frame.root.descend_mut(&frame.path);
            scope.entries.entry(response_name).or_default().add(field);
        }
    }

    /// Whether everything a fragment selects, at every depth, can merge into
    /// the current scope.
    pub(crate) fn can_merge_fragment(&self, fragment_name: &Name) -> bool {
        let Some(selections) = self.fragment_scopes.get(fragment_name) else {
            return false;
        };
        self.current_scope()
            .is_none_or(|scope| self.can_merge_scope(scope, selections))
    }

    fn can_merge_scope(&self, scope: &ResponseScope, incoming: &ResponseScope) -> bool {
        incoming.entries.iter().all(|(name, entry)| {
            entry
                .fields
                .iter()
                .all(|f| self.can_merge_into(scope, name, f))
                && scope.entries.get(name).is_none_or(|existing| {
                    self.can_merge_scope(&existing.children, &entry.children)
                })
        })
    }

    pub(crate) fn record_fragment(&mut self, fragment_name: &Name) {
        let Some(selections) = self.fragment_scopes.get(fragment_name) else {
            return;
        };
        if let Some(frame) = self.response_scopes.last_mut() {
            frame.root.descend_mut(&frame.path).merge(selections);
        }
    }

    /// Fragments generated by the builder capture their scope as they are
    /// generated. This derives it for fragments passed to `with_document`,
    /// returning `None` until the scopes of fragments it spreads are known.
    pub(crate) fn fragment_scope(&self, fragment: &FragmentDef) -> Option<ResponseScope> {
        self.scope_of(&fragment.type_condition.name, &fragment.selection_set)
    }

    fn scope_of(&self, parent: &Name, selection_set: &SelectionSet) -> Option<ResponseScope> {
        let mut scope = ResponseScope::default();
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let ty = self.output_field_ty(parent, &field.name)?;
                    let children = match &field.selection_set {
                        Some(inner) => self.scope_of(ty.name(), inner)?,
                        None => ResponseScope::default(),
                    };
                    let response_name = field.alias.clone().unwrap_or_else(|| field.name.clone());
                    let entry = scope.entries.entry(response_name).or_default();
                    entry.add(SelectedField {
                        parent: parent.clone(),
                        ty,
                        args: field.args.clone(),
                    });
                    entry.children.merge(&children);
                }
                Selection::InlineFragment(inline) => {
                    let parent = inline.type_condition.as_ref().map_or(parent, |tc| &tc.name);
                    scope.merge(&self.scope_of(parent, &inline.selection_set)?);
                }
                Selection::FragmentSpread(spread) => {
                    scope.merge(self.fragment_scopes.get(&spread.name)?);
                }
            }
        }
        Some(scope)
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

fn typename_ty() -> Ty {
    Ty::NonNull(Box::new(Ty::Named(Name::new("String".to_string()))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Unstructured;

    #[test]
    fn repeated_selections_are_recorded_once() {
        let mut u = Unstructured::new(&[]);
        let mut builder = DocumentBuilder::new(&mut u);
        let name = Name::new(String::from("f"));
        let field = SelectedField {
            parent: Name::new(String::from("T")),
            ty: Ty::Named(Name::new(String::from("Int"))),
            args: Vec::new(),
        };
        let (_, scope) = builder.with_response_scope(|b| {
            for _ in 0..3 {
                b.record_selection(name.clone(), field.clone());
            }
        });
        assert_eq!(scope.entries[&name].fields, vec![field]);
    }
}
