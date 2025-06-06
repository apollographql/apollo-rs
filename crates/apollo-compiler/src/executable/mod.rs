//! High-level representation of an executable document,
//! which can contain operations and fragments.
//!
//! Compared to an [`ast::Document`] which follows closely the structure of GraphQL syntax,
//! an [`ExecutableDocument`] interpreted in the context of a valid [`Schema`]
//! and contains typing information.
//!
//! In some cases like [`SelectionSet`], this module and the [`ast`] module
//! define different Rust types with the same names.
//! In other cases like [`Directive`] there is no data structure difference needed,
//! so this module reuses and publicly re-exports some Rust types from the [`ast`] module.
//!
//! ## “Build” errors
//!
//! As a result of how `ExecutableDocument` containing typing information,
//! not all AST documents (even if filtering out type system definitions) can be fully represented:
//! creating a `ExecutableDocument` can cause errors (on top of any potential syntax error)
//! for cases like selecting a field not defined in the schema.
//!
//! When such errors (or in [`ExecutableDocument::parse`], syntax errors) happen,
//! a partial document is returned together with a list of diagnostics.
//!
//! ## Structural sharing and mutation
//!
//! Like in AST, many parts of a `ExecutableDocument` are reference-counted with [`Node`].
//! This allows sharing nodes between documents without cloning entire subtrees.
//! To modify a node, the [`make_mut`][Node::make_mut] method provides copy-on-write semantics.
//!
//! ## Validation
//!
//! The [Validation] section of the GraphQL specification defines validation rules
//! beyond syntax errors and errors detected while constructing a `ExecutableDocument`.
//! The [`validate`][ExecutableDocument::validate] method returns either:
//!
//! * An immutable [`Valid<ExecutableDocument>`] type wrapper, or
//! * The document together with a list of diagnostics
//!
//! If there is no mutation needed between parsing and validation,
//! [`ExecutableDocument::parse_and_validate`] does both in one step.
//!
//! [Validation]: https://spec.graphql.org/draft/#sec-Validation
//!
//! ## Serialization
//!
//! `ExecutableDocument` and other types types implement [`Display`][std::fmt::Display]
//! and [`ToString`] by serializing to GraphQL syntax with a default configuration.
//! [`serialize`][ExecutableDocument::serialize] methods return a builder
//! that has chaining methods for setting serialization configuration,
//! and also implements `Display` and `ToString`.

use crate::ast;
use crate::collections::IndexMap;
use crate::coordinate::FieldArgumentCoordinate;
use crate::coordinate::TypeAttributeCoordinate;
use crate::parser::Parser;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::schema;
use crate::validation::DiagnosticList;
use crate::validation::Valid;
use crate::validation::WithErrors;
use crate::Node;
use crate::Schema;
use indexmap::map::Entry;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

pub(crate) mod from_ast;
mod serialize;
pub(crate) mod validation;

pub use crate::ast::Argument;
use crate::ast::ArgumentByNameError;
pub use crate::ast::Directive;
pub use crate::ast::DirectiveList;
pub use crate::ast::NamedType;
pub use crate::ast::OperationType;
pub use crate::ast::Type;
pub use crate::ast::Value;
pub use crate::ast::VariableDefinition;
use crate::collections::HashSet;
use crate::request::RequestError;
pub use crate::Name;

/// Executable definitions, annotated with type information
#[derive(Debug, Clone, Default)]
pub struct ExecutableDocument {
    /// If this document was originally parsed from a source file,
    /// this map contains one entry for that file and its ID.
    ///
    /// The document may have been modified since.
    pub sources: SourceMap,

    pub operations: OperationMap,
    pub fragments: FragmentMap,
}

/// Operations definitions for a given executable document
#[derive(Debug, Clone, Default, PartialEq)]
pub struct OperationMap {
    pub anonymous: Option<Node<Operation>>,
    pub named: IndexMap<Name, Node<Operation>>,
}

/// Definitions of named fragments for a given executable document
pub type FragmentMap = IndexMap<Name, Node<Fragment>>;

