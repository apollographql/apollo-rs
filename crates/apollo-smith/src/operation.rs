use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::selection_set::Selection;
use crate::selection_set::SelectionSet;
use crate::variable::VariableDef;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Arbitrary;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

/// The __operationDef type represents an operation definition
///
/// *OperationDefinition*:
///     OperationType Name? VariableDefinitions? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Operations).
#[derive(Debug, Clone)]
pub struct OperationDef {
    pub(crate) operation_type: OperationType,
    pub(crate) name: Option<Name>,
    pub(crate) variable_definitions: Vec<VariableDef>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) selection_set: SelectionSet,
}

impl From<OperationDef> for ast::Definition {
    fn from(x: OperationDef) -> Self {
        ast::OperationDefinition {
            operation_type: x.operation_type.into(),
            name: x.name.map(Into::into),
            directives: Directive::to_ast(x.directives),
            variables: x
                .variable_definitions
                .into_iter()
                .map(|x| Node::new(x.into()))
                .collect(),
            selection_set: x.selection_set.into(),
        }
        .into()
    }
}

impl From<OperationDef> for String {
    fn from(op_def: OperationDef) -> Self {
        ast::Definition::from(op_def).to_string()
    }
}

impl TryFrom<apollo_parser::cst::OperationDefinition> for OperationDef {
    type Error = crate::FromError;

    fn try_from(
        operation_def: apollo_parser::cst::OperationDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: operation_def.name().map(Name::from),
            directives: operation_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            operation_type: operation_def
                .operation_type()
                .map(OperationType::from)
                .unwrap_or(OperationType::Query),
            variable_definitions: Vec::new(),
            selection_set: operation_def.selection_set().unwrap().try_into()?,
        })
    }
}

/// The __operationType type represents the kind of operation
///
/// *OperationType*:
///     query | mutation | subscription
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug, Arbitrary, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl From<OperationType> for ast::OperationType {
    fn from(op_type: OperationType) -> Self {
        match op_type {
            OperationType::Query => Self::Query,
            OperationType::Mutation => Self::Mutation,
            OperationType::Subscription => Self::Subscription,
        }
    }
}

impl From<apollo_parser::cst::OperationType> for OperationType {
    fn from(op_type: apollo_parser::cst::OperationType) -> Self {
        if op_type.query_token().is_some() {
            Self::Query
        } else if op_type.mutation_token().is_some() {
            Self::Mutation
        } else if op_type.subscription_token().is_some() {
            Self::Subscription
        } else {
            Self::Query
        }
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `OperationDef` taking the last `SchemaDef`.
    pub fn operation_definition(&mut self) -> ArbitraryResult<Option<OperationDef>> {
        self.operation_definition_in_document(false)
    }

    /// Like [`operation_definition`], but forces the operation to be
    /// named when `require_named` is true. A document with more than
    /// one operation may not contain an anonymous operation.
    ///
    /// Operation Name Uniqueness across the document is upheld by
    /// `type_name`, which never returns a name that already exists on
    /// the builder.
    ///
    /// See <https://spec.graphql.org/October2021/#sec-Lone-Anonymous-Operation>
    /// and <https://spec.graphql.org/October2021/#sec-Operation-Name-Uniqueness>.
    ///
    /// [`operation_definition`]: Self::operation_definition
    pub(crate) fn operation_definition_in_document(
        &mut self,
        require_named: bool,
    ) -> ArbitraryResult<Option<OperationDef>> {
        let schema = match self.schema_def.clone() {
            Some(schema_def) => schema_def,
            None => return Ok(None),
        };

        let want_name = require_named || self.u.arbitrary().unwrap_or(false);
        let name = if want_name {
            Some(self.type_name()?)
        } else {
            None
        };

        let available_operations = {
            let mut ops = vec![];
            if let Some(query) = &schema.query {
                ops.push((OperationType::Query, query));
            }
            if let Some(mutation) = &schema.mutation {
                ops.push((OperationType::Mutation, mutation));
            }
            if let Some(subscription) = &schema.subscription {
                ops.push((OperationType::Subscription, subscription));
            }

            ops
        };

        let (operation_type, chosen_ty) = self.u.choose(&available_operations)?;
        let directive_location = match operation_type {
            OperationType::Query => DirectiveLocation::Query,
            OperationType::Mutation => DirectiveLocation::Mutation,
            OperationType::Subscription => DirectiveLocation::Subscription,
        };
        let directives = self.directives(directive_location)?;

        // Stack
        self.stack_ty(chosen_ty);

        let selection_set = if matches!(operation_type, OperationType::Subscription) {
            // Subscription operations must have exactly one root field
            // per <https://spec.graphql.org/October2021/#sec-Single-root-field>.
            // We generate exactly one field to satisfy that condition. The 0-index
            // is sometimes used to create an alias for the field.
            SelectionSet {
                selections: vec![Selection::Field(self.field(0)?)],
            }
        } else {
            self.selection_set()?
        };

        self.stack.pop();
        assert!(
            self.stack.is_empty(),
            "the stack must be empty at the end of an operation definition"
        );

        // TODO
        let variable_definitions = vec![];

        Ok(Some(OperationDef {
            operation_type: *operation_type,
            name,
            variable_definitions,
            directives,
            selection_set,
        }))
    }
}
