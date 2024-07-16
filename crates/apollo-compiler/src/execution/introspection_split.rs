use crate::ast;
use crate::collections::HashSet;
use crate::collections::IndexMap;
use crate::executable::Field;
use crate::executable::Fragment;
use crate::executable::FragmentSpread;
use crate::executable::InlineFragment;
use crate::executable::Operation;
use crate::executable::OperationType;
use crate::executable::Selection;
use crate::executable::SelectionSet;
use crate::execution::GraphQLError;
use crate::execution::Response;
use crate::execution::SchemaIntrospectionQuery;
use crate::node::NodeLocation;
use crate::schema;
use crate::schema::Name;
use crate::validation::SuspectedValidationBug;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Node;
use crate::Schema;
use crate::SourceMap;
use indexmap::map::Entry;

/// Result of [`split`][Self::split]ting [schema introspection] fields from an operation.
///
/// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
pub enum SchemaIntrospectionSplit {
    /// The selected operation does *not* use [schema introspection] fields.
    /// It should be executed unchanged.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    None,

    /// The selected operation *only* uses [schema introspection] fields.
    /// This provides the [`execute`][SchemaIntrospectionQuery::execute]’able introspection query,
    /// there is nothing else to execute.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    Only(SchemaIntrospectionQuery),

    /// The selected operation uses *both* [schema introspection] fields and other fields.
    /// Each part should be executed, and their responses merged with [`Response::merge`].
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    Both {
        /// The [`execute`][SchemaIntrospectionQuery::execute]’able query
        /// for schema introspection parts of the original operation.
        introspection_query: SchemaIntrospectionQuery,

        /// The rest of the operation.
        ///
        /// This document contains exactly one operation with schema introspection fields removed,
        /// and the fragment definitions that are still needed.
        /// The operation definition name is preserved,
        /// so either `None` or the original `Option<&str>` name request can be passed
        /// to [`ExecutableDocument::get_operation`] to obtain the one operation.
        filtered_document: Valid<ExecutableDocument>,
    },
}

pub enum SchemaIntrospectionError {
    SuspectedValidationBug(SuspectedValidationBug),
    Unsupported {
        message: String,
        location: Option<NodeLocation>,
    },
}

impl SchemaIntrospectionSplit {
    /// Splits [schema introspection] fields from an operation.
    ///
    /// Returns either a split result, or a [request error] that should stop any further execution
    /// and can be converted with [`into_response`][SchemaIntrospectionError::into_response].
    ///
    /// In [the execution model described in the GrapQL specification][execution],
    /// a single server executes an entire operation by traversing its selections
    /// and calling resolver functions for each individual field.
    /// In this model, schema introspection is just another set of fields
    /// with dedicated resolver functions.
    ///
    /// In other models such as [Apollo Federation] there may not be an obvious place
    /// to “plug in” such resolvers. Instead, this function splits an operation
    /// into either or both introspection and other parts that can be executed separately.
    /// Full execution of introspection parts is provided by [`SchemaIntrospectionQuery::execute`].
    ///
    /// In an unconventional schema
    /// where the type of the `query` operation is also the type of some field,
    /// it is possible to use [schema introspection] fields nested in other fields.
    /// This function returns [`SchemaIntrospectionError::Unsupported`] for such operations,
    /// as they cannot be split into parts that have disjoint response keys.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    /// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
    /// [execution]: https://spec.graphql.org/October2021/#sec-Execution
    /// [Apollo Federation]: https://www.apollographql.com/docs/federation/
    pub fn split(
        schema: &Valid<Schema>,
        document: &Valid<ExecutableDocument>,
        operation: &Node<Operation>,
    ) -> Result<Self, SchemaIntrospectionError> {
        if operation.operation_type != OperationType::Query {
            check_non_query(document, operation)?;
            return Ok(Self::None);
        }

        let mut fragments_info = IndexMap::with_hasher(Default::default());
        let operation_field_kinds =
            collect_field_kinds(document, &mut fragments_info, &operation.selection_set)?;
        if operation_field_kinds.schema_introspection.is_none() {
            Ok(Self::None)
        } else if !operation_field_kinds.has_other_fields {
            // Clone unmodified operation
            let new_operation = operation.clone();
            // Clone unmodified fragments, but only the relevant ones
            let fragments = fragments_info
                .keys()
                .map(|&key| (key.clone(), document.fragments[key].clone()))
                .collect();
            let introspection_document =
                make_single_operation_document(schema, document, new_operation, fragments);
            Ok(Self::Only(SchemaIntrospectionQuery(introspection_document)))
        } else {
            let mut fragments_done = HashSet::with_hasher(Default::default());
            let mut new_documents = Split {
                introspection: DocumentBuilder::new(document, operation),
                other: DocumentBuilder::new(document, operation),
            };
            let operation_selection_set = split_selection_set(
                &mut fragments_done,
                &mut new_documents,
                &operation.selection_set,
            );
            Ok(Self::Both {
                introspection_query: SchemaIntrospectionQuery(new_documents.introspection.build(
                    schema,
                    document,
                    operation_selection_set.introspection,
                )),
                filtered_document: new_documents.other.build(
                    schema,
                    document,
                    operation_selection_set.other,
                ),
            })
        }
    }
}

