use crate::DocumentBuilder;
use apollo_compiler::ast::Argument;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Create arbitrary arguments matching their definitions.
    pub fn arguments_with_def(
        &mut self,
        args_def: &[Node<InputValueDefinition>],
    ) -> ArbitraryResult<Vec<Argument>> {
        let arguments = args_def
            .iter()
            .map(|input_val_def| self.argument_with_def(input_val_def))
            .collect::<ArbitraryResult<Vec<_>>>()?;

        Ok(arguments)
    }

    /// Create an arbitrary `Argument`
    pub fn argument_with_def(
        &mut self,
        input_val_def: &InputValueDefinition,
    ) -> ArbitraryResult<Argument> {
        let name = input_val_def.name.clone();
        let value = self.input_value_for_type(&input_val_def.ty)?;

        Ok(Argument {
            name,
            value: Node::new(value),
        })
    }

    /// Create arbitrary argument definitions.
    pub fn arguments_definition(&mut self) -> ArbitraryResult<Vec<Node<InputValueDefinition>>> {
        self.input_values_def(
            DirectiveLocation::ArgumentDefinition,
            &IndexSet::new(),
            None,
        )
        .map(|definitions| definitions.into_iter().map(Node::new).collect())
    }
}
