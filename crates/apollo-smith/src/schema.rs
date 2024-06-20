use crate::{
    argument::Argument,
    description::Description,
    directive::{Directive, DirectiveLocation},
    input_value::InputValue,
    name::Name,
    ty::Ty,
    DocumentBuilder,
};
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// A GraphQL service’s collective type system capabilities are referred to as that service’s “schema”.
///
/// *SchemaDefinition*:
///     Description? **schema** Directives? **{** RootOperationTypeDefinition* **}**
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Schema).
#[derive(Debug, Clone)]
pub struct SchemaDef {
    pub(crate) description: Option<Description>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) query: Option<Ty>,
    pub(crate) mutation: Option<Ty>,
    pub(crate) subscription: Option<Ty>,
    pub(crate) extend: bool,
}

impl From<SchemaDef> for ast::Definition {
    fn from(x: SchemaDef) -> Self {
        let root_operations = [
            (ast::OperationType::Query, x.query),
            (ast::OperationType::Mutation, x.mutation),
            (ast::OperationType::Subscription, x.subscription),
        ]
        .into_iter()
        .flat_map(|(ty, name)| name.map(|n| Node::new((ty, n.name().into()))))
        .collect();
        if x.extend {
            ast::SchemaExtension {
                directives: Directive::to_ast(x.directives),
                root_operations,
            }
            .into()
        } else {
            ast::SchemaDefinition {
                description: x.description.map(Into::into),
                directives: Directive::to_ast(x.directives),
                root_operations,
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::SchemaDefinition> for SchemaDef {
    type Error = crate::FromError;

    fn try_from(schema_def: apollo_parser::cst::SchemaDefinition) -> Result<Self, Self::Error> {
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

impl TryFrom<apollo_parser::cst::SchemaExtension> for SchemaDef {
    type Error = crate::FromError;

    fn try_from(schema_def: apollo_parser::cst::SchemaExtension) -> Result<Self, Self::Error> {
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
    pub fn schema_definition(&mut self) -> ArbitraryResult<SchemaDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let mut directives = self.directives(DirectiveLocation::Schema)?;
        if self.is_supergraph {
            let link_arg = Argument {
                name: Name::new(String::from("url")),
                value: InputValue::String("https://specs.apollo.dev/link/v1.0".into()),
            };
            let directive = Directive {
                name: Name::new(String::from("link")),
                arguments: vec![link_arg],
            };
            directives.insert(Name::new("link".to_string()), directive.into());
        }
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
