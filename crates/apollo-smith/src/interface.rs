use std::collections::HashSet;

use apollo_encoder::InterfaceDef;
use arbitrary::Result;

use crate::{
    description::Description, directive::Directive, field::FieldDef, name::Name, DocumentBuilder,
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
    pub(crate) directives: Vec<Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<InterfaceTypeDef> for InterfaceDef {
    fn from(itf: InterfaceTypeDef) -> Self {
        let mut itf_def = InterfaceDef::new(itf.name.into());
        itf_def.description(itf.description.map(String::from));
        itf.fields_def
            .into_iter()
            .for_each(|f| itf_def.field(f.into()));
        itf.directives
            .into_iter()
            .for_each(|directive| itf_def.directive(directive.into()));
        itf.interfaces
            .into_iter()
            .for_each(|interface| itf_def.interface(interface.into()));
        if itf.extend {
            itf_def.extend();
        }

        itf_def
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
        let directives = self.directives()?;
        let interfaces = self.interface_implements()?;

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
    pub fn interface_implements(&mut self) -> Result<HashSet<Name>> {
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
