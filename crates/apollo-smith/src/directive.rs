use std::collections::{HashMap, HashSet};

use arbitrary::{Arbitrary, Result};

use crate::{
    argument::{Argument, ArgumentsDef},
    description::Description,
    name::Name,
    DocumentBuilder,
};

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
    pub(crate) directive_locations: HashSet<DirectiveLocation>,
}

impl From<DirectiveDef> for apollo_encoder::DirectiveDefinition {
    fn from(dir_def: DirectiveDef) -> Self {
        let mut new_dir_def = Self::new(dir_def.name.into());
        if let Some(description) = dir_def.description {
            new_dir_def.description(description.into())
        }
        if let Some(args_def) = dir_def.arguments_definition {
            args_def
                .input_value_definitions
                .into_iter()
                .for_each(|input_val_def| new_dir_def.arg(input_val_def.into()));
        }
        if dir_def.repeatable {
            new_dir_def.repeatable();
        }
        dir_def
            .directive_locations
            .into_iter()
            .for_each(|dir_loc| new_dir_def.location(dir_loc.into()));

        new_dir_def
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::DirectiveDefinition> for DirectiveDef {
    type Error = crate::FromError;

    fn try_from(
        directive_def: apollo_parser::ast::DirectiveDefinition,
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

impl From<Directive> for apollo_encoder::Directive {
    fn from(directive: Directive) -> Self {
        let mut new_directive = Self::new(directive.name.into());
        directive
            .arguments
            .into_iter()
            .for_each(|arg| new_directive.arg(arg.into()));

        new_directive
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Directive> for Directive {
    type Error = crate::FromError;

    fn try_from(directive: apollo_parser::ast::Directive) -> Result<Self, Self::Error> {
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

#[cfg(feature = "parser-impl")]
impl Directive {
    pub(crate) fn convert_directives(
        directives: apollo_parser::ast::Directives,
    ) -> Result<HashMap<Name, Directive>, crate::FromError> {
        directives
            .directives()
            .map(|d| Ok((d.name().unwrap().into(), Directive::try_from(d)?)))
            .collect()
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary vector of `Directive`
    pub fn directives(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> Result<HashMap<Name, Directive>> {
        if self.directive_defs.is_empty() {
            return Ok(HashMap::new());
        }

        let num_directives = self.u.int_in_range(0..=(self.directive_defs.len() - 1))?;
        let directives = (0..num_directives)
            .map(|_| self.directive(directive_location))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flat_map(|d| d.map(|d| (d.name.clone(), d)))
            .collect();

        Ok(directives)
    }

    /// Create an arbitrary `Directive` given a directive location
    pub fn directive(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> Result<Option<Directive>> {
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
    pub fn directive_def(&mut self) -> Result<DirectiveDef> {
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

    /// Create an arbitrary `HashSet` of `DirectiveLocation`
    pub fn directive_locations(&mut self) -> Result<HashSet<DirectiveLocation>> {
        (1..self.u.int_in_range(2..=5usize)?)
            .map(|_| self.u.arbitrary())
            .collect::<Result<HashSet<_>>>()
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
