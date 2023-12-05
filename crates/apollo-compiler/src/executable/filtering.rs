use crate::ast;
use crate::executable::Field;
use crate::executable::Fragment;
use crate::executable::FragmentSpread;
use crate::executable::InlineFragment;
use crate::executable::Operation;
use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::schema;
use crate::schema::Name;
use crate::validation::SuspectedValidationBug;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Node;
use crate::Schema;
use indexmap::IndexMap;
use std::collections::HashSet;

type FragmentMap = IndexMap<Name, Node<Fragment>>;

pub(crate) struct FilteredDocumentBuilder<'doc, Predicate>
where
    Predicate: FnMut(&Selection) -> bool,
{
    document: &'doc Valid<ExecutableDocument>,
    remove_selection: Predicate,
    new_fragments: FragmentMap,

    /// The contents of these fragments was filtered to nothing.
    /// Corresonding fragment spreads should be removed.
    emptied_fragments: HashSet<&'doc Name>,

    /// Avoid infinite recursion
    fragments_being_processed: HashSet<&'doc Name>,

    /// Remove unused variables to satisfy the _All Variables Used_ validation rule.
    /// This feels like busy work. How important is it to produce a fully valid document?
    /// <https://spec.graphql.org/October2021/#sec-All-Variables-Used>
    variables_used: HashSet<&'doc Name>,
}