/// FieldSet information created for FieldSet parsing in `@requires` directive.
/// Annotated with type information.
#[derive(Debug, Clone)]
pub struct FieldSet {
    /// If this document was originally parsed from a source file,
    /// this map contains one entry for that file and its ID.
    ///
    /// The document may have been modified since.
    pub sources: SourceMap,

    pub selection_set: SelectionSet,
}

/// An [_OperationDefinition_](https://spec.graphql.org/draft/#OperationDefinition)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub operation_type: OperationType,
    pub name: Option<Name>,
    pub variables: Vec<Node<VariableDefinition>>,
    pub directives: DirectiveList,
    pub selection_set: SelectionSet,
}

/// A [_FragmentDefinition_](https://spec.graphql.org/draft/#FragmentDefinition)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub name: Name,
    pub directives: DirectiveList,
    pub selection_set: SelectionSet,
}

/// A [_SelectionSet_](https://spec.graphql.org/draft/#SelectionSet)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectionSet {
    pub ty: NamedType,
    pub selections: Vec<Selection>,
}

/// A [_Selection_](https://spec.graphql.org/draft/#Selection)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selection {
    Field(Node<Field>),
    FragmentSpread(Node<FragmentSpread>),
    InlineFragment(Node<InlineFragment>),
}

/// A [_Field_](https://spec.graphql.org/draft/#Field) selection,
/// linked to the corresponding field definition in the schema.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    /// The definition of this field in an object type or interface type definition in the schema
    pub definition: Node<schema::FieldDefinition>,
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
    pub directives: DirectiveList,
    pub selection_set: SelectionSet,
}

/// A [_FragmentSpread_](https://spec.graphql.org/draft/#FragmentSpread)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: DirectiveList,
}

/// A [_InlineFragment_](https://spec.graphql.org/draft/#InlineFragment)
/// annotated with type information.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: DirectiveList,
    pub selection_set: SelectionSet,
}

/// Errors that can occur during conversion from AST to executable document or
/// validation of an executable document.
#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum BuildError {
    #[error("an executable document must not contain {describe}")]
    TypeSystemDefinition {
        name: Option<Name>,
        describe: &'static str,
    },

    #[error("anonymous operation cannot be selected when the document contains other operations")]
    AmbiguousAnonymousOperation,

    #[error(
        "the operation `{name_at_previous_location}` is defined multiple times in the document"
    )]
    OperationNameCollision { name_at_previous_location: Name },

    #[error(
        "the fragment `{name_at_previous_location}` is defined multiple times in the document"
    )]
    FragmentNameCollision { name_at_previous_location: Name },

    #[error("`{operation_type}` root operation type is not defined")]
    UndefinedRootOperation { operation_type: &'static str },

    #[error(
        "type condition `{type_name}` of fragment `{fragment_name}` \
         is not a type defined in the schema"
    )]
    UndefinedTypeInNamedFragmentTypeCondition {
        type_name: NamedType,
        fragment_name: Name,
    },

    #[error("type condition `{type_name}` of inline fragment is not a type defined in the schema")]
    UndefinedTypeInInlineFragmentTypeCondition {
        type_name: NamedType,
        path: SelectionPath,
    },

    #[error("field selection of scalar type `{type_name}` must not have subselections")]
    SubselectionOnScalarType {
        type_name: NamedType,
        path: SelectionPath,
    },

    #[error("field selection of enum type `{type_name}` must not have subselections")]
    SubselectionOnEnumType {
        type_name: NamedType,
        path: SelectionPath,
    },

    #[error("type `{type_name}` does not have a field `{field_name}`")]
    UndefinedField {
        type_name: NamedType,
        field_name: Name,
        path: SelectionPath,
    },

    // Validation errors
    #[error(
        "{} can only have one root field",
        subscription_name_or_anonymous(name)
    )]
    SubscriptionUsesMultipleFields {
        name: Option<Name>,
        fields: Vec<Name>,
    },

    #[error(
        "{} can not have an introspection field as a root field",
        subscription_name_or_anonymous(name)
    )]
    SubscriptionUsesIntrospection {
        /// Name of the operation
        name: Option<Name>,
        /// Name of the introspection field
        field: Name,
    },
    #[error(
        "{} can not specify @skip or @include on root fields",
        subscription_name_or_anonymous(name)
    )]
    SubscriptionUsesConditionalSelection {
        /// Name of the operation
        name: Option<Name>,
    },

    #[error("{0}")]
    ConflictingFieldType(Box<ConflictingFieldType>),
    #[error("{0}")]
    ConflictingFieldArgument(Box<ConflictingFieldArgument>),
    #[error("{0}")]
    ConflictingFieldName(Box<ConflictingFieldName>),
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("operation must not select different types using the same name `{alias}`")]
pub(crate) struct ConflictingFieldType {
    /// Name or alias of the non-unique field.
    pub(crate) alias: Name,
    pub(crate) original_location: Option<SourceSpan>,
    pub(crate) original_coordinate: TypeAttributeCoordinate,
    pub(crate) original_type: Type,
    pub(crate) conflicting_location: Option<SourceSpan>,
    pub(crate) conflicting_coordinate: TypeAttributeCoordinate,
    pub(crate) conflicting_type: Type,
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("operation must not provide conflicting field arguments for the same name `{alias}`")]
pub(crate) struct ConflictingFieldArgument {
    /// Name or alias of the non-unique field.
    pub(crate) alias: Name,
    pub(crate) original_location: Option<SourceSpan>,
    pub(crate) original_coordinate: FieldArgumentCoordinate,
    pub(crate) original_value: Option<Value>,
    pub(crate) conflicting_location: Option<SourceSpan>,
    pub(crate) conflicting_coordinate: FieldArgumentCoordinate,
    pub(crate) conflicting_value: Option<Value>,
}

