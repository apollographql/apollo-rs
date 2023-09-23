use crate::ast;
use crate::schema;
use crate::Arc;
use crate::FileId;
use crate::Node;
use crate::Parser;
use crate::Schema;
use crate::SourceFile;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::collections::HashSet;
use std::path::Path;

pub(crate) mod from_ast;
mod serialize;

pub use crate::ast::{
    Argument, Directive, Directives, Name, NamedType, OperationType, Type, Value,
    VariableDefinition,
};

/// Executable definitions, annotated with type information
#[derive(Debug, Clone, Default)]
pub struct ExecutableDocument {
    /// If this document was originally parsed from a source file,
    /// that file and its ID.
    ///
    /// The document may have been modified since.
    pub source: Option<(FileId, Arc<SourceFile>)>,

    /// Errors that occurred when building this document,
    /// either parsing a source file or converting from AST.
    pub build_errors: Vec<BuildError>,

    pub anonymous_operation: Option<Node<Operation>>,
    pub named_operations: IndexMap<Name, Node<Operation>>,
    pub fragments: IndexMap<Name, Node<Fragment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub operation_type: OperationType,
    pub variables: Vec<Node<VariableDefinition>>,
    pub directives: Directives,
    pub selection_set: SelectionSet,
}

pub enum OperationRef<'a> {
    Anonymous(&'a Node<Operation>),
    Named(&'a Name, &'a Node<Operation>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub directives: Directives,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    pub ty: NamedType,
    pub selections: Vec<Selection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selection {
    Field(Node<Field>),
    FragmentSpread(Node<FragmentSpread>),
    InlineFragment(Node<InlineFragment>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Field {
    /// The definition of this field in an object type or interface type definition in the schema
    pub definition: Node<schema::FieldDefinition>,
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
    pub directives: Directives,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Directives,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Directives,
    pub selection_set: SelectionSet,
}

/// AST node that has been skipped during conversion to `ExecutableDocument`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// Found a type system definition, which is unexpected when building an executable document.
    ///
    /// If this is intended, use `parse_mixed`.
    UnexpectedTypeSystemDefinition(ast::Definition),

    /// Found multiple operations without a name
    DuplicateAnonymousOperation(Node<ast::OperationDefinition>),

    /// Found multiple operations with the same name
    OperationNameCollision(Node<ast::OperationDefinition>),

    /// Found multiple fragments with the same name
    FragmentNameCollision(Node<ast::FragmentDefinition>),

    /// The schema does not define a root operation
    /// for the operation type of this operation definition
    UndefinedRootOperation(Node<ast::OperationDefinition>),

    /// Could not resolve the type of this field because the schema does not define
    /// the type of its parent selection set
    UndefinedType {
        /// Which top-level executable definition contains this error
        top_level: ExecutableDefinitionName,
        /// Response keys (alias or name) of nested fields that contain the field with the error,
        /// outer-most to inner-most.
        ancestor_fields: Vec<Name>,
        type_name: NamedType,
        field: Node<ast::Field>,
    },

    /// Could not resolve the type of this field because the schema does not define it
    UndefinedField {
        /// Which top-level executable definition contains this error
        top_level: ExecutableDefinitionName,
        /// Response keys (alias or name) of nested fields that contain the field with the error,
        /// outer-most to inner-most.
        ancestor_fields: Vec<Name>,
        type_name: NamedType,
        field: Node<ast::Field>,
    },
}

/// Designates by name a top-level definition in an executable document
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutableDefinitionName {
    AnonymousOperation,
    NamedOperation(Name),
    Fragment(Name),
}

/// A request error returned by [`ExecutableDocument::get_operation`]
///
/// If `get_operation`’s `name_request` argument was `Some`, this error indicates
/// that the document does not contain an operation with the requested name.
///
/// If `name_request` was `None`, the request is ambiguous
/// because the document contains multiple operations
/// (or zero, though the document would be invalid in that case).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct GetOperationError();

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
    pub fn parse(schema: &Schema, source_text: impl Into<String>, path: impl AsRef<Path>) -> Self {
        Parser::new().parse_executable(schema, source_text, path)
    }

    /// Returns an iterator of operations, both anonymous and named
    pub fn all_operations(&self) -> impl Iterator<Item = OperationRef<'_>> {
        self.anonymous_operation
            .as_ref()
            .into_iter()
            .map(OperationRef::Anonymous)
            .chain(
                self.named_operations
                    .iter()
                    .map(|(name, op)| OperationRef::Named(name, op)),
            )
    }

    /// Return the relevant operation for a request, or a request error
    ///
    /// This the [GetOperation](https://spec.graphql.org/October2021/#GetOperation())
    /// algorithm in the _Executing Requests_ section of the specification.
    ///
    /// A GraphQL request comes with a document (which may contain multiple operations)
    /// an an optional operation name. When a name is given the request executes the operation
    /// with that name, which is expected to exist. When it is not given / null / `None`,
    /// the document is expected to contain a single operation (which may or may not be named)
    /// to avoid ambiguity.
    pub fn get_operation(
        &self,
        name_request: Option<&str>,
    ) -> Result<OperationRef<'_>, GetOperationError> {
        if let Some(name) = name_request {
            // Honor the request
            self.named_operations
                .get_key_value(name)
                .map(|(name, op)| OperationRef::Named(name, op))
        } else if let Some(op) = &self.anonymous_operation {
            // No name request, return the anonymous operation if it’s the only operation
            self.named_operations
                .is_empty()
                .then_some(OperationRef::Anonymous(op))
        } else {
            // No name request or anonymous operation, return a named operation if it’s the only one
            self.named_operations.iter().next().and_then(|(name, op)| {
                (self.named_operations.len() == 1).then_some(OperationRef::Named(name, op))
            })
        }
        .ok_or(GetOperationError())
    }

    /// Similar to [`get_operation`][Self::get_operation] but returns a mutable reference.
    pub fn get_operation_mut(
        &mut self,
        name_request: Option<&str>,
    ) -> Result<&mut Operation, GetOperationError> {
        if let Some(name) = name_request {
            // Honor the request
            self.named_operations.get_mut(name)
        } else if let Some(op) = &mut self.anonymous_operation {
            // No name request, return the anonymous operation if it’s the only operation
            self.named_operations.is_empty().then_some(op)
        } else {
            // No name request or anonymous operation, return a named operation if it’s the only one
            let len = self.named_operations.len();
            self.named_operations
                .values_mut()
                .next()
                .and_then(|op| (len == 1).then_some(op))
        }
        .map(Node::make_mut)
        .ok_or(GetOperationError())
    }

    serialize_method!();
}