impl<'doc, Predicate> FilteredDocumentBuilder<'doc, Predicate>
where
    Predicate: FnMut(&Selection) -> bool,
{
    /// Return a document with exactly one operation,
    /// which is `operation` filtered according to `remove_selection`.
    ///
    /// If a non-empty selection set becomes empty, its parent is removed.
    /// Returns `None` if there is nothing left.
    ///
    /// The returned document also contains fragments needed by the remaining selections.
    /// Fragment definitions are filtered too.
    pub(crate) fn single_operation(
        schema: &Valid<Schema>,
        document: &'doc Valid<ExecutableDocument>,
        operation: &'doc Operation,
        remove_selection: Predicate,
    ) -> Result<Option<Valid<ExecutableDocument>>, SuspectedValidationBug> {
        let mut builder = Self {
            document,
            remove_selection,
            new_fragments: FragmentMap::new(),
            emptied_fragments: HashSet::new(),
            fragments_being_processed: HashSet::new(),
            variables_used: HashSet::new(),
        };
        let Some(new_operation) = builder.filter_operation(operation)? else {
            return Ok(None);
        };
        let mut new_document = ExecutableDocument {
            sources: document.sources.clone(),
            anonymous_operation: None,
            named_operations: IndexMap::new(),
            fragments: builder.new_fragments,
        };
        new_document.insert_operation(new_operation);
        let valid = if cfg!(debug_assertions) {
            new_document
                .validate(schema)
                .expect("filtering a valid document should result in a valid document")
        } else {
            Valid::assume_valid(new_document)
        };
        Ok(Some(valid))
    }

    fn filter_operation(
        &mut self,
        operation: &'doc Operation,
    ) -> Result<Option<Operation>, SuspectedValidationBug> {
        self.variables_used.clear();
        for var in &operation.variables {
            if let Some(default) = &var.default_value {
                self.variables_in_value(default)
            }
        }
        for directive in &operation.directives {
            for arg in &directive.arguments {
                self.variables_in_value(&arg.value)
            }
        }
        let Some(selection_set) = self.filter_selection_set(&operation.selection_set)? else {
            return Ok(None);
        };
        Ok(Some(Operation {
            operation_type: operation.operation_type,
            name: operation.name.clone(),
            variables: operation
                .variables
                .iter()
                .filter(|var| self.variables_used.contains(&var.name))
                .cloned()
                .collect(),
            directives: operation.directives.clone(),
            selection_set,
        }))
    }

    fn filter_selection_set(
        &mut self,
        selection_set: &'doc SelectionSet,
    ) -> Result<Option<SelectionSet>, SuspectedValidationBug> {
        let selections = selection_set
            .selections
            .iter()
            .filter_map(|selection| self.filter_selection(selection).transpose())
            .collect::<Result<Vec<_>, _>>()?;
        if !selections.is_empty() {
            Ok(Some(SelectionSet {
                ty: selection_set.ty.clone(),
                selections,
            }))
        } else {
            Ok(None)
        }
    }

    fn filter_selection(
        &mut self,
        selection: &'doc Selection,
    ) -> Result<Option<Selection>, SuspectedValidationBug> {
        if (self.remove_selection)(selection) {
            return Ok(None);
        }
        let new_selection = match selection {
            Selection::Field(field) => {
                let selection_set = if field.selection_set.selections.is_empty() {
                    // Keep a leaf field as-is
                    field.selection_set.clone()
                } else {
                    // `?` removes a non-leaf field if its sub-selections becomes empty
                    let Some(set) = self.filter_selection_set(&field.selection_set)? else {
                        return Ok(None);
                    };
                    set
                };
                for arg in &field.arguments {
                    self.variables_in_value(&arg.value)
                }
                Selection::Field(field.same_location(Field {
                    definition: field.definition.clone(),
                    alias: field.alias.clone(),
                    name: field.name.clone(),
                    arguments: field.arguments.clone(),
                    directives: field.directives.clone(),
                    selection_set,
                }))
            }
            Selection::InlineFragment(inline_fragment) => {
                let Some(selection_set) =
                    self.filter_selection_set(&inline_fragment.selection_set)?
                else {
                    return Ok(None);
                };
                Selection::InlineFragment(inline_fragment.same_location(InlineFragment {
                    type_condition: inline_fragment.type_condition.clone(),
                    directives: inline_fragment.directives.clone(),
                    selection_set,
                }))
            }
            Selection::FragmentSpread(fragment_spread) => {
                let name = &fragment_spread.fragment_name;
                if self.emptied_fragments.contains(name) {
                    return Ok(None);
                }
                if self.fragments_being_processed.contains(name) {
                    return Err(SuspectedValidationBug {
                        message: "fragment spread cycle".to_owned(),
                        location: fragment_spread.location(),
                    });
                }
                if !self.new_fragments.contains_key(name) {
                    let fragment_def = self.document.fragments.get(name).ok_or_else(|| {
                        SuspectedValidationBug {
                            message: "undefined fragment".to_owned(),
                            location: name.location(),
                        }
                    })?;

                    let Some(selection_set) =
                        self.filter_selection_set(&fragment_def.selection_set)?
                    else {
                        self.emptied_fragments.insert(name);
                        return Ok(None);
                    };
                    for directive in &fragment_def.directives {
                        for arg in &directive.arguments {
                            self.variables_in_value(&arg.value)
                        }
                    }
                    self.new_fragments.insert(
                        fragment_def.name.clone(),
                        fragment_def.same_location(Fragment {
                            name: fragment_def.name.clone(),
                            directives: fragment_def.directives.clone(),
                            selection_set,
                        }),
                    );
                }
                Selection::FragmentSpread(fragment_spread.same_location(FragmentSpread {
                    fragment_name: name.clone(),
                    directives: fragment_spread.directives.clone(),
                }))
            }
        };
        for directive in selection.directives() {
            for arg in &directive.arguments {
                self.variables_in_value(&arg.value)
            }
        }
        Ok(Some(new_selection))
    }

    fn variables_in_value(&mut self, value: &'doc ast::Value) {
        match value {
            schema::Value::Variable(name) => {
                self.variables_used.insert(name);
            }
            schema::Value::List(list) => {
                for value in list {
                    self.variables_in_value(value)
                }
            }
            schema::Value::Object(object) => {
                for (_name, value) in object {
                    self.variables_in_value(value)
                }
            }
            schema::Value::Null
            | schema::Value::Enum(_)
            | schema::Value::String(_)
            | schema::Value::Float(_)
            | schema::Value::Int(_)
            | schema::Value::Boolean(_) => {}
        }
    }
}