#[derive(thiserror::Error, Debug, Clone)]
#[error("cannot select different fields into the same alias `{alias}`")]
pub(crate) struct ConflictingFieldName {
    /// Name of the non-unique field.
    pub(crate) alias: Name,
    pub(crate) original_location: Option<SourceSpan>,
    pub(crate) original_selection: TypeAttributeCoordinate,
    pub(crate) conflicting_location: Option<SourceSpan>,
    pub(crate) conflicting_selection: TypeAttributeCoordinate,
}

fn subscription_name_or_anonymous(name: &Option<Name>) -> impl std::fmt::Display + '_ {
    crate::validation::diagnostics::NameOrAnon {
        name: name.as_ref(),
        if_some_prefix: "subscription",
        if_none: "anonymous subscription",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectionPath {
    pub(crate) root: ExecutableDefinitionName,
    pub(crate) nested_fields: Vec<Name>,
}

/// Designates by name a top-level definition in an executable document
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ExecutableDefinitionName {
    AnonymousOperation(ast::OperationType),
    NamedOperation(ast::OperationType, Name),
    Fragment(Name),
}

impl ExecutableDocument {
    /// Create an empty document, to be filled programatically
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse an executable document with the default configuration.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn parse(
        schema: &Valid<Schema>,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, WithErrors<Self>> {
        Parser::new().parse_executable(schema, source_text, path)
    }

    /// [`parse`][Self::parse] then [`validate`][Self::validate],
    /// to get a `Valid<ExecutableDocument>` when mutating it isn’t needed.
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn parse_and_validate(
        schema: &Valid<Schema>,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Valid<Self>, WithErrors<Self>> {
        let (doc, mut errors) = Parser::new().parse_executable_inner(schema, source_text, path);
        Arc::make_mut(&mut errors.sources)
            .extend(schema.sources.iter().map(|(k, v)| (*k, v.clone())));
        validation::validate_executable_document(&mut errors, schema, &doc);
        errors.into_valid_result(doc)
    }

    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn validate(self, schema: &Valid<Schema>) -> Result<Valid<Self>, WithErrors<Self>> {
        let mut sources = IndexMap::clone(&schema.sources);
        sources.extend(self.sources.iter().map(|(k, v)| (*k, v.clone())));
        let mut errors = DiagnosticList::new(Arc::new(sources));
        validation::validate_executable_document(&mut errors, schema, &self);
        errors.into_valid_result(self)
    }

    serialize_method!();
}

impl Eq for ExecutableDocument {}

/// `sources` and `build_errors` are ignored for comparison
impl PartialEq for ExecutableDocument {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            sources: _,
            operations,
            fragments,
        } = self;
        *operations == other.operations && *fragments == other.fragments
    }
}

impl OperationMap {
    /// Creates a new `OperationMap` containing one operation
    pub fn from_one(operation: impl Into<Node<Operation>>) -> Self {
        let mut map = Self::default();
        map.insert(operation);
        map
    }

    pub fn is_empty(&self) -> bool {
        self.anonymous.is_none() && self.named.is_empty()
    }

    pub fn len(&self) -> usize {
        self.anonymous.is_some() as usize + self.named.len()
    }

    /// Returns an iterator of operations, both anonymous and named
    pub fn iter(&self) -> impl Iterator<Item = &'_ Node<Operation>> {
        self.anonymous
            .as_ref()
            .into_iter()
            .chain(self.named.values())
    }

    /// Return the relevant operation for a request, or a request error
    ///
    /// This is the [_GetOperation()_](https://spec.graphql.org/October2021/#GetOperation())
    /// algorithm in the _Executing Requests_ section of the specification.
    ///
    /// A GraphQL request comes with a document (which may contain multiple operations)
    /// an an optional operation name. When a name is given the request executes the operation
    /// with that name, which is expected to exist. When it is not given / null / `None`,
    /// the document is expected to contain a single operation (which may or may not be named)
    /// to avoid ambiguity.
    pub fn get(&self, name_request: Option<&str>) -> Result<&Node<Operation>, RequestError> {
        if let Some(name) = name_request {
            // Honor the request
            self.named
                .get(name)
                .ok_or_else(|| format!("No operation named '{name}'"))
        } else {
            // No name request (`operationName` unspecified or null)
            if let Some(op) = &self.anonymous {
                // Return the anonymous operation if it’s the only operation
                self.named.is_empty().then_some(op)
            } else {
                // No anonymous operation, return a named operation if it’s the only one
                self.named
                    .values()
                    .next()
                    .and_then(|op| (self.named.len() == 1).then_some(op))
            }
            .ok_or_else(|| {
                "Ambiguous request: multiple operations but no specified `operationName`".to_owned()
            })
        }
        .map_err(|message| RequestError {
            message,
            location: None,
            is_suspected_validation_bug: false,
        })
    }

    /// Similar to [`get`][Self::get] but returns a mutable reference.
    pub fn get_mut(&mut self, name_request: Option<&str>) -> Result<&mut Operation, RequestError> {
        if let Some(name) = name_request {
            // Honor the request
            self.named
                .get_mut(name)
                .ok_or_else(|| format!("No operation named '{name}'"))
        } else {
            // No name request (`operationName` unspecified or null)
            if let Some(op) = &mut self.anonymous {
                // Return the anonymous operation if it’s the only operation
                self.named.is_empty().then_some(op)
            } else {
                // No anonymous operation, return a named operation if it’s the only one
                let len = self.named.len();
                self.named
                    .values_mut()
                    .next()
                    .and_then(|op| (len == 1).then_some(op))
            }
            .ok_or_else(|| {
                "Ambiguous request: multiple operations but no specified `operationName`".to_owned()
            })
        }
        .map(Node::make_mut)
        .map_err(|message| RequestError {
            message,
            location: None,
            is_suspected_validation_bug: false,
        })
    }

    /// Insert the given operation in either `named_operations` or `anonymous_operation`
    /// as appropriate, and return the old operation (if any) with that name (or lack thereof).
    pub fn insert(&mut self, operation: impl Into<Node<Operation>>) -> Option<Node<Operation>> {
        let operation = operation.into();
        if let Some(name) = &operation.name {
            self.named.insert(name.clone(), operation)
        } else {
            self.anonymous.replace(operation)
        }
    }
}

