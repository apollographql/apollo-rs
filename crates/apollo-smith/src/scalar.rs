use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    DocumentBuilder,
};
use apollo_compiler::ast;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// Represents scalar types such as Int, String, and Boolean.
/// Scalars cannot have fields.
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Scalar).
#[derive(Debug, Clone)]
pub struct ScalarTypeDef {
    pub(crate) name: Name,
    pub(crate) description: Option<Description>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) extend: bool,
}

impl From<ScalarTypeDef> for ast::Definition {
    fn from(x: ScalarTypeDef) -> Self {
        if x.extend {
            ast::ScalarTypeExtension {
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
            }
            .into()
        } else {
            ast::ScalarTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::ScalarTypeDefinition> for ScalarTypeDef {
    type Error = crate::FromError;

    fn try_from(scalar_def: apollo_parser::cst::ScalarTypeDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            description: scalar_def
                .description()
                .and_then(|d| d.string_value())
                .map(|s| Description::from(Into::<String>::into(s))),
            name: scalar_def.name().unwrap().into(),
            directives: scalar_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: false,
        })
    }
}

impl TryFrom<apollo_parser::cst::ScalarTypeExtension> for ScalarTypeDef {
    type Error = crate::FromError;

    fn try_from(scalar_def: apollo_parser::cst::ScalarTypeExtension) -> Result<Self, Self::Error> {
        Ok(Self {
            description: None,
            name: scalar_def.name().unwrap().into(),
            directives: scalar_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: true,
        })
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `ScalarTypeDef`
    pub fn scalar_type_definition(&mut self) -> ArbitraryResult<ScalarTypeDef> {
        let extend = !self.scalar_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let name = if extend {
            let available_scalars: Vec<&Name> = self
                .scalar_type_defs
                .iter()
                .filter_map(|scalar| {
                    if scalar.extend {
                        None
                    } else {
                        Some(&scalar.name)
                    }
                })
                .collect();
            (*self.u.choose(&available_scalars)?).clone()
        } else {
            self.type_name()?
        };
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let directives = self.directives(DirectiveLocation::Scalar)?;
        // Extended scalar must have directive
        let extend = !directives.is_empty() && self.u.arbitrary().unwrap_or(false);

        Ok(ScalarTypeDef {
            name,
            description,
            directives,
            extend,
        })
    }
}

pub(crate) fn link_import() -> ScalarTypeDef {
    ScalarTypeDef {
        name: Name::new(String::from("link__Import")),
        description: None,
        directives: Default::default(),
        extend: false,
    }
}
