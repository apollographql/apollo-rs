use super::*;
use crate::ast::serialize::curly_brackets_space_separated;
use crate::ast::serialize::State;
use std::fmt;

impl ExecutableDocument {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        // TODO: avoid allocating temporary AST nodes?
        // it would ~duplicate large parts of ast/serialize.rs
        self.to_ast().serialize_impl(state)
    }

    pub(crate) fn to_ast(&self) -> ast::Document {
        let mut doc = ast::Document::new();
        if let Some(operation) = &self.anonymous_operation {
            doc.definitions.push(operation.to_ast(operation.location()))
        }
        for operation in self.named_operations.values() {
            doc.definitions.push(operation.to_ast(operation.location()))
        }
        for fragment in self.fragments.values() {
            doc.definitions.push(fragment.to_ast(fragment.location()))
        }
        doc
    }
}

impl Operation {
    fn to_ast(&self, location: Option<NodeLocation>) -> ast::Definition {
        let def = ast::OperationDefinition {
            operation_type: self.operation_type,
            name: self.name.clone(),
            variables: self.variables.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        };
        ast::Definition::OperationDefinition(Node::new_opt_location(def, location))
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast(None).serialize_impl(state)
    }
}

impl Fragment {
    fn to_ast(&self, location: Option<NodeLocation>) -> ast::Definition {
        let def = ast::FragmentDefinition {
            name: self.name.clone(),
            type_condition: self.selection_set.ty.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        };
        ast::Definition::FragmentDefinition(Node::new_opt_location(def, location))
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast(None).serialize_impl(state)
    }
}

impl SelectionSet {
    pub(crate) fn to_ast(&self) -> Vec<ast::Selection> {
        self.selections
            .iter()
            .map(|selection| selection.to_ast())
            .collect()
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        curly_brackets_space_separated(state, &self.selections, |state, sel| {
            sel.serialize_impl(state)
        })
    }
}

impl Selection {
    pub(crate) fn to_ast(&self) -> ast::Selection {
        match self {
            Selection::Field(field) => ast::Selection::Field(field.same_location(field.to_ast())),
            Selection::FragmentSpread(fragment_spread) => ast::Selection::FragmentSpread(
                fragment_spread.same_location(fragment_spread.to_ast()),
            ),
            Selection::InlineFragment(inline_fragment) => ast::Selection::InlineFragment(
                inline_fragment.same_location(inline_fragment.to_ast()),
            ),
        }
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast().serialize_impl(state)
    }
}

impl Field {
    pub(crate) fn to_ast(&self) -> ast::Field {
        ast::Field {
            alias: self.alias.clone(),
            name: self.name.clone(),
            arguments: self.arguments.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        }
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast().serialize_impl(state)
    }
}

impl InlineFragment {
    pub(crate) fn to_ast(&self) -> ast::InlineFragment {
        ast::InlineFragment {
            type_condition: self.type_condition.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        }
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast().serialize_impl(state)
    }
}

impl FragmentSpread {
    pub(crate) fn to_ast(&self) -> ast::FragmentSpread {
        ast::FragmentSpread {
            fragment_name: self.fragment_name.clone(),
            directives: self.directives.clone(),
        }
    }

    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        self.to_ast().serialize_impl(state)
    }
}

impl FieldSet {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        if let Some((first, rest)) = self.selection_set.selections.split_first() {
            first.serialize_impl(state)?;
            for value in rest {
                state.new_line_or_space()?;
                value.serialize_impl(state)?;
            }
        }
        Ok(())
    }
}