impl Operation {
    /// Returns the name of the schema type this operation selects against.
    pub fn object_type(&self) -> &NamedType {
        &self.selection_set.ty
    }

    /// Returns true if this is a query operation.
    pub fn is_query(&self) -> bool {
        self.operation_type == OperationType::Query
    }

    /// Returns true if this is a mutation operation.
    pub fn is_mutation(&self) -> bool {
        self.operation_type == OperationType::Mutation
    }

    /// Returns true if this is a subscription operation.
    pub fn is_subscription(&self) -> bool {
        self.operation_type == OperationType::Subscription
    }

    /// Return whether this operation is a query that only selects introspection meta-fields:
    /// `__type`, `__schema`, and `__typename`
    pub fn is_introspection(&self, document: &ExecutableDocument) -> bool {
        self.is_query()
            && self
                .root_fields(document)
                .all(|field| matches!(field.name.as_str(), "__type" | "__schema" | "__typename"))
    }

    /// Returns an iterator of field selections that are at the root of the response.
    /// That is, inline fragments and fragment spreads at the root are traversed,
    /// but field sub-selections are not.
    ///
    /// See also [`all_fields`][Self::all_fields].
    ///
    /// `document` is used to look up fragment definitions.
    ///
    /// This does **not** perform [field merging],
    /// so multiple items in this iterator may have the same response key
    /// or point to the same field definition.
    /// Named fragments however are only traversed once even if spread multiple times.
    ///
    /// [field merging]: https://spec.graphql.org/draft/#sec-Field-Selection-Merging
    pub fn root_fields<'doc>(
        &'doc self,
        document: &'doc ExecutableDocument,
    ) -> impl Iterator<Item = &'doc Node<Field>> {
        self.selection_set.root_fields(document)
    }

    /// Returns an iterator of all field selections in this operation.
    ///
    /// See also [`root_fields`][Self::root_fields].
    ///
    /// `document` is used to look up fragment definitions.
    ///
    /// This does **not** perform [field merging],
    /// so multiple items in this iterator may have the same response key
    /// or point to the same field definition.
    /// Named fragments however are only traversed once even if spread multiple times.
    ///
    /// [field merging]: https://spec.graphql.org/draft/#sec-Field-Selection-Merging
    pub fn all_fields<'doc>(
        &'doc self,
        document: &'doc ExecutableDocument,
    ) -> impl Iterator<Item = &'doc Node<Field>> {
        self.selection_set.all_fields(document)
    }

    serialize_method!();
}

