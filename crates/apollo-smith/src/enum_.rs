use std::{collections::HashSet, hash::Hash};

use apollo_encoder::{EnumDef, EnumValue};
use arbitrary::Result;

use crate::{description::Description, directive::Directive, name::Name, DocumentBuilder};

#[derive(Debug, Clone)]
pub struct EnumTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) directives: Vec<Directive>,
    pub(crate) enum_values_def: HashSet<EnumValueDefinition>,
    pub(crate) extend: bool,
}

impl From<EnumTypeDef> for EnumDef {
    fn from(enum_: EnumTypeDef) -> Self {
        let mut new_enum = EnumDef::new(enum_.name.into());
        new_enum.description(enum_.description.map(String::from));
        enum_
            .enum_values_def
            .into_iter()
            .for_each(|val| new_enum.value(val.into()));
        enum_
            .directives
            .into_iter()
            .for_each(|directive| new_enum.directive(directive.into()));
        if enum_.extend {
            new_enum.extend();
        }

        new_enum
    }
}

#[derive(Debug, Clone)]
pub struct EnumValueDefinition {
    pub(crate) description: Option<Description>,
    pub(crate) value: Name,
    pub(crate) directives: Vec<Directive>,
}

impl From<EnumValueDefinition> for EnumValue {
    fn from(enum_val: EnumValueDefinition) -> Self {
        let mut new_enum_val = Self::new(enum_val.value.into());
        new_enum_val.description(enum_val.description.map(String::from));
        enum_val
            .directives
            .into_iter()
            .for_each(|directive| new_enum_val.directive(directive.into()));

        new_enum_val
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
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.type_name()?;
        let enum_values_def = self.enum_values_definition()?;
        let directives = self.directives()?;

        Ok(EnumTypeDef {
            description,
            name,
            enum_values_def,
            directives,
            extend: self.u.arbitrary().unwrap_or(false),
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
            let directives = self.directives()?;

            enum_values_def.insert(EnumValueDefinition {
                description,
                value,
                directives,
            });
        }

        Ok(enum_values_def)
    }
}
