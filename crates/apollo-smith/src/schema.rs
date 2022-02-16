use crate::{description::Description, directive::Directive, ty::Ty, DocumentBuilder};
use arbitrary::Result;

/// A GraphQL service’s collective type system capabilities are referred to as that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Schema).
#[derive(Debug)]
pub struct SchemaDef {
    pub(crate) description: Option<Description>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) query: Option<Ty>,
    pub(crate) mutation: Option<Ty>,
    pub(crate) subscription: Option<Ty>,
    pub(crate) extend: bool,
}

impl From<SchemaDef> for apollo_encoder::SchemaDefinition {
    fn from(schema_def: SchemaDef) -> Self {
        let mut new_schema_def = Self::new();
        new_schema_def.description(schema_def.description.map(String::from));
        schema_def
            .directives
            .into_iter()
            .for_each(|directive| new_schema_def.directive(directive.into()));
        if let Some(query) = schema_def.query {
            new_schema_def.query(apollo_encoder::Type_::from(query).to_string());
        }
        if let Some(mutation) = schema_def.mutation {
            new_schema_def.mutation(apollo_encoder::Type_::from(mutation).to_string());
        }
        if let Some(subscription) = schema_def.subscription {
            new_schema_def.subscription(apollo_encoder::Type_::from(subscription).to_string());
        }
        if schema_def.extend {
            new_schema_def.extend();
        }

        new_schema_def
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `SchemaDef`
    pub fn schema_definition(&mut self) -> Result<SchemaDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let directives = self.directives()?;
        let named_types: Vec<Ty> = self
            .list_existing_object_types()
            .into_iter()
            .filter(Ty::is_named)
            .collect();

        let arbitrary_idx: usize = self.u.arbitrary::<usize>()?;

        let mut query = (arbitrary_idx % 2 == 0)
            .then(|| self.choose_named_ty(&named_types))
            .transpose()?;
        let mut mutation = (arbitrary_idx % 3 == 0)
            .then(|| self.choose_named_ty(&named_types))
            .transpose()?;
        let mut subscription = (arbitrary_idx % 5 == 0)
            .then(|| self.choose_named_ty(&named_types))
            .transpose()?;
        // If no one has been filled
        if let (None, None, None) = (&query, &mutation, &subscription) {
            match self.u.int_in_range(0..=2usize)? {
                0 => query = Some(self.choose_named_ty(&named_types)?),
                1 => mutation = Some(self.choose_named_ty(&named_types)?),
                2 => subscription = Some(self.choose_named_ty(&named_types)?),
                _ => unreachable!(),
            }
        }

        Ok(SchemaDef {
            description,
            directives,
            query,
            mutation,
            subscription,
            extend: self.u.arbitrary().unwrap_or(false),
        })
    }
}