impl Fragment {
    pub fn type_condition(&self) -> &NamedType {
        &self.selection_set.ty
    }

    serialize_method!();
}

impl SelectionSet {
    /// Create a new selection set
    pub fn new(ty: NamedType) -> Self {
        Self {
            ty,
            selections: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.selections.is_empty()
    }

    pub fn push(&mut self, selection: impl Into<Selection>) {
        self.selections.push(selection.into())
    }

    pub fn extend(&mut self, selections: impl IntoIterator<Item = impl Into<Selection>>) {
        self.selections
            .extend(selections.into_iter().map(|sel| sel.into()))
    }

    /// Create a new field to be added to this selection set with [`push`][Self::push]
    ///
    /// Returns an error if the type of this selection set is not defined
    /// or does not have a field named `name`.
    pub fn new_field<'schema>(
        &self,
        schema: &'schema Schema,
        name: Name,
    ) -> Result<Field, schema::FieldLookupError<'schema>> {
        let definition = schema.type_field(&self.ty, &name)?.node.clone();
        Ok(Field::new(name, definition))
    }

    /// Create a new inline fragment to be added to this selection set with [`push`][Self::push]
    pub fn new_inline_fragment(&self, opt_type_condition: Option<NamedType>) -> InlineFragment {
        if let Some(type_condition) = opt_type_condition {
            InlineFragment::with_type_condition(type_condition)
        } else {
            InlineFragment::without_type_condition(self.ty.clone())
        }
    }

    /// Create a new fragment spread to be added to this selection set with [`push`][Self::push]
    pub fn new_fragment_spread(&self, fragment_name: Name) -> FragmentSpread {
        FragmentSpread::new(fragment_name)
    }

    /// Returns an iterator of field selections directly in this selection set.
    ///
    /// Does not recur into inline fragments or fragment spreads.
    pub fn fields(&self) -> impl Iterator<Item = &Node<Field>> {
        self.selections.iter().filter_map(|sel| sel.as_field())
    }

    /// Returns an iterator of field selections that are at the root of the response.
    /// That is, inline fragments and fragment spreads at the root are traversed,
    /// but field sub-selections are not.
    ///
    /// See also [`all_fields`][Self::all_fields].
    ///
    /// `document` is used to look up fragment definitions.
    ///
    /// This does **not** perform [field merging],
    /// so multiple items in this iterator may have the same response key
    /// or point to the same field definition.
    /// Named fragments however are only traversed once even if spread multiple times.
    ///
    /// [field merging]: https://spec.graphql.org/draft/#sec-Field-Selection-Merging
    pub fn root_fields<'doc>(
        &'doc self,
        document: &'doc ExecutableDocument,
    ) -> impl Iterator<Item = &'doc Node<Field>> {
        let mut stack = vec![self.selections.iter()];
        let mut fragments_seen = HashSet::default();
        std::iter::from_fn(move || {
            while let Some(selection_set_iter) = stack.last_mut() {
                match selection_set_iter.next() {
                    Some(Selection::Field(field)) => {
                        // Yield one item from the `root_fields()` iterator
                        // but ignore its sub-selections in `field.selection_set`
                        return Some(field);
                    }
                    Some(Selection::InlineFragment(inline)) => {
                        stack.push(inline.selection_set.selections.iter())
                    }
                    Some(Selection::FragmentSpread(spread)) => {
                        if let Some(def) = document.fragments.get(&spread.fragment_name) {
                            let new = fragments_seen.insert(&spread.fragment_name);
                            if new {
                                stack.push(def.selection_set.selections.iter())
                            }
                        } else {
                            // Undefined fragments are silently ignored.
                            // They should never happen in a valid document.
                        }
                    }
                    None => {
                        // Remove an empty iterator from the stack
                        // and continue with the parent selection set
                        stack.pop();
                    }
                }
            }
            None
        })
    }

    /// Returns an iterator of all field selections in this operation.
    ///
    /// See also [`root_fields`][Self::root_fields].
    ///
    /// `document` is used to look up fragment definitions.
    ///
    /// This does **not** perform [field merging],
    /// so multiple items in this iterator may have the same response key
    /// or point to the same field definition.
    /// Named fragments however are only traversed once even if spread multiple times.
    ///
    /// [field merging]: https://spec.graphql.org/draft/#sec-Field-Selection-Merging
    pub fn all_fields<'doc>(
        &'doc self,
        document: &'doc ExecutableDocument,
    ) -> impl Iterator<Item = &'doc Node<Field>> {
        let mut stack = vec![self.selections.iter()];
        let mut fragments_seen = HashSet::default();
        std::iter::from_fn(move || {
            while let Some(selection_set_iter) = stack.last_mut() {
                match selection_set_iter.next() {
                    Some(Selection::Field(field)) => {
                        if !field.selection_set.is_empty() {
                            // Will be considered for the next call
                            stack.push(field.selection_set.selections.iter())
                        }
                        // Yield one item from the `all_fields()` iterator
                        return Some(field);
                    }
                    Some(Selection::InlineFragment(inline)) => {
                        stack.push(inline.selection_set.selections.iter())
                    }
                    Some(Selection::FragmentSpread(spread)) => {
                        if let Some(def) = document.fragments.get(&spread.fragment_name) {
                            let new = fragments_seen.insert(&spread.fragment_name);
                            if new {
                                stack.push(def.selection_set.selections.iter())
                            }
                        } else {
                            // Undefined fragments are silently ignored.
                            // They should never happen in a valid document.
                        }
                    }
                    None => {
                        // Remove an empty iterator from the stack
                        // and continue with the parent selection set
                        stack.pop();
                    }
                }
            }
            None
        })
    }

    serialize_method!();
}