fn field_is_schema_introspection(field: &Field) -> bool {
    field.name == "__schema" || field.name == "__type"
}

/// Returns an error if schema introspection is used anywhere.
/// Called for a mutation or subscription operation.
fn check_non_query<'doc>(
    document: &'doc Valid<ExecutableDocument>,
    operation: &'doc Operation,
) -> Result<(), SchemaIntrospectionError> {
    fn check_selection_set<'doc>(
        fragments_visited: &mut HashSet<&'doc Name>,
        fragments_to_visit: &mut HashSet<&'doc Name>,
        selection_set: &'doc SelectionSet,
    ) -> Result<(), &'doc Node<Field>> {
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    if field_is_schema_introspection(field) {
                        return Err(field);
                    }
                    check_selection_set(
                        fragments_visited,
                        fragments_to_visit,
                        &field.selection_set,
                    )?
                }
                Selection::InlineFragment(inline_fragment) => check_selection_set(
                    fragments_visited,
                    fragments_to_visit,
                    &inline_fragment.selection_set,
                )?,
                Selection::FragmentSpread(fragment_spread) => {
                    let name = &fragment_spread.fragment_name;
                    if !fragments_visited.contains(name) {
                        fragments_to_visit.insert(name);
                    }
                }
            }
        }
        Ok(())
    }
    let unsupported = |field: &Node<Field>| SchemaIntrospectionError::Unsupported {
        message: format!(
            "Schema introspection field {} is not supported in a {} operation",
            field.name, operation.operation_type,
        ),
        location: field.location(),
    };
    let mut fragments_visited = HashSet::with_hasher(Default::default());
    let mut fragments_to_visit = HashSet::with_hasher(Default::default());
    check_selection_set(
        &mut fragments_visited,
        &mut fragments_to_visit,
        &operation.selection_set,
    )
    .map_err(unsupported)?;
    while let Some(name) = fragments_to_visit.iter().next().copied() {
        let fragment_def = get_fragment(document, name)?;
        check_selection_set(
            &mut fragments_visited,
            &mut fragments_to_visit,
            &fragment_def.selection_set,
        )
        .map_err(unsupported)?;
        fragments_to_visit.remove(name);
        fragments_visited.insert(name);
    }
    Ok(())
}

/// As found in `ExecutableDocument::fragments`
type FragmentMap = IndexMap<Name, Node<Fragment>>;

/// The given operation and fragments are expected to form a valid document.
/// This is checked iff debug assertions are enabled.
fn make_single_operation_document(
    schema: &Valid<Schema>,
    document: &Valid<ExecutableDocument>,
    new_operation: Node<Operation>,
    fragments: FragmentMap,
) -> Valid<ExecutableDocument> {
    let mut new_document = ExecutableDocument {
        sources: document.sources.clone(),
        operations: Default::default(),
        fragments,
    };
    new_document.operations.insert(new_operation);
    if cfg!(debug_assertions) {
        new_document
            .validate(schema)
            .expect("filtering a valid document should result in a valid document")
    } else {
        Valid::assume_valid(new_document)
    }
}

