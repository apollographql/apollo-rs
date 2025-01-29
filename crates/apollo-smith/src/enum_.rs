use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::hash::Hash;

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
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) enum_values_def: IndexSet<EnumValueDefinition>,
    pub(crate) extend: bool,
}

impl From<EnumTypeDef> for ast::Definition {
    fn from(x: EnumTypeDef) -> Self {
        if x.extend {
            ast::EnumTypeExtension {
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
                values: x
                    .enum_values_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        } else {
            ast::EnumTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                directives: Directive::to_ast(x.directives),
                values: x
                    .enum_values_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::EnumTypeDefinition> for EnumTypeDef {
    type Error = crate::FromError;

    fn try_from(
        enum_def: apollo_parser::cst::EnumTypeDefinition,
    ) -> std::result::Result<Self, Self::Error> {
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
                .collect::<std::result::Result<_, _>>()?,
            extend: false,
        })
    }
}

impl TryFrom<apollo_parser::cst::EnumTypeExtension> for EnumTypeDef {
    type Error = crate::FromError;

    fn try_from(
        enum_def: apollo_parser::cst::EnumTypeExtension,
    ) -> std::result::Result<Self, Self::Error> {
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
                .collect::<std::result::Result<_, _>>()?,
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
    pub(crate) directives: IndexMap<Name, Directive>,
}

impl From<EnumValueDefinition> for ast::EnumValueDefinition {
    fn from(x: EnumValueDefinition) -> Self {
        Self {
            description: x.description.map(Into::into),
            value: x.value.into(),
            directives: Directive::to_ast(x.directives),
        }
    }
}

impl TryFrom<apollo_parser::cst::EnumValueDefinition> for EnumValueDefinition {
    type Error = crate::FromError;

    fn try_from(
        enum_value_def: apollo_parser::cst::EnumValueDefinition,
    ) -> std::result::Result<Self, Self::Error> {
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

impl DocumentBuilder<'_> {
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
    pub fn enum_values_definition(&mut self) -> Result<IndexSet<EnumValueDefinition>> {
        let mut enum_values_def = IndexSet::with_capacity(self.u.int_in_range(2..=10usize)?);
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
