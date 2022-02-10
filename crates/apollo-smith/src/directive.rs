use std::collections::HashSet;

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

impl From<DirectiveDef> for apollo_encoder::DirectiveDef {
    fn from(dir_def: DirectiveDef) -> Self {
        let mut new_dir_def = Self::new(dir_def.name.into());
        new_dir_def.description(dir_def.description.map(String::from));
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary vector of `Directive`
    pub fn directives(&mut self) -> Result<Vec<Directive>> {
        // TODO choose only existing directives
        let num_directives = self.u.int_in_range(0..=4)?;
        let directives = (0..num_directives)
            .map(|_| self.directive())
            .collect::<Result<Vec<_>>>()?;

        Ok(directives)
    }

    /// Create an arbitrary `Directive`
    pub fn directive(&mut self) -> Result<Directive> {
        let name = self.name()?;
        let arguments = self.arguments()?;

        Ok(Directive { name, arguments })
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
#[derive(Debug, Clone, PartialEq, Hash, Eq, Arbitrary)]
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
