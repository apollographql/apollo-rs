use std::collections::HashMap;

use arbitrary::Result as ArbitraryResult;

use crate::{
    directive::{Directive, DirectiveLocation},
    input_value::InputValue,
    name::Name,
    ty::Ty,
    DocumentBuilder,
};

/// The __variableDef type represents a variable definition
///
/// *VariableDefinition*:
///     VariableName : Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Variables).
#[derive(Debug)]
pub struct VariableDef {
    name: Name,
    ty: Ty,
    default_value: Option<InputValue>,
    directives: HashMap<Name, Directive>,
}

impl From<VariableDef> for apollo_encoder::VariableDefinition {
    fn from(var_def: VariableDef) -> Self {
        let mut new_var_def = Self::new(var_def.name.into(), var_def.ty.into());
        if let Some(default) = var_def.default_value {
            new_var_def.default_value(default.into())
        }
        var_def
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_var_def.directive(directive.into()));

        new_var_def
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary list of `VariableDef`
    pub fn variable_definitions(&mut self) -> ArbitraryResult<Vec<VariableDef>> {
        (0..self.u.int_in_range(0..=7usize)?)
            .map(|_| self.variable_definition()) // TODO do not generate duplication variable name
            .collect()
    }

    /// Create an arbitrary `VariableDef`
    pub fn variable_definition(&mut self) -> ArbitraryResult<VariableDef> {
        let name = self.type_name()?;
        let ty = self.choose_ty(&self.list_existing_types())?;
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value())
            .transpose()?;
        let directives = self.directives(DirectiveLocation::VariableDefinition)?;

        Ok(VariableDef {
            name,
            ty,
            default_value,
            directives,
        })
    }
}
