use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use apollo_encoder::{EnumDefinition, EnumValue};
use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    DocumentBuilder,
};

/// Enums are special scalars that can only have a defined set of values.
///
/// *EnumTypeDefinition*:
///     Description? **enum** Name Directives? EnumValuesDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Enums).
#[derive(Debug, Clone)]
pub struct EnumTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) enum_values_def: HashSet<EnumValueDefinition>,
    pub(crate) extend: bool,
}

impl From<EnumTypeDef> for EnumDefinition {
    fn from(enum_: EnumTypeDef) -> Self {
        let mut new_enum = EnumDefinition::new(enum_.name.into());
        if let Some(description) = enum_.description {
            new_enum.description(description.into())
        }
        enum_
            .enum_values_def
            .into_iter()
            .for_each(|val| new_enum.value(val.into()));
        enum_
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_enum.directive(directive.into()));
        if enum_.extend {
            new_enum.extend();
        }

        new_enum
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::EnumTypeDefinition> for EnumTypeDef {
    type Error = crate::FromError;

    fn try_from(enum_def: apollo_parser::ast::EnumTypeDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            description: enum_def
                .description()
                .and_then(|d| d.string_value())
                .map(|s| Description::from(Into::<String>::into(s))),
            name: enum_def.name().unwrap().into(),
            directives: enum_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            enum_values_def: enum_def
                .enum_values_definition()
                .expect("must have enum values definition")
                .enum_value_definitions()
                .map(EnumValueDefinition::try_from)
                .collect::<Result<_, _>>()?,
            extend: false,
        })
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::EnumTypeExtension> for EnumTypeDef {
    type Error = crate::FromError;

    fn try_from(enum_def: apollo_parser::ast::EnumTypeExtension) -> Result<Self, Self::Error> {
        Ok(Self {
            description: None,
            name: enum_def.name().unwrap().into(),
            directives: enum_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            enum_values_def: enum_def
                .enum_values_definition()
                .expect("must have enum values definition")
                .enum_value_definitions()
                .map(EnumValueDefinition::try_from)
                .collect::<Result<_, _>>()?,
            extend: true,
        })
    }
}

/// The __EnumValue type represents one of possible values of an enum.
///
/// *EnumValueDefinition*:
///     Description? EnumValue Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__EnumValue-Type).
#[derive(Debug, Clone)]
pub struct EnumValueDefinition {
    pub(crate) description: Option<Description>,
    pub(crate) value: Name,
    pub(crate) directives: HashMap<Name, Directive>,
}

impl From<EnumValueDefinition> for EnumValue {
    fn from(enum_val: EnumValueDefinition) -> Self {
        let mut new_enum_val = Self::new(enum_val.value.into());
        if let Some(description) = enum_val.description {
            new_enum_val.description(description.into())
        }
        enum_val
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_enum_val.directive(directive.into()));

        new_enum_val
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::EnumValueDefinition> for EnumValueDefinition {
    type Error = crate::FromError;

    fn try_from(
        enum_value_def: apollo_parser::ast::EnumValueDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            description: enum_value_def.description().map(Description::from),
            value: enum_value_def
                .enum_value()
                .expect("enum value def must have enum value")
                .name()
                .expect("enum value mus have a name")
                .into(),
            directives: enum_value_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl PartialEq for EnumValueDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for EnumValueDefinition {}

impl Hash for EnumValueDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `EnumTypeDef`
    pub fn enum_type_definition(&mut self) -> Result<EnumTypeDef> {
        let extend = !self.enum_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = if extend {
            let available_enums: Vec<&Name> = self
                .enum_type_defs
                .iter()
                .filter_map(|enm| if enm.extend { None } else { Some(&enm.name) })
                .collect();
            (*self.u.choose(&available_enums)?).clone()
        } else {
            self.type_name()?
        };
        let enum_values_def = self.enum_values_definition()?;
        let directives = self.directives(DirectiveLocation::Enum)?;

        Ok(EnumTypeDef {
            description,
            name,
            enum_values_def,
            directives,
            extend,
        })
    }

    /// Choose an arbitrary `EnumTypeDef` in existings (already created) enum definitions
    pub fn choose_enum(&mut self) -> Result<&EnumTypeDef> {
        self.u.choose(&self.enum_type_defs)
    }

    /// Create an arbitrary variant `Name` given an enum
    pub fn arbitrary_variant<'b>(&mut self, enum_: &'b EnumTypeDef) -> Result<&'b Name> {
        let arbitrary_idx = self.u.int_in_range(0..=(enum_.enum_values_def.len() - 1))?;
        Ok(enum_
            .enum_values_def
            .iter()
            .nth(arbitrary_idx)
            .map(|e| &e.value)
            .expect("cannot get variant"))
    }

    /// Create an arbitrary `EnumValueDefinition`
    pub fn enum_values_definition(&mut self) -> Result<HashSet<EnumValueDefinition>> {
        let mut enum_values_def = HashSet::with_capacity(self.u.int_in_range(2..=10usize)?);
        for i in 0..self.u.int_in_range(2..=10usize)? {
            let description = self
                .u
                .arbitrary()
                .unwrap_or(false)
                .then(|| self.description())
                .transpose()?;
            let value = self.name_with_index(i)?;
            let directives = self.directives(DirectiveLocation::EnumValue)?;

            enum_values_def.insert(EnumValueDefinition {
                description,
                value,
                directives,
            });
        }

        Ok(enum_values_def)
    }
}
