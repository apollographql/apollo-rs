use std::collections::{HashMap, HashSet};

use apollo_encoder::InterfaceDefinition;
use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    field::FieldDef,
    name::Name,
    DocumentBuilder,
};

/// InterfaceTypeDef is an abstract type where there are common fields declared.
///
/// Any type that implements an interface must define all the fields with names
/// and types exactly matching. The implementations of this interface are
/// explicitly listed out in possibleTypes.
///
/// *InterfaceTypeDefinition*:
///     Description? **interface** Name ImplementsInterfaces? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#InterfaceTypeDefinition).
#[derive(Debug, Clone)]
pub struct InterfaceTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) interfaces: HashSet<Name>,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<InterfaceTypeDef> for InterfaceDefinition {
    fn from(itf: InterfaceTypeDef) -> Self {
        let mut itf_def = InterfaceDefinition::new(itf.name.into());
        itf_def.description(itf.description.map(String::from));
        itf.fields_def
            .into_iter()
            .for_each(|f| itf_def.field(f.into()));
        itf.directives
            .into_iter()
            .for_each(|(_, directive)| itf_def.directive(directive.into()));
        itf.interfaces
            .into_iter()
            .for_each(|interface| itf_def.interface(interface.into()));
        if itf.extend {
            itf_def.extend();
        }

        itf_def
    }
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::InterfaceTypeDefinition> for InterfaceTypeDef {
    fn from(interface_def: apollo_parser::ast::InterfaceTypeDefinition) -> Self {
        Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: interface_def.description().map(Description::from),
            directives: interface_def
                .directives()
                .map(|d| {
                    d.directives()
                        .map(|d| (d.name().unwrap().into(), Directive::from(d)))
                        .collect()
                })
                .unwrap_or_default(),
            extend: false,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::from)
                .collect(),
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::InterfaceTypeExtension> for InterfaceTypeDef {
    fn from(interface_def: apollo_parser::ast::InterfaceTypeExtension) -> Self {
        Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: interface_def
                .directives()
                .map(|d| {
                    d.directives()
                        .map(|d| (d.name().unwrap().into(), Directive::from(d)))
                        .collect()
                })
                .unwrap_or_default(),
            extend: true,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::from)
                .collect(),
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `InterfaceTypeDef`
    pub fn interface_type_definition(&mut self) -> Result<InterfaceTypeDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.type_name()?;
        let fields_def = self.fields_definition(&[])?;
        let directives = self.directives(DirectiveLocation::Interface)?;
        let interfaces = self.implements_interfaces()?;

        Ok(InterfaceTypeDef {
            description,
            name,
            fields_def,
            directives,
            extend: self.u.arbitrary().unwrap_or(false),
            interfaces,
        })
    }

    /// Create an arbitrary `HashSet` of implemented interfaces
    pub fn implements_interfaces(&mut self) -> Result<HashSet<Name>> {
        if self.interface_type_defs.is_empty() {
            return Ok(HashSet::new());
        }

        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let mut interface_impls = HashSet::with_capacity(num_itf);

        for _ in 0..num_itf {
            interface_impls.insert(self.u.choose(&self.interface_type_defs)?.name.clone());
        }

        Ok(interface_impls)
    }
}
