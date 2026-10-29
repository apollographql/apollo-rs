use crate::DocumentBuilder;
use apollo_compiler::ast::Argument;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::ast::Type;
use apollo_compiler::ast::Value;
use apollo_compiler::ast::VariableDefinition;
use apollo_compiler::coordinate::TypeAttributeCoordinate;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// Variables defined so far in the operation being generated, and the
/// arguments chosen for fields within it that use them.
#[derive(Debug, Default)]
pub(crate) struct OperationVariables {
    pub(crate) definitions: Vec<VariableDefinition>,
    pub(crate) arguments: IndexMap<TypeAttributeCoordinate, Vec<Argument>>,
}

impl DocumentBuilder<'_> {
    /// Generate arguments for a field. Within an operation, some arguments
    /// pass one of its variables, defined on first use so that every
    /// variable is both defined and used.
    pub(crate) fn field_arguments(
        &mut self,
        args_def: &[Node<InputValueDefinition>],
    ) -> ArbitraryResult<Vec<Argument>> {
        args_def
            .iter()
            .map(|def| {
                if self.operation_variables.is_some() && self.u.ratio(1, 3)? {
                    Ok(Argument {
                        name: def.name.clone(),
                        value: Node::new(Value::Variable(self.variable_for(&def.ty)?)),
                    })
                } else {
                    self.argument_with_def(def)
                }
            })
            .collect()
    }

    /// Pick or define a variable that can be passed where `ty` is expected.
    pub(crate) fn variable_for(&mut self, ty: &Type) -> ArbitraryResult<Name> {
        // A variable can be passed where its own type, or the nullable form
        // of it, is expected.
        let non_null = (ty.clone()).non_null();
        let compatible: Vec<Name> = self
            .operation_variables
            .iter()
            .flat_map(|vars| &vars.definitions)
            .filter(|v| &*v.ty == ty || *v.ty == non_null)
            .map(|v| v.name.clone())
            .collect();
        if !compatible.is_empty() && self.u.arbitrary()? {
            return Ok(self.u.choose(&compatible)?.clone());
        }

        let name = self.type_name()?;
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value_for_type(ty))
            .transpose()?;
        let directives = self.directives(DirectiveLocation::VariableDefinition)?;
        if let Some(vars) = &mut self.operation_variables {
            vars.definitions.push(VariableDefinition {
                description: None,
                name: name.clone(),
                ty: Node::new(ty.clone()),
                default_value: default_value.map(Node::new),
                directives,
            });
        }
        Ok(name)
    }

    /// How many variables the current operation has defined, so variables
    /// defined after this point can be dropped with `rollback_variables`.
    pub(crate) fn variable_checkpoint(&self) -> Option<usize> {
        self.operation_variables
            .as_ref()
            .map(|vars| vars.definitions.len())
    }

    /// Drop variables defined since `checkpoint`, for arguments that end up
    /// unused.
    pub(crate) fn rollback_variables(&mut self, checkpoint: Option<usize>) {
        if let (Some(vars), Some(count)) = (&mut self.operation_variables, checkpoint) {
            vars.definitions.truncate(count);
        }
    }

    /// Arguments chosen earlier for a field, preferring those of the current
    /// operation, which may pass its variables.
    pub(crate) fn cached_arguments(
        &self,
        coord: &TypeAttributeCoordinate,
    ) -> Option<&Vec<Argument>> {
        self.operation_variables
            .as_ref()
            .and_then(|vars| vars.arguments.get(coord))
            .or_else(|| self.chosen_arguments.get(coord))
    }

    /// Arguments that pass variables only make sense within the current
    /// operation, so they aren't reused by other operations or fragments.
    pub(crate) fn cache_arguments(&mut self, coord: TypeAttributeCoordinate, args: Vec<Argument>) {
        let uses_variables = args
            .iter()
            .any(|arg| matches!(*arg.value, Value::Variable(_)));
        match &mut self.operation_variables {
            Some(vars) if uses_variables => {
                vars.arguments.insert(coord, args);
            }
            _ => {
                self.chosen_arguments.insert(coord, args);
            }
        }
    }

    /// Create an arbitrary list of `VariableDefinition`
    pub fn variable_definitions(&mut self) -> ArbitraryResult<Vec<VariableDefinition>> {
        (0..self.u.int_in_range(0..=7usize)?)
            .map(|_| self.variable_definition())
            .collect()
    }

    /// Create an arbitrary `VariableDefinition`
    pub fn variable_definition(&mut self) -> ArbitraryResult<VariableDefinition> {
        let name = self.type_name()?;
        let ty = self.choose_ty(&self.list_existing_input_types())?;
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value_for_type(&ty))
            .transpose()?;
        let directives = self.directives(DirectiveLocation::VariableDefinition)?;

        Ok(VariableDefinition {
            description: None,
            name,
            ty: Node::new(ty),
            default_value: default_value.map(Node::new),
            directives,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generated;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    const SCHEMA: &str = "schema { query: Query }
        type Query { f(a: Int, b: String!, c: [Int!]): Int, g(a: Int!): Query }";

    proptest! {
        #[test]
        fn operations_use_valid_variables(case in generated(SCHEMA, |b| b.operation_definition())) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }
}
