use std::{collections::HashSet, sync::Arc};

use crate::{
    hir::{
        Directive, Field, FragmentDefinition, HirNodeLocation, Name, TypeDefinition, Value,
        Variable,
    },
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SelectionSet {
    pub(crate) selection: Arc<Vec<Selection>>,
}

impl SelectionSet {
    /// Get a reference to the selection set's selection.
    pub fn selection(&self) -> &[Selection] {
        self.selection.as_ref()
    }

    /// Get a refernce to the selection set's fields (not inline fragments, or
    /// fragment spreads).
    pub fn fields(&self) -> Vec<Field> {
        let fields: Vec<Field> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::Field(field) => return Some(field.as_ref().clone()),
                _ => None,
            })
            .collect();

        fields
    }

    /// Get a reference to selection set's fragment spread.
    pub fn fragment_spreads(&self) -> Vec<FragmentSpread> {
        let fragment_spread: Vec<FragmentSpread> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::FragmentSpread(fragment_spread) => {
                    return Some(fragment_spread.as_ref().clone())
                }
                _ => None,
            })
            .collect();

        fragment_spread
    }

    /// Get a reference to selection set's inline fragments.
    pub fn inline_fragments(&self) -> Vec<InlineFragment> {
        let inline_fragments: Vec<InlineFragment> = self
            .selection()
            .iter()
            .filter_map(|sel| match sel {
                Selection::InlineFragment(inline) => return Some(inline.as_ref().clone()),
                _ => None,
            })
            .collect();

        inline_fragments
    }

    /// Find a field a selection set.
    pub fn field(&self, name: &str) -> Option<&Field> {
        self.selection().iter().find_map(|sel| {
            if let Selection::Field(field) = sel {
                if field.name() == name {
                    return Some(field.as_ref());
                }
                None
            } else {
                None
            }
        })
    }

    /// Get all variables used in this selection set.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        /// Recursively collect used variables. Accounts for self-referential fragments.
        fn collect_used_variables(
            db: &dyn HirDatabase,
            set: &SelectionSet,
            seen_fragments: &mut HashSet<Arc<FragmentDefinition>>,
            output: &mut Vec<Variable>,
        ) {
            for selection in set.selection() {
                match selection {
                    Selection::Field(field) => {
                        output.extend(field.self_used_variables());
                        collect_used_variables(db, field.selection_set(), seen_fragments, output);
                    }
                    Selection::FragmentSpread(spread) => {
                        output.extend(spread.self_used_variables());

                        let Some(fragment) = spread.fragment(db) else {
                            return;
                        };
                        if seen_fragments.contains(&fragment) {
                            return; // prevent recursion loop
                        }
                        seen_fragments.insert(Arc::clone(&fragment));
                        collect_used_variables(
                            db,
                            fragment.selection_set(),
                            seen_fragments,
                            output,
                        );
                    }
                    Selection::InlineFragment(inline) => {
                        output.extend(inline.self_used_variables());
                        collect_used_variables(db, inline.selection_set(), seen_fragments, output);
                    }
                }
            }
        }

        let mut output = vec![];
        collect_used_variables(db, self, &mut HashSet::new(), &mut output);
        output
    }

    /// Returns true if all the [`Selection`]s in this selection set are themselves introspections.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        fn is_introspection_impl(
            db: &dyn HirDatabase,
            set: &SelectionSet,
            seen_fragments: &mut HashSet<Arc<FragmentDefinition>>,
        ) -> bool {
            set.selection().iter().all(|selection| match selection {
                Selection::Field(field) => field.is_introspection(),
                Selection::FragmentSpread(spread) => {
                    let maybe_fragment = spread.fragment(db);
                    maybe_fragment.map_or(false, |fragment| {
                        if seen_fragments.contains(&fragment) {
                            false
                        } else {
                            seen_fragments.insert(Arc::clone(&fragment));
                            is_introspection_impl(db, &fragment.selection_set, seen_fragments)
                        }
                    })
                }
                Selection::InlineFragment(inline) => {
                    is_introspection_impl(db, &inline.selection_set, seen_fragments)
                }
            })
        }

        is_introspection_impl(db, self, &mut HashSet::new())
    }

    /// Create a selection set for the concatenation of two selection sets' fields.
    ///
    /// This does not deduplicate fields: if the two selection sets both select a field `a`, the
    /// merged set will select field `a` twice.
    pub fn merge(&self, other: &SelectionSet) -> SelectionSet {
        let mut merged: Vec<Selection> =
            Vec::with_capacity(self.selection.len() + other.selection.len());
        merged.append(&mut self.selection.as_ref().clone());
        merged.append(&mut other.selection.as_ref().clone());

        SelectionSet {
            selection: Arc::new(merged),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Selection {
    Field(Arc<Field>),
    FragmentSpread(Arc<FragmentSpread>),
    InlineFragment(Arc<InlineFragment>),
}
impl Selection {
    /// Get variables used in the selection set.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        match self {
            Selection::Field(field) => field.variables(db),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.variables(db),
            Selection::InlineFragment(inline_fragment) => inline_fragment.variables(db),
        }
    }

    /// Returns `true` if the selection is [`Field`].
    ///
    /// [`Field`]: Selection::Field
    #[must_use]
    pub fn is_field(&self) -> bool {
        matches!(self, Self::Field(..))
    }

    /// Returns `true` if the selection is [`FragmentSpread`].
    ///
    /// [`FragmentSpread`]: Selection::FragmentSpread
    #[must_use]
    pub fn is_fragment_spread(&self) -> bool {
        matches!(self, Self::FragmentSpread(..))
    }

    /// Returns `true` if the selection is [`InlineFragment`].
    ///
    /// [`InlineFragment`]: Selection::InlineFragment
    #[must_use]
    pub fn is_inline_fragment(&self) -> bool {
        matches!(self, Self::InlineFragment(..))
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Selection::Field(field) => field.loc(),
            Selection::FragmentSpread(fragment_spread) => fragment_spread.loc(),
            Selection::InlineFragment(inline_fragment) => inline_fragment.loc(),
        }
    }
}

