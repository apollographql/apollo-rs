use crate::{
    argument::{Argument, ArgumentsDef},
    description::Description,
    name::Name,
    DocumentBuilder,
};
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::{Arbitrary, Result as ArbitraryResult};
use indexmap::{IndexMap, IndexSet};

/// The `__DirectiveDef` type represents a Directive definition.
///
/// *DirectiveDefinition*:
///     Description? **directive @** Name Arguments Definition? **repeatable**? **on** DirectiveLocations
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Type-System.Directives).
#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) arguments_definition: Option<ArgumentsDef>,
    pub(crate) repeatable: bool,
    pub(crate) directive_locations: IndexSet<DirectiveLocation>,
}

impl From<DirectiveDef> for ast::Definition {
    fn from(dir_def: DirectiveDef) -> Self {
        ast::DirectiveDefinition {
            description: dir_def.description.map(Into::into),
            name: dir_def.name.into(),
            arguments: dir_def
                .arguments_definition
                .map(Into::into)
                .unwrap_or_default(),
            repeatable: dir_def.repeatable,
            locations: dir_def
                .directive_locations
                .into_iter()
                .map(Into::into)
                .collect(),
        }
        .into()
    }
}

impl TryFrom<apollo_parser::cst::DirectiveDefinition> for DirectiveDef {
    type Error = crate::FromError;