fn get_fragment<'doc>(
    document: &'doc Valid<ExecutableDocument>,
    name: &Name,
) -> Result<&'doc Node<Fragment>, SchemaIntrospectionError> {
    document.fragments.get(name).ok_or_else(|| {
        SuspectedValidationBug {
            message: format!("undefined fragment {name}"),
            location: name.location(),
        }
        .into()
    })
}

/// Which kinds of fields are reachable from a given selection set?
/// Either directly or through (inline or named) fragments, but not through nested fields.
#[derive(Clone, Copy, Default)]
struct TopLevelFieldKinds<'doc> {
    schema_introspection: Option<&'doc Node<Field>>,
    has_other_fields: bool,
}

impl std::ops::BitOrAssign for TopLevelFieldKinds<'_> {
    fn bitor_assign(&mut self, rhs: Self) {
        if self.schema_introspection.is_none() {
            self.schema_introspection = rhs.schema_introspection
        }
        self.has_other_fields |= rhs.has_other_fields
    }
}

enum Computation<T> {
    Ongoing,
    Done(T),
}

/// This function has triple purpose:
///
/// * Return which kinds of fields are used at the “response top-level” of a selection set.
///   That is, inline fragments and fragment spreads are considered same-level,
///   but sub-selections of non-leaf fields are not.
///   For the root selection set of the operation,
///   this determines which `SchemaIntrospectionSplit` variant `split` returns.
///
/// * Populate the `fragments` hash map so it has a key
///   for every fragment reachable from this selection set,
///   including through other fragments.
///   Map values are used for the first purpose, but `SchemaIntrospectionSplit::split`
///   also relies on key presense.
///
/// * Return an "unsupported" error if schema introspection are used nested in other fields.
fn collect_field_kinds<'doc>(
    document: &'doc Valid<ExecutableDocument>,
    fragments: &mut IndexMap<&'doc Name, Computation<TopLevelFieldKinds<'doc>>>,
    selection_set: &'doc SelectionSet,
) -> Result<TopLevelFieldKinds<'doc>, SchemaIntrospectionError> {
    let mut top_level_field_kinds = TopLevelFieldKinds::default();
    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                let nested_field_kinds =
                    collect_field_kinds(document, fragments, &field.selection_set)?;
                if field_is_schema_introspection(field) {
                    top_level_field_kinds
                        .schema_introspection
                        .get_or_insert(field);
                    // `nested_field_kinds` not used here but the recursive call above
                    // still populate `fragments` with reachable fragments.
                } else {
                    if let Some(schema_introspection_field) =
                        nested_field_kinds.schema_introspection
                    {
                        return Err(SchemaIntrospectionError::Unsupported {
                            message: format!(
                                "Schema introspection field {} is not supported \
                                 nested in other fields",
                                schema_introspection_field.name
                            ),
                            location: schema_introspection_field.location(),
                        });
                    }
                    top_level_field_kinds.has_other_fields = true;
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                top_level_field_kinds |=
                    collect_field_kinds(document, fragments, &inline_fragment.selection_set)?;
            }
            Selection::FragmentSpread(fragment_spread) => {
                let fragment_def = get_fragment(document, &fragment_spread.fragment_name)?;
                let name = &fragment_def.name; // with location at definition, not spread
                match fragments.entry(name) {
                    Entry::Occupied(entry) => match entry.get() {
                        Computation::Ongoing => {
                            return Err(SuspectedValidationBug {
                                message: "fragment cycle".to_owned(),
                                location: name.location(),
                            }
                            .into());
                        }
                        Computation::Done(fragment_field_kinds) => {
                            top_level_field_kinds |= *fragment_field_kinds
                        }
                    },
                    Entry::Vacant(entry) => {
                        entry.insert(Computation::Ongoing);
                        let fragment_field_kinds =
                            collect_field_kinds(document, fragments, &fragment_def.selection_set)?;
                        fragments.insert(name, Computation::Done(fragment_field_kinds));
                        top_level_field_kinds |= fragment_field_kinds
                    }
                }
            }
        }
    }
    Ok(top_level_field_kinds)
}