impl Eq for ExecutableDocument {}

/// `source` and `build_errors` are ignored for comparison
impl PartialEq for ExecutableDocument {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            source: _,
            build_errors: _,
            anonymous_operation,
            named_operations,
            fragments,
        } = self;
        *anonymous_operation == other.anonymous_operation
            && *named_operations == other.named_operations
            && *fragments == other.fragments
    }
}

impl<'a> OperationRef<'a> {
    pub fn name(&self) -> Option<&'a Name> {
        match self {
            OperationRef::Anonymous(_) => None,
            OperationRef::Named(name, _) => Some(name),
        }
    }

    pub fn definition(&self) -> &'a Node<Operation> {
        match self {
            OperationRef::Anonymous(def) | OperationRef::Named(_, def) => def,
        }
    }
}

impl std::ops::Deref for OperationRef<'_> {
    type Target = Node<Operation>;

    fn deref(&self) -> &Self::Target {
        self.definition()
    }
}

impl Operation {
    pub fn object_type(&self) -> &NamedType {
        &self.selection_set.ty
    }

    /// Return whether this operation is a query that only selects introspection meta-fields:
    /// `__type`, `__schema`, and `__typename`
    pub fn is_introspection(&self, document: &ExecutableDocument) -> bool {
        fn is_introspection_impl<'a>(
            document: &'a ExecutableDocument,
            seen_fragments: &mut HashSet<&'a Name>,
            set: &'a SelectionSet,
        ) -> bool {
            set.selections.iter().all(|selection| match selection {
                Selection::Field(field) => {
                    matches!(field.name.as_str(), "__type" | "__schema" | "__typename")
                }
                Selection::FragmentSpread(spread) => {
                    document
                        .fragments
                        .get(&spread.fragment_name)
                        .is_some_and(|fragment| {
                            let new = seen_fragments.insert(&spread.fragment_name);
                            if new {
                                is_introspection_impl(
                                    document,
                                    seen_fragments,
                                    &fragment.selection_set,
                                )
                            } else {
                                // This isn't the first time we've seen this spread.
                                // We trust that the first visit will find all
                                // relevant fields and stop the recursion (without
                                // affecting the overall `all` result).
                                true
                            }
                        })
                }
                Selection::InlineFragment(inline) => {
                    is_introspection_impl(document, seen_fragments, &inline.selection_set)
                }
            })
        }

