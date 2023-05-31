use std::sync::Arc;

use crate::{
    hir::{Directive, HirNodeLocation, Name, SelectionSet, TypeDefinition, Variable},
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentDefinition {
    pub(crate) name: Name,
    pub(crate) type_condition: String,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

// NOTE @lrlna: all the getter methods here return the exact types that are
// stored in salsa's DB, Arc<>'s and all. In the long run, this should return
// the underlying values, as what's important is that the values are Arc<>'d in
// the database.
impl FragmentDefinition {
    /// Get a reference to the fragment definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> &Name {
        &self.name
    }

    /// Get a reference to fragment definition's type condition.
    pub fn type_condition(&self) -> &str {
        self.type_condition.as_ref()
    }

    /// Get fragment definition's directives.
    /// TODO: is this good??
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

    /// Get a reference to fragment definition's selection set.
    /// TODO: is this good??
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    // NOTE @lrlna: we will need to think and implement scope for fragment
    // definitions used/defined variables, as defined variables change based on
    // which operation definition the fragment is used in.

    /// Get variables used in a fragment definition.
    ///
    /// TODO(@goto-bus-stop): Maybe rename this to used_variables
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        self.selection_set.variables(db)
    }

    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.name().to_string())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Get the location information for the "head" of the fragment definition, namely the
    /// `fragment` keyword and the name.
    pub(crate) fn head_loc(&self) -> HirNodeLocation {
        self.name_src()
            .loc()
            .map(|name_loc| HirNodeLocation {
                // Adjust the node length to include the name
                node_len: name_loc.end_offset() - self.loc.offset(),
                ..self.loc
            })
            .unwrap_or(self.loc)
    }
}