/// Represent both kinds of fragment selections: named and inline fragments.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum FragmentSelection {
    FragmentSpread(Arc<FragmentSpread>),
    InlineFragment(Arc<InlineFragment>),
}

impl FragmentSelection {
    /// Get the name of this fragment's type condition.
    ///
    /// This returns `None` on the following invalid inputs:
    /// - `self` is a named fragment spread, but the fragment it refers to is not defined
    /// - `self` is an inline fragment without an explicit type condition, used in a selection set
    ///   with a declared parent type that is not defined in the schema
    pub fn type_condition(&self, db: &dyn HirDatabase) -> Option<String> {
        match self {
            FragmentSelection::FragmentSpread(spread) => spread
                .fragment(db)
                .map(|frag| frag.type_condition().to_string()),
            FragmentSelection::InlineFragment(inline) => inline
                .type_condition()
                .or(inline.parent_obj.as_deref())
                .map(ToString::to_string),
        }
    }

    /// Get this fragment's selection set. This may be `None` if the fragment spread refers to an
    /// undefined fragment.
    pub fn selection_set(&self, db: &dyn HirDatabase) -> Option<SelectionSet> {
        match self {
            FragmentSelection::FragmentSpread(spread) => {
                spread.fragment(db).map(|frag| frag.selection_set().clone())
            }
            FragmentSelection::InlineFragment(inline) => Some(inline.selection_set().clone()),
        }
    }

    /// Get the type that this fragment is being spread onto.
    ///
    /// Returns `None` if the fragment is spread into a selection of an undefined field or type,
    /// like in:
    /// ```graphql
    /// type Query {
    ///   field: Int
    /// }
    /// query {
    ///   nonExistentField {
    ///     ... spreadToUnknownType
    ///   }
    /// }
    /// ```
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        match self {
            FragmentSelection::FragmentSpread(spread) => spread.parent_type(db),
            FragmentSelection::InlineFragment(inline) => inline.parent_type(db),
        }
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            FragmentSelection::FragmentSpread(fragment_spread) => fragment_spread.loc(),
            FragmentSelection::InlineFragment(inline_fragment) => inline_fragment.loc(),
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct InlineFragment {
    pub(crate) type_condition: Option<Name>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) parent_obj: Option<String>,
    pub(crate) loc: HirNodeLocation,
}

impl InlineFragment {
    /// Get a reference to inline fragment's type condition.
    pub fn type_condition(&self) -> Option<&str> {
        self.type_condition.as_ref().map(|t| t.src())
    }

    /// Get the type this fragment is spread onto.
    ///
    /// ## Examples
    /// ```graphql
    /// type Query {
    ///     field: X
    /// }
    /// query {
    ///     ... on Query { field } # spread A
    ///     field {
    ///         ... on X { subField } # spread B
    ///     }
    /// }
    /// ```
    /// `A.parent_type()` is `Query`.
    /// `B.parent_type()` is `X`.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Get inline fragments's type definition.
    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.type_condition()?.to_string())
    }

    /// Get a reference to inline fragment's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference inline fragment's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Return an iterator over the variables used in directives on this spread.
    ///
    /// Variables used *inside* the fragment are not included. For that, use
    /// [`variables()`][Self::variables].
    pub fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.directives()
            .iter()
            .flat_map(Directive::arguments)
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(var.clone()),
                _ => None,
            })
    }

    /// Get variables in use in the inline fragment.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.selection_set.variables(db)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if the inline fragment's [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        self.selection_set().is_introspection(db)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentSpread {
    pub(crate) name: Name,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) parent_obj: Option<String>,
    pub(crate) loc: HirNodeLocation,
}

impl FragmentSpread {
    /// Get a reference to the fragment spread's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the fragment definition this fragment spread is referencing.
    pub fn fragment(&self, db: &dyn HirDatabase) -> Option<Arc<FragmentDefinition>> {
        db.find_fragment_by_name(self.loc.file_id(), self.name().to_string())
    }

    /// Get the type this fragment is spread onto.
    ///
    /// ## Examples
    /// ```graphql
    /// type Query {
    ///     field: X
    /// }
    /// query {
    ///     ...fragment
    ///     field { ...subFragment }
    /// }
    /// ```
    /// `fragment.parent_type()` is `Query`.
    /// `subFragment.parent_type()` is `X`.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Return an iterator over the variables used in directives on this spread.
    ///
    /// Variables used by the fragment definition are not included. For that, use
    /// [`variables()`][Self::variables].
    pub fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.directives()
            .iter()
            .flat_map(Directive::arguments)
            .filter_map(|arg| match arg.value() {
                Value::Variable(var) => Some(var.clone()),
                _ => None,
            })
    }

    /// Get fragment spread's defined variables.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.fragment(db)
            .map(|fragment| fragment.variables(db))
            .unwrap_or_default()
    }

    /// Get a reference to fragment spread directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if the fragment referenced by this spread exists and its
    /// [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        let maybe_fragment = self.fragment(db);
        maybe_fragment.map_or(false, |fragment| {
            fragment.selection_set.is_introspection(db)
        })
    }
}
