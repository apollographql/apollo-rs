use super::*;
use crate::ast::serialize::State;
use std::fmt;

impl ExecutableDocument {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        // TODO: avoid allocating temporary AST nodes?
        // it would ~duplicate large parts of ast/serialize.rs
        self.to_ast().serialize_impl(state)
    }

    fn to_ast(&self) -> ast::Document {
        let mut doc = ast::Document::new();
        if let Some(operation) = &self.anonymous_operation {
            doc.definitions.push(operation.to_ast(None))
        }
        for (name, operation) in &self.named_operations {
            doc.definitions.push(operation.to_ast(Some(name)))
        }
        for (name, fragment) in &self.fragments {
            doc.definitions.push(fragment.to_ast(name))
        }
        doc
    }
}

impl Operation {
    fn to_ast(&self, name: Option<&Name>) -> ast::Definition {
        ast::Definition::OperationDefinition(Node::new_synthetic(ast::OperationDefinition {
            operation_type: self.operation_type,
            name: name.cloned(),
            variables: self.variables.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        }))
    }
}

impl Fragment {
    fn to_ast(&self, name: &Name) -> ast::Definition {
        ast::Definition::FragmentDefinition(Node::new_synthetic(ast::FragmentDefinition {
            name: name.clone(),
            type_condition: self.selection_set.ty.clone(),
            directives: self.directives.clone(),
            selection_set: self.selection_set.to_ast(),
        }))
    }
}

impl SelectionSet {
    fn to_ast(&self) -> Vec<ast::Selection> {
        self.selections()
            .iter()
            .map(|selection| match selection {
                Selection::Field(field) => ast::Selection::Field(Node::new_synthetic(ast::Field {
                    alias: field.alias.clone(),
                    name: field.name.clone(),
                    arguments: field.arguments.clone(),
                    directives: field.directives.clone(),
                    selection_set: field.selection_set.to_ast(),
                })),
                Selection::FragmentSpread(fragment_spread) => {
                    ast::Selection::FragmentSpread(Node::new_synthetic(ast::FragmentSpread {
                        fragment_name: fragment_spread.fragment_name.clone(),
                        directives: fragment_spread.directives.clone(),
                    }))
                }
                Selection::InlineFragment(inline_fragment) => {
                    ast::Selection::InlineFragment(Node::new_synthetic(ast::InlineFragment {
                        type_condition: inline_fragment.type_condition.clone(),
                        directives: inline_fragment.directives.clone(),
                        selection_set: inline_fragment.selection_set.to_ast(),
                    }))
                }
            })
            .collect()
    }
}
