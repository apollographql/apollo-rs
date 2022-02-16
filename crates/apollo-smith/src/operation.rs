use arbitrary::{Arbitrary, Result};

use crate::{
    directive::Directive, name::Name, selection_set::SelectionSet, variable::VariableDef,
    DocumentBuilder,
};

/// The __operationDef type represents an operation definition
///
/// *OperationDefinition*:
///     OperationType Name? VariableDefinitions? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Operations).
#[derive(Debug)]
pub struct OperationDef {
    pub(crate) operation_type: OperationType,
    pub(crate) name: Option<Name>,
    pub(crate) variable_definitions: Vec<VariableDef>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) shorthand: bool,
}

impl From<OperationDef> for apollo_encoder::OperationDefinition {
    fn from(op_def: OperationDef) -> Self {
        let mut new_op_def = Self::new(op_def.operation_type.into(), op_def.selection_set.into());
        new_op_def.name(op_def.name.map(String::from));
        op_def
            .variable_definitions
            .into_iter()
            .for_each(|var_def| new_op_def.variable_definition(var_def.into()));
        op_def.shorthand.then(|| new_op_def.shorthand());
        op_def
            .directives
            .into_iter()
            .for_each(|directive| new_op_def.directive(directive.into()));

        new_op_def
    }
}

/// The __operationType type represents the kind of operation
///
/// *OperationType*:
///     query | mutation | subscription
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug, Arbitrary)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl From<OperationType> for apollo_encoder::OperationType {
    fn from(op_type: OperationType) -> Self {
        match op_type {
            OperationType::Query => Self::Query,
            OperationType::Mutation => Self::Mutation,
            OperationType::Subscription => Self::Subscription,
        }
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `OperationDef`
    pub fn operation_definition(&mut self) -> Result<OperationDef> {
        let name = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.type_name())
            .transpose()?;
        let operation_type = self.u.arbitrary()?;
        let directives = self.directives()?;
        let selection_set = self.selection_set()?;
        let variable_definitions = self.variable_definitions()?;
        let shorthand = self.operation_defs.is_empty() && self.u.arbitrary().unwrap_or(false);

        Ok(OperationDef {
            operation_type,
            name,
            variable_definitions,
            directives,
            selection_set,
            shorthand,
        })
    }
}