#[derive(Default)]
struct Split<T> {
    introspection: T,
    other: T,
}

struct DocumentBuilder<'doc> {
    original_document: &'doc Valid<ExecutableDocument>,
    original_operation: &'doc Node<Operation>,
    variables_used: HashSet<&'doc Name>,
    new_fragments: FragmentMap,
}

fn split_selection_set<'doc>(
    fragments_done: &mut HashSet<&'doc Name>,
    new_documents: &mut Split<DocumentBuilder<'doc>>,
    selection_set: &'doc SelectionSet,
) -> Split<SelectionSet> {
    let mut new_selection_sets = Split {
        introspection: SelectionSet::new(selection_set.ty.clone()),
        other: SelectionSet::new(selection_set.ty.clone()),
    };
    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                // A field’s sub-selections are not top-level and therefore don’t need to be split.
                // Clone as-is, and visit to collect fragment definitions and variables used.
                if field_is_schema_introspection(field) {
                    new_selection_sets.introspection.push(field.clone());
                    new_documents.introspection.visit_field(field);
                } else {
                    new_selection_sets.other.push(field.clone());
                    new_documents.other.visit_field(field);
                }
            }
            Selection::InlineFragment(inline_fragment) => {
                // Add an inline fragment if the split nested selection set is non-empty
                let if_non_empty = |doc: &mut DocumentBuilder<'doc>,
                                    parent: &mut SelectionSet,
                                    nested: SelectionSet| {
                    if !nested.selections.is_empty() {
                        doc.visit_directives(&inline_fragment.directives);
                        parent.push(inline_fragment.same_location(InlineFragment {
                            type_condition: inline_fragment.type_condition.clone(),
                            directives: inline_fragment.directives.clone(),
                            selection_set: nested,
                        }))
                    }
                };
                let nested = split_selection_set(
                    // document,
                    fragments_done,
                    new_documents,
                    &inline_fragment.selection_set,
                );
                if_non_empty(
                    &mut new_documents.introspection,
                    &mut new_selection_sets.introspection,
                    nested.introspection,
                );
                if_non_empty(
                    &mut new_documents.other,
                    &mut new_selection_sets.other,
                    nested.other,
                );
            }
            Selection::FragmentSpread(fragment_spread) => {
                let name = &fragment_spread.fragment_name;
                let new = fragments_done.insert(name);
                if new {
                    let document = &new_documents.introspection.original_document;
                    // Checked in `collect_field_kinds`
                    let fragment_def = &document.fragments[name];
                    // Add a fragment definition if the split selection set is non-empty
                    let if_non_empty = |doc: &mut DocumentBuilder<'doc>, nested: SelectionSet| {
                        if !nested.selections.is_empty() {
                            doc.visit_directives(&fragment_def.directives);
                            doc.new_fragments.insert(
                                fragment_def.name.clone(),
                                fragment_def.same_location(Fragment {
                                    name: fragment_def.name.clone(),
                                    directives: fragment_def.directives.clone(),
                                    selection_set: nested,
                                }),
                            );
                        }
                    };
                    let nested = split_selection_set(
                        fragments_done,
                        new_documents,
                        &fragment_def.selection_set,
                    );
                    if_non_empty(&mut new_documents.introspection, nested.introspection);
                    if_non_empty(&mut new_documents.other, nested.other);
                }
                // Add a fragment spread if the above resulted in a fragment definition
                let if_defined = |doc: &mut DocumentBuilder<'doc>, parent: &mut SelectionSet| {
                    if doc.new_fragments.contains_key(name) {
                        doc.visit_directives(&fragment_spread.directives);
                        parent.push(fragment_spread.same_location(FragmentSpread {
                            fragment_name: fragment_spread.fragment_name.clone(),
                            directives: fragment_spread.directives.clone(),
                        }))
                    }
                };
                if_defined(
                    &mut new_documents.introspection,
                    &mut new_selection_sets.introspection,
                );
                if_defined(&mut new_documents.other, &mut new_selection_sets.other);
            }
        }
    }
    new_selection_sets
}

