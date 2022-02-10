use arbitrary::Result;

use crate::{description::Description, directive::Directive, name::Name, DocumentBuilder};

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
    pub(crate) directives: Vec<Directive>,
    pub(crate) extend: bool,
}

impl From<ScalarTypeDef> for apollo_encoder::ScalarDef {
    fn from(scalar_def: ScalarTypeDef) -> Self {
        let mut new_scalar_def = Self::new(scalar_def.name.into());
        new_scalar_def.description(scalar_def.description.map(String::from));
        scalar_def
            .directives
            .into_iter()
            .for_each(|directive| new_scalar_def.directive(directive.into()));
        if scalar_def.extend {
            new_scalar_def.extend();
        }

        new_scalar_def
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `ScalarTypeDef`
    pub fn scalar_type_definition(&mut self) -> Result<ScalarTypeDef> {
        let name = self.type_name()?;
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let directives = self.directives()?;
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