    fn try_from(
        directive_def: apollo_parser::cst::DirectiveDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            description: directive_def
                .description()
                .and_then(|d| d.string_value())
                .map(|s| Description::from(Into::<String>::into(s))),
            name: directive_def.name().unwrap().into(),
            arguments_definition: directive_def
                .arguments_definition()
                .map(ArgumentsDef::try_from)
                .transpose()?,
            repeatable: directive_def.repeatable_token().is_some(),
            directive_locations: directive_def
                .directive_locations()
                .map(|dls| {
                    dls.directive_locations()
                        .map(|dl| DirectiveLocation::from(dl.text().unwrap().to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

/// The `__Directive` type represents a Directive, it provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.
///
/// *Directive*:
///     @ Name Arguments?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Directives).
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub(crate) name: Name,
    pub(crate) arguments: Vec<Argument>,
}

impl From<Directive> for ast::Directive {
    fn from(directive: Directive) -> Self {
        Self {
            name: directive.name.into(),
            arguments: directive
                .arguments
                .into_iter()
                .map(|a| Node::new(a.into()))
                .collect(),
        }
    }
}

impl TryFrom<apollo_parser::cst::Directive> for Directive {
    type Error = crate::FromError;

    fn try_from(directive: apollo_parser::cst::Directive) -> Result<Self, Self::Error> {
        Ok(Self {
            name: directive.name().unwrap().into(),
            arguments: directive
                .arguments()
                .map(|args| {
                    args.arguments()
                        .map(Argument::try_from)
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl Directive {
    pub(crate) fn convert_directives(
        directives: apollo_parser::cst::Directives,
    ) -> Result<IndexMap<Name, Directive>, crate::FromError> {
        directives
            .directives()
            .map(|d| Ok((d.name().unwrap().into(), Directive::try_from(d)?)))
            .collect()
    }

    pub(crate) fn to_ast(map: IndexMap<Name, Directive>) -> ast::DirectiveList {
        map.into_values().map(ast::Directive::from).collect()
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary vector of `Directive`
    pub fn directives(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<IndexMap<Name, Directive>> {
        if self.directive_defs.is_empty() {
            return Ok(IndexMap::new());
        }

        let num_directives = self.u.int_in_range(0..=(self.directive_defs.len() - 1))?;
        let directives = (0..num_directives)
            .map(|_| self.directive(directive_location))
            .collect::<ArbitraryResult<Vec<_>>>()?
            .into_iter()
            .flat_map(|d| d.map(|d| (d.name.clone(), d)))
            .collect();

        Ok(directives)
    }

    /// Create an arbitrary `Directive` given a directive location
    pub fn directive(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<Option<Directive>> {
        let available_directive_defs: Vec<&DirectiveDef> = self
            .directive_defs
            .iter()
            .filter(|dd| {
                dd.directive_locations.is_empty()
                    || dd.directive_locations.contains(&directive_location)
            })
            .collect();
        if available_directive_defs.is_empty() {
            return Ok(None);
        }
        let directive_def = self.u.choose(&available_directive_defs)?;

        let name = directive_def.name.clone();
        let arguments = directive_def
            .arguments_definition
            .clone()
            .map(|args_def| self.arguments_with_def(&args_def))
            .unwrap_or_else(|| Ok(vec![]))?;

        Ok(Some(Directive { name, arguments }))
    }

    /// Create an arbitrary `DirectiveDef`
    pub fn directive_def(&mut self) -> ArbitraryResult<DirectiveDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.type_name()?;
        let arguments_definition = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.arguments_definition())
            .transpose()?;
        let repeatable = self.u.arbitrary().unwrap_or(false);
        let directive_locations = self.directive_locations()?;

        Ok(DirectiveDef {
            description,
            name,
            arguments_definition,
            repeatable,
            directive_locations,
        })
    }

    /// Create an arbitrary `IndexSet` of `DirectiveLocation`
    pub fn directive_locations(&mut self) -> ArbitraryResult<IndexSet<DirectiveLocation>> {
        (1..self.u.int_in_range(2..=5usize)?)
            .map(|_| self.u.arbitrary())
            .collect::<ArbitraryResult<IndexSet<_>>>()
    }
}

/// The `__DirectiveLocation` type represents a Directive location.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Arbitrary, Copy)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

impl From<DirectiveLocation> for ast::DirectiveLocation {
    fn from(dl: DirectiveLocation) -> Self {
        match dl {
            DirectiveLocation::Query => Self::Query,
            DirectiveLocation::Mutation => Self::Mutation,
            DirectiveLocation::Subscription => Self::Subscription,
            DirectiveLocation::Field => Self::Field,
            DirectiveLocation::FragmentDefinition => Self::FragmentDefinition,
            DirectiveLocation::FragmentSpread => Self::FragmentSpread,
            DirectiveLocation::InlineFragment => Self::InlineFragment,
            DirectiveLocation::VariableDefinition => Self::VariableDefinition,
            DirectiveLocation::Schema => Self::Schema,
            DirectiveLocation::Scalar => Self::Scalar,
            DirectiveLocation::Object => Self::Object,
            DirectiveLocation::FieldDefinition => Self::FieldDefinition,
            DirectiveLocation::ArgumentDefinition => Self::ArgumentDefinition,
            DirectiveLocation::Interface => Self::Interface,
            DirectiveLocation::Union => Self::Union,
            DirectiveLocation::Enum => Self::Enum,
            DirectiveLocation::EnumValue => Self::EnumValue,
            DirectiveLocation::InputObject => Self::InputObject,
            DirectiveLocation::InputFieldDefinition => Self::InputFieldDefinition,
        }
    }
}

impl From<DirectiveLocation> for String {
    fn from(dl: DirectiveLocation) -> Self {
        match dl {
            DirectiveLocation::Query => String::from("QUERY"),
            DirectiveLocation::Mutation => String::from("MUTATION"),
            DirectiveLocation::Subscription => String::from("SUBSCRIPTION"),
            DirectiveLocation::Field => String::from("FIELD"),
            DirectiveLocation::FragmentDefinition => String::from("FRAGMENT_DEFINITION"),
            DirectiveLocation::FragmentSpread => String::from("FRAGMENT_SPREAD"),
            DirectiveLocation::InlineFragment => String::from("INLINE_FRAGMENT"),
            DirectiveLocation::VariableDefinition => String::from("VARIABLE_DEFINITION"),
            DirectiveLocation::Schema => String::from("SCHEMA"),
            DirectiveLocation::Scalar => String::from("SCALAR"),
            DirectiveLocation::Object => String::from("OBJECT"),
            DirectiveLocation::FieldDefinition => String::from("FIELD_DEFINITION"),
            DirectiveLocation::ArgumentDefinition => String::from("ARGUMENT_DEFINITION"),
            DirectiveLocation::Interface => String::from("INTERFACE"),
            DirectiveLocation::Union => String::from("UNION"),
            DirectiveLocation::Enum => String::from("ENUM"),
            DirectiveLocation::EnumValue => String::from("ENUM_VALUE"),
            DirectiveLocation::InputObject => String::from("INPUT_OBJECT"),
            DirectiveLocation::InputFieldDefinition => String::from("INPUT_FIELD_DEFINITION"),
        }
    }
}

impl From<String> for DirectiveLocation {
    fn from(dl: String) -> Self {
        match dl.as_str() {
            "QUERY" => DirectiveLocation::Query,
            "MUTATION" => DirectiveLocation::Mutation,
            "SUBSCRIPTION" => DirectiveLocation::Subscription,
            "FIELD" => DirectiveLocation::Field,
            "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
            "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
            "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
            "VARIABLE_DEFINITION" => DirectiveLocation::VariableDefinition,
            "SCHEMA" => DirectiveLocation::Schema,
            "SCALAR" => DirectiveLocation::Scalar,
            "OBJECT" => DirectiveLocation::Object,
            "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
            "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
            "INTERFACE" => DirectiveLocation::Interface,
            "UNION" => DirectiveLocation::Union,
            "ENUM" => DirectiveLocation::Enum,
            "ENUM_VALUE" => DirectiveLocation::EnumValue,
            "INPUT_OBJECT" => DirectiveLocation::InputObject,
            "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
            other => unreachable!(
                "cannot have {} as a directive location. Documentation: https://spec.graphql.org/October2021/#DirectiveLocation",
                other
            ),
        }
    }
}
