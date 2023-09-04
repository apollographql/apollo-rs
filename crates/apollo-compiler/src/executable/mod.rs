use crate::ast;
use crate::Node;
use crate::Schema;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::fmt;

mod from_ast;
mod serialize;
#[cfg(test)]
mod tests;

pub use crate::ast::{Directive, Name, NamedType, OperationType, Type, Value, VariableDefinition};
use std::collections::HashSet;

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
    pub arguments: Vec<(Name, Node<Value>)>,
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

/// Tried to create a selection set that would be invalid for the given schema.
///
/// This is not full validation of the executable document,
/// only some type-related cases cause this error to be returned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError(&'static str);

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, validate for details", self.0)
    }
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
    pub fn from_ast(schema: &Schema, document: &ast::Document) -> Result<Self, TypeError> {
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
    pub fn get_operation<N>(
        &self,
        name_request: Option<&N>,
    ) -> Result<OperationRef<'_>, GetOperationError>
    where
        N: std::hash::Hash + indexmap::Equivalent<Name>,
    {
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
    pub fn new(schema: &Schema, ty: NamedType) -> Result<Self, TypeError> {
        if schema.types.contains_key(&ty) {
            Ok(Self {
                ty,
                selections: Vec::new(),
            })
        } else {
            Err(TypeError("no type definition with that name"))
        }
    }

    /// Create a new selection set for the root of an operation
    pub fn for_operation(
        schema: &Schema,
        operation_type: OperationType,
    ) -> Result<Self, TypeError> {
        let ty = schema
            .root_operation(operation_type)
            .ok_or(TypeError(
                "missing root operation definition for the operation type",
            ))?
            .node
            .clone();
        if let Some(def) = schema.types.get(&ty) {
            if def.is_object() {
                Ok(Self {
                    ty,
                    selections: Vec::new(),
                })
            } else {
                Err(TypeError(
                    "type definition for the root operation is not an object type",
                ))
            }
        } else {
            Err(TypeError("missing type definition for the root operation"))
        }
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
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
    /// Returns an error if the type of this selection set does not have a field named `name`,
    /// or if that field’s own type is not defined.
    pub fn new_field(&self, schema: &Schema, name: Name) -> Result<Field, TypeError> {
        let ty = schema
            .type_field(&self.ty, &name)
            .ok_or(TypeError("no field definition with that name"))?
            .ty
            .clone();
        let selection_set = SelectionSet::new(schema, ty.inner_named_type().clone())?;
        Ok(Field {
            ty,
            alias: None,
            name,
            arguments: Vec::new(),
            directives: Vec::new(),
            selection_set,
        })
    }

    /// Create a new inline fragment to be added to this selection set with [`push`][Self::push]
    pub fn new_inline_fragment(
        &self,
        schema: &Schema,
        type_condition: Option<NamedType>,
    ) -> Result<InlineFragment, TypeError> {
        let inner_parent_type = type_condition.clone().unwrap_or(self.ty.clone());
        let inner = SelectionSet::new(schema, inner_parent_type)?;
        Ok(InlineFragment {
            type_condition,
            directives: Vec::new(),
            selection_set: inner,
        })
    }

    /// Create a new fragment spread to be added to this selection set with [`push`][Self::push]
    pub fn new_fragment_spread(&self, fragment_name: Name) -> FragmentSpread {
        FragmentSpread {
            fragment_name,
            directives: Vec::new(),
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
        Self::Field(Node::new_synthetic(value))
    }
}

impl From<InlineFragment> for Selection {
    fn from(value: InlineFragment) -> Self {
        Self::InlineFragment(Node::new_synthetic(value))
    }
}

impl From<FragmentSpread> for Selection {
    fn from(value: FragmentSpread) -> Self {
        Self::FragmentSpread(Node::new_synthetic(value))
    }
}

impl Field {
    pub fn with_alias(mut self, alias: impl Into<Option<Name>>) -> Self {
        self.alias = alias.into();
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
        self.arguments.push((name.into(), value.into()));
        self
    }

    pub fn with_arguments(
        mut self,
        arguments: impl IntoIterator<Item = (impl Into<Name>, impl Into<Node<Value>>)>,
    ) -> Self {
        self.arguments.extend(
            arguments
                .into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
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

    fn with_ast_selections(
        mut self,
        schema: &Schema,
        ast_selections: &[ast::Selection],
    ) -> Result<Self, TypeError> {
        self.selection_set.extend_from_ast(schema, ast_selections)?;
        Ok(self)
    }
}

impl InlineFragment {
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

    fn with_ast_selections(
        mut self,
        schema: &Schema,
        ast_selections: &[ast::Selection],
    ) -> Result<Self, TypeError> {
        self.selection_set.extend_from_ast(schema, ast_selections)?;
        Ok(self)
    }
}

impl FragmentSpread {
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
