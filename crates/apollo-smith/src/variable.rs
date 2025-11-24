use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::input_value::Constness;
use crate::input_value::InputValue;
use crate::name::Name;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// The __variableDef type represents a variable definition
///
/// *VariableDefinition*:
///     VariableName : Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Variables).
#[derive(Debug, Clone)]
pub struct VariableDef {
    name: Name,
    ty: Ty,
    default_value: Option<InputValue>,
    directives: IndexMap<Name, Directive>,
}

impl From<VariableDef> for ast::VariableDefinition {
    fn from(x: VariableDef) -> Self {
        Self {
            name: x.name.into(),
            ty: Node::new(x.ty.into()),
            default_value: x.default_value.map(|x| Node::new(x.into())),
            directives: Directive::to_ast(x.directives),
        }
    }
}

impl DocumentBuilder<'_> {
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
            .then(|| self.input_value(Constness::Const))
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
