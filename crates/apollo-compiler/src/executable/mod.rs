use crate::ast;
use crate::ast::Name;
use crate::ast::OperationType;
use crate::Node;
use crate::Schema;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::fmt;

mod from_ast;
mod serialize;

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
    pub variables: Vec<Node<ast::VariableDefinition>>,
    pub directives: Vec<Node<ast::Directive>>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub directives: Vec<Node<ast::Directive>>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionSet {
    pub ty: ast::NamedType,
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
    pub ty: ast::Type,
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<(Name, Node<ast::Value>)>,
    pub directives: Vec<Node<ast::Directive>>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Vec<Node<ast::Directive>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineFragment {
    pub type_condition: Option<ast::NamedType>,
    pub directives: Vec<Node<ast::Directive>>,
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
        self.0.fmt(f)
    }
}

impl ExecutableDocument {
    pub fn from_ast(schema: &Schema, document: &ast::Document) -> Result<Self, TypeError> {
        self::from_ast::document_from_ast(schema, document)
    }

    serialize_method!();
}

impl Operation {
    pub fn object_type(&self) -> &ast::NamedType {
        &self.selection_set.ty
    }
}

impl Fragment {
    pub fn type_condition(&self) -> &ast::NamedType {
        &self.selection_set.ty
    }
}

impl SelectionSet {
    /// Create a new selection set
    pub fn new(schema: &Schema, ty: ast::NamedType) -> Result<Self, TypeError> {
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
        type_condition: Option<ast::NamedType>,
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

    pub fn with_directive(mut self, directive: impl Into<Node<ast::Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<ast::Directive>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn with_argument(
        mut self,
        name: impl Into<Name>,
        value: impl Into<Node<ast::Value>>,
    ) -> Self {
        self.arguments.push((name.into(), value.into()));
        self
    }

    pub fn with_arguments(
        mut self,
        arguments: impl IntoIterator<Item = (impl Into<Name>, impl Into<Node<ast::Value>>)>,
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
    pub fn with_directive(mut self, directive: impl Into<Node<ast::Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<ast::Directive>>,
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
    pub fn with_directive(mut self, directive: impl Into<Node<ast::Directive>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Node<ast::Directive>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }
}