impl Selection {
    pub fn directives(&self) -> &DirectiveList {
        match self {
            Self::Field(sel) => &sel.directives,
            Self::FragmentSpread(sel) => &sel.directives,
            Self::InlineFragment(sel) => &sel.directives,
        }
    }

    pub fn as_field(&self) -> Option<&Node<Field>> {
        if let Self::Field(field) = self {
            Some(field)
        } else {
            None
        }
    }

    pub fn as_inline_fragment(&self) -> Option<&Node<InlineFragment>> {
        if let Self::InlineFragment(inline) = self {
            Some(inline)
        } else {
            None
        }
    }

    pub fn as_fragment_spread(&self) -> Option<&Node<FragmentSpread>> {
        if let Self::FragmentSpread(spread) = self {
            Some(spread)
        } else {
            None
        }
    }

    serialize_method!();
}

impl From<Node<Field>> for Selection {
    fn from(node: Node<Field>) -> Self {
        Self::Field(node)
    }
}

impl From<Node<InlineFragment>> for Selection {
    fn from(node: Node<InlineFragment>) -> Self {
        Self::InlineFragment(node)
    }
}

impl From<Node<FragmentSpread>> for Selection {
    fn from(node: Node<FragmentSpread>) -> Self {
        Self::FragmentSpread(node)
    }
}

