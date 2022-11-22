use std::collections::HashMap;

use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    DocumentBuilder,
};

/// Represents scalar types such as Int, String, and Boolean.
/// Scalars cannot have fields.
///
/// *ScalarTypeDefinition*:
///     Description? **scalar** Name Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Scalar).
#[derive(Debug)]
pub struct ScalarTypeDef {
    pub(crate) name: Name,
    pub(crate) description: Option<Description>,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) extend: bool,
}

impl From<ScalarTypeDef> for apollo_encoder::ScalarDefinition {
    fn from(scalar_def: ScalarTypeDef) -> Self {
        let mut new_scalar_def = Self::new(scalar_def.name.into());
        if let Some(description) = scalar_def.description {
            new_scalar_def.description(description.into());
        }
        scalar_def
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_scalar_def.directive(directive.into()));
        if scalar_def.extend {
            new_scalar_def.extend();
        }

        new_scalar_def
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::ScalarTypeDefinition> for ScalarTypeDef {
    type Error = crate::FromError;

    fn try_from(scalar_def: apollo_parser::ast::ScalarTypeDefinition) -> Result<Self, Self::Error> {
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

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::ScalarTypeExtension> for ScalarTypeDef {
    type Error = crate::FromError;

    fn try_from(scalar_def: apollo_parser::ast::ScalarTypeExtension) -> Result<Self, Self::Error> {
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
    pub fn scalar_type_definition(&mut self) -> Result<ScalarTypeDef> {
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
