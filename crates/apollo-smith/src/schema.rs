use std::collections::HashMap;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    ty::Ty,
    DocumentBuilder,
};
use arbitrary::Result;

/// A GraphQL service’s collective type system capabilities are referred to as that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Schema).
#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub(crate) description: Option<Description>,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) query: Option<Ty>,
    pub(crate) mutation: Option<Ty>,
    pub(crate) subscription: Option<Ty>,
    pub(crate) extend: bool,
}

impl From<SchemaDef> for apollo_encoder::SchemaDefinition {
    fn from(schema_def: SchemaDef) -> Self {
        let mut new_schema_def = Self::new();
        if let Some(description) = schema_def.description {
            new_schema_def.description(description.into())
        }
        schema_def
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_schema_def.directive(directive.into()));
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

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::SchemaDefinition> for SchemaDef {
    type Error = crate::FromError;

    fn try_from(schema_def: apollo_parser::ast::SchemaDefinition) -> Result<Self, Self::Error> {
        let mut query = None;
        let mut mutation = None;
        let mut subcription = None;
        for root_op in schema_def.root_operation_type_definitions() {
            let op_type = root_op.operation_type().unwrap();
            let named_type = root_op.named_type().unwrap();
            if op_type.query_token().is_some() {
                query = named_type.into();
            } else if op_type.mutation_token().is_some() {
                mutation = named_type.into();
            } else if op_type.subscription_token().is_some() {
                subcription = named_type.into();
            } else {
                panic!("operation type must be one of query|mutation|subscription");
            }
        }
        Ok(Self {
            description: schema_def.description().map(Description::from),
            directives: schema_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            query: query.map(Ty::from),
            mutation: mutation.map(Ty::from),
            subscription: subcription.map(Ty::from),
            extend: false,
        })
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::SchemaExtension> for SchemaDef {
    type Error = crate::FromError;

    fn try_from(schema_def: apollo_parser::ast::SchemaExtension) -> Result<Self, Self::Error> {
        let mut query = None;
        let mut mutation = None;
        let mut subcription = None;
        for root_op in schema_def.root_operation_type_definitions() {
            let op_type = root_op.operation_type().unwrap();
            let named_type = root_op.named_type().unwrap();
            if op_type.query_token().is_some() {
                query = named_type.into();
            } else if op_type.mutation_token().is_some() {
                mutation = named_type.into();
            } else if op_type.subscription_token().is_some() {
                subcription = named_type.into();
            } else {
                panic!("operation type must be one of query|mutation|subscription");
            }
        }
        Ok(Self {
            description: None,
            directives: schema_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            query: query.map(Ty::from),
            mutation: mutation.map(Ty::from),
            subscription: subcription.map(Ty::from),
            extend: true,
        })
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
        let directives = self.directives(DirectiveLocation::Schema)?;
        let named_types: Vec<Ty> = self
            .list_existing_object_types()
            .into_iter()
            .filter(|ty| ty.is_named() && !ty.is_builtin())
            .collect();

        let arbitrary_idx: usize = self.u.arbitrary::<usize>()?;

        let mut query = (arbitrary_idx % 2 == 0)
            .then(|| self.u.choose(&named_types))
            .transpose()?
            .cloned();
        let mut mutation = (arbitrary_idx % 3 == 0)
            .then(|| self.u.choose(&named_types))
            .transpose()?
            .cloned();
        let mut subscription = (arbitrary_idx % 5 == 0)
            .then(|| self.u.choose(&named_types))
            .transpose()?
            .cloned();
        // If no one has been filled
        if let (None, None, None) = (&query, &mutation, &subscription) {
            let arbitrary_op_type_idx = self.u.int_in_range(0..=2usize)?;
            match arbitrary_op_type_idx {
                0 => query = Some(self.u.choose(&named_types)?.clone()),
                1 => mutation = Some(self.u.choose(&named_types)?.clone()),
                2 => subscription = Some(self.u.choose(&named_types)?.clone()),
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