impl From<Field> for Selection {
    fn from(value: Field) -> Self {
        Self::Field(Node::new(value))
    }
}

impl From<InlineFragment> for Selection {
    fn from(value: InlineFragment) -> Self {
        Self::InlineFragment(Node::new(value))
    }
}

impl From<FragmentSpread> for Selection {
    fn from(value: FragmentSpread) -> Self {
        Self::FragmentSpread(Node::new(value))
    }
}

impl Field {
    /// Create a new field with the given name and type.
    ///
    /// See [`SelectionSet::new_field`] too look up the type in a schema instead.
    pub fn new(name: Name, definition: Node<schema::FieldDefinition>) -> Self {
        let selection_set = SelectionSet::new(definition.ty.inner_named_type().clone());
        Field {
            definition,
            alias: None,
            name,
            arguments: Vec::new(),
            directives: DirectiveList::new(),
            selection_set,
        }
    }

    pub fn with_alias(mut self, alias: Name) -> Self {
        self.alias = Some(alias);
        self
    }

    pub fn with_opt_alias(mut self, alias: Option<Name>) -> Self {
        self.alias = alias;
        self
    }

    pub fn with_directive(mut self, directive: impl Into<Node<Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<Directive>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn with_argument(mut self, name: Name, value: impl Into<Node<Value>>) -> Self {
        self.arguments.push((name, value).into());
        self
    }

    pub fn with_arguments(mut self, arguments: impl IntoIterator<Item = Node<Argument>>) -> Self {
        self.arguments.extend(arguments);
        self
    }

    pub fn with_selection(mut self, selection: impl Into<Selection>) -> Self {
        self.selection_set.push(selection);
        self
    }

    pub fn with_selections(
        mut self,
        selections: impl IntoIterator<Item = impl Into<Selection>>,
    ) -> Self {
        self.selection_set.extend(selections);
        self
    }

    /// Returns the response key for this field: the alias if there is one, or the name
    pub fn response_key(&self) -> &Name {
        self.alias.as_ref().unwrap_or(&self.name)
    }

    /// The type of this field, from the field definition
    pub fn ty(&self) -> &Type {
        &self.definition.ty
    }

    /// Look up in `schema` the definition of the inner type of this field.
    ///
    /// The inner type is [`ty()`][Self::ty] after unwrapping non-null and list markers.
    pub fn inner_type_def<'a>(&self, schema: &'a Schema) -> Option<&'a schema::ExtendedType> {
        schema.types.get(self.ty().inner_named_type())
    }

    /// Returns the value of the argument named `name`, accounting for nullability
    /// and for the default value in `schema`’s directive definition.
    pub fn argument_by_name(&self, name: &str) -> Result<&Node<Value>, ArgumentByNameError> {
        Argument::argument_by_name(&self.arguments, name, || {
            self.definition
                .argument_by_name(name)
                .ok_or(ArgumentByNameError::NoSuchArgument)
        })
    }

    /// Returns the value of the argument named `name`, as specified in the field selection.
    ///
    /// Returns `None` if the field selection does not specify this argument.
    ///
    /// If the field definition makes this argument nullable or defines a default value,
    /// consider using [`argument_by_name`][Self::argument_by_name] instead.
    pub fn specified_argument_by_name(&self, name: &str) -> Option<&Node<Value>> {
        Argument::specified_argument_by_name(&self.arguments, name)
    }

    serialize_method!();
}