impl<'doc> DocumentBuilder<'doc> {
    fn new(
        original_document: &'doc Valid<ExecutableDocument>,
        original_operation: &'doc Node<Operation>,
    ) -> Self {
        Self {
            original_document,
            original_operation,
            variables_used: Default::default(),
            new_fragments: Default::default(),
        }
    }

    fn visit_selection_set(&mut self, selection_set: &'doc SelectionSet) {
        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => self.visit_field(field),
                Selection::InlineFragment(inline_fragment) => {
                    self.visit_directives(&inline_fragment.directives);
                    self.visit_selection_set(&inline_fragment.selection_set);
                }
                Selection::FragmentSpread(fragment_spread) => {
                    self.visit_directives(&fragment_spread.directives);
                    let name = &fragment_spread.fragment_name;
                    if let Entry::Vacant(entry) = self.new_fragments.entry(name.clone()) {
                        // Checked in `collect_field_kinds`
                        let fragment_def = &self.original_document.fragments[name];
                        entry.insert(fragment_def.clone());
                        self.visit_directives(&fragment_def.directives);
                        self.visit_selection_set(&fragment_def.selection_set);
                    };
                }
            }
        }
    }

    fn visit_field(&mut self, field: &'doc Field) {
        for arg in &field.arguments {
            self.visit_value(&arg.value)
        }
        self.visit_directives(&field.directives);
        self.visit_selection_set(&field.selection_set);
    }

    fn visit_directives(&mut self, directives: &'doc ast::DirectiveList) {
        for directive in directives {
            for arg in &directive.arguments {
                self.visit_value(&arg.value)
            }
        }
    }

    fn visit_value(&mut self, value: &'doc ast::Value) {
        match value {
            schema::Value::Variable(name) => {
                self.variables_used.insert(name);
            }
            schema::Value::List(list) => {
                for value in list {
                    self.visit_value(value)
                }
            }
            schema::Value::Object(object) => {
                for (_name, value) in object {
                    self.visit_value(value)
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

    fn build(
        mut self,
        schema: &Valid<Schema>,
        document: &Valid<ExecutableDocument>,
        operation_selection_set: SelectionSet,
    ) -> Valid<ExecutableDocument> {
        for directive in &self.original_operation.directives {
            for arg in &directive.arguments {
                self.visit_value(&arg.value)
            }
        }
        let new_operation = self.original_operation.same_location(Operation {
            operation_type: self.original_operation.operation_type,
            name: self.original_operation.name.clone(),
            variables: self
                .original_operation
                .variables
                .iter()
                .filter(|var| self.variables_used.contains(&var.name))
                .cloned()
                .collect(),
            directives: self.original_operation.directives.clone(),
            selection_set: operation_selection_set,
        });
        make_single_operation_document(schema, document, new_operation, self.new_fragments)
    }
}

impl From<SuspectedValidationBug> for SchemaIntrospectionError {
    fn from(value: SuspectedValidationBug) -> Self {
        Self::SuspectedValidationBug(value)
    }
}

impl SchemaIntrospectionError {
    /// Convert into a JSON-serializable error as represented in a GraphQL response
    pub fn into_graphql_error(self, sources: &SourceMap) -> GraphQLError {
        match self {
            Self::SuspectedValidationBug(s) => s.into_graphql_error(sources),
            Self::Unsupported { message, location } => {
                GraphQLError::new(message, location, sources)
            }
        }
    }

    /// Convert into a response with this error as a [request error]
    /// that prevented execution from starting.
    ///
    /// [request error]: https://spec.graphql.org/October2021/#sec-Errors.Request-errors
    pub fn into_response(self, sources: &SourceMap) -> Response {
        Response::from_request_error(self.into_graphql_error(sources))
    }
}
