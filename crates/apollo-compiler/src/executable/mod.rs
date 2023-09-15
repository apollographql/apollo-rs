use crate::ast;
use crate::ast::impls::directives_by_name;
use crate::schema::FieldLookupError;
use crate::Node;
use crate::Schema;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::collections::HashSet;

mod from_ast;
mod serialize;
#[cfg(test)]
mod tests;

pub use crate::ast::{
    Argument, Directive, Name, NamedType, OperationType, Type, Value, VariableDefinition,
};

/// Executable definitions, annotated with type information
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableDocument {
    pub named_operations: IndexMap<Name, Node<Operation>>,
    pub anonymous_operation: Option<Node<Operation>>,
    pub fragments: IndexMap<Name, Node<Fragment>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    pub operation_type: OperationType,
    pub variables: Vec<Node<VariableDefinition>>,
    pub directives: Vec<Node<Directive>>,
    pub selection_set: SelectionSet,
}

pub enum OperationRef<'a> {
    Anonymous(&'a Node<Operation>),
    Named(&'a Name, &'a Node<Operation>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub directives: Vec<Node<Directive>>,
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
    /// The type of this field, resolved from context and schema
    pub ty: Type,
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Node<Argument>>,
    pub directives: Vec<Node<Directive>>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Vec<Node<Directive>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFragment {
    pub type_condition: Option<NamedType>,
    pub directives: Vec<Node<Directive>>,
    pub selection_set: SelectionSet,
}

/// AST node that has been skipped during conversion to `ExecutableDocument`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstructionError {
    /// The schema does not define a root operation
    /// for the operation type of this operation definition
    UndefinedRootOperation(Node<ast::OperationDefinition>),
    /// Could not resolve the type of this field because the schema does not define
    /// the type of its parent selection set
    UndefinedType {
        type_name: NamedType,
        field: Node<ast::Field>,
    },
    /// Could not resolve the type of this field because the schema does not define it
    UndefinedField {
        type_name: NamedType,
        field: Node<ast::Field>,
    },
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
    pub fn from_ast(
        schema: &Schema,
        document: &ast::Document,
    ) -> (Self, Result<(), Vec<ConstructionError>>) {
        self::from_ast::document_from_ast(schema, document)
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

    directive_methods!();
}

impl Fragment {
    pub fn type_condition(&self) -> &NamedType {
        &self.selection_set.ty
    }

    directive_methods!();
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
    ) -> Result<Field, FieldLookupError> {
        let name = name.into();
        let ty = schema.type_field(&self.ty, &name)?.ty.clone();
        Ok(Field::new(name, ty))
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
    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives. For non-repeatable directives,
    /// see [`directive_by_name`][Self::directive_by_name] (singular)
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
        match self {
            Selection::Field(field) => directives_by_name(&field.directives, name),
            Selection::FragmentSpread(spread) => directives_by_name(&spread.directives, name),
            Selection::InlineFragment(inline) => directives_by_name(&inline.directives, name),
        }
    }

    directive_by_name_method!();
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
    pub fn new(name: impl Into<Name>, ty: Type) -> Self {
        let selection_set = SelectionSet::new(ty.inner_named_type().clone());
        Field {
            ty,
            alias: None,
            name: name.into(),
            arguments: Vec::new(),
            directives: Vec::new(),
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

    directive_methods!();
}

impl InlineFragment {
    pub fn with_type_condition(type_condition: impl Into<NamedType>) -> Self {
        let type_condition = type_condition.into();
        let selection_set = SelectionSet::new(type_condition.clone());
        Self {
            type_condition: Some(type_condition),
            directives: Vec::new(),
            selection_set,
        }
    }

    pub fn without_type_condition(parent_selection_set_type: impl Into<NamedType>) -> Self {
        Self {
            type_condition: None,
            directives: Vec::new(),
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

    directive_methods!();
}

impl FragmentSpread {
    pub fn new(fragment_name: impl Into<Name>) -> Self {
        Self {
            fragment_name: fragment_name.into(),
            directives: Vec::new(),
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

    directive_methods!();
}