impl InlineFragment {
    pub fn with_type_condition(type_condition: NamedType) -> Self {
        let selection_set = SelectionSet::new(type_condition.clone());
        Self {
            type_condition: Some(type_condition),
            directives: DirectiveList::new(),
            selection_set,
        }
    }

    pub fn without_type_condition(parent_selection_set_type: NamedType) -> Self {
        Self {
            type_condition: None,
            directives: DirectiveList::new(),
            selection_set: SelectionSet::new(parent_selection_set_type),
        }
    }

    pub fn with_directive(mut self, directive: impl Into<Node<Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<Directive>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn with_selection(mut self, selection: impl Into<Selection>) -> Self {
        self.selection_set.push(selection);
        self
    }

    pub fn with_selections(
        mut self,
        selections: impl IntoIterator<Item = impl Into<Selection>>,
    ) -> Self {
        self.selection_set.extend(selections);
        self
    }

    serialize_method!();
}

impl FragmentSpread {
    pub fn new(fragment_name: Name) -> Self {
        Self {
            fragment_name,
            directives: DirectiveList::new(),
        }
    }

    pub fn with_directive(mut self, directive: impl Into<Node<Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<Directive>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn fragment_def<'a>(&self, document: &'a ExecutableDocument) -> Option<&'a Node<Fragment>> {
        document.fragments.get(&self.fragment_name)
    }

    serialize_method!();
}

impl FieldSet {
    /// Parse the given source a selection set with optional outer brackets.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    pub fn parse(
        schema: &Valid<Schema>,
        type_name: NamedType,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<FieldSet, WithErrors<FieldSet>> {
        Parser::new().parse_field_set(schema, type_name, source_text, path)
    }

    /// [`parse`][Self::parse] then [`validate`][Self::validate],
    /// to get a `Valid<ExecutableDocument>` when mutating it isn’t needed.
    pub fn parse_and_validate(
        schema: &Valid<Schema>,
        type_name: NamedType,
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Valid<Self>, WithErrors<Self>> {
        let (field_set, mut errors) =
            Parser::new().parse_field_set_inner(schema, type_name, source_text, path);
        validation::validate_field_set(&mut errors, schema, &field_set);
        errors.into_valid_result(field_set)
    }

    pub fn validate(&self, schema: &Valid<Schema>) -> Result<(), DiagnosticList> {
        let mut sources = IndexMap::clone(&schema.sources);
        sources.extend(self.sources.iter().map(|(k, v)| (*k, v.clone())));
        let mut errors = DiagnosticList::new(Arc::new(sources));
        validation::validate_field_set(&mut errors, schema, self);
        errors.into_result()
    }

    serialize_method!();
}

impl fmt::Display for SelectionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.root {
            ExecutableDefinitionName::AnonymousOperation(operation_type) => {
                write!(f, "{operation_type}")?
            }
            ExecutableDefinitionName::NamedOperation(operation_type, name) => {
                write!(f, "{operation_type} {name}")?
            }
            ExecutableDefinitionName::Fragment(name) => write!(f, "fragment {name}")?,
        }
        for name in &self.nested_fields {
            write!(f, " → {name}")?
        }
        Ok(())
    }
}