        self.operation_type == OperationType::Query
            && is_introspection_impl(document, &mut HashSet::new(), &self.selection_set)
    }
}

impl Fragment {
    pub fn type_condition(&self) -> &NamedType {
        &self.selection_set.ty
    }
}

impl SelectionSet {
    /// Create a new selection set
    pub fn new(ty: impl Into<NamedType>) -> Self {
        Self {
            ty: ty.into(),
            selections: Vec::new(),
        }
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
    pub fn new_field(
        &self,
        schema: &Schema,
        name: impl Into<Name>,
    ) -> Result<Field, schema::FieldLookupError> {
        let name = name.into();
        let definition = schema.type_field(&self.ty, &name)?.node.clone();
        Ok(Field::new(name, definition))
    }

    /// Create a new inline fragment to be added to this selection set with [`push`][Self::push]
    pub fn new_inline_fragment(
        &self,
        opt_type_condition: Option<impl Into<NamedType>>,
    ) -> InlineFragment {
        if let Some(type_condition) = opt_type_condition {
            InlineFragment::with_type_condition(type_condition)
        } else {
            InlineFragment::without_type_condition(self.ty.clone())
        }
    }

    /// Create a new fragment spread to be added to this selection set with [`push`][Self::push]
    pub fn new_fragment_spread(&self, fragment_name: impl Into<Name>) -> FragmentSpread {
        FragmentSpread::new(fragment_name)
    }
}

impl Selection {
    pub fn directives(&self) -> &Directives {
        match self {
            Self::Field(sel) => &sel.directives,
            Self::FragmentSpread(sel) => &sel.directives,
            Self::InlineFragment(sel) => &sel.directives,
        }
    }
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
    pub fn new(name: impl Into<Name>, definition: Node<schema::FieldDefinition>) -> Self {
        let selection_set = SelectionSet::new(definition.ty.inner_named_type().clone());
        Field {
            definition,
            alias: None,
            name: name.into(),
            arguments: Vec::new(),
            directives: Directives::new(),
            selection_set,
        }
    }

    pub fn with_alias(mut self, alias: impl Into<Name>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    pub fn with_opt_alias(mut self, alias: Option<impl Into<Name>>) -> Self {
        self.alias = alias.map(Into::into);
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

    pub fn with_argument(mut self, name: impl Into<Name>, value: impl Into<Node<Value>>) -> Self {
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
}

impl InlineFragment {
    pub fn with_type_condition(type_condition: impl Into<NamedType>) -> Self {
        let type_condition = type_condition.into();
        let selection_set = SelectionSet::new(type_condition.clone());
        Self {
            type_condition: Some(type_condition),
            directives: Directives::new(),
            selection_set,
        }
    }

    pub fn without_type_condition(parent_selection_set_type: impl Into<NamedType>) -> Self {
        Self {
            type_condition: None,
            directives: Directives::new(),
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
}

impl FragmentSpread {
    pub fn new(fragment_name: impl Into<Name>) -> Self {
        Self {
            fragment_name: fragment_name.into(),
            directives: Directives::new(),
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
}
