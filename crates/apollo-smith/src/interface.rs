use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::field::FieldDef;
use crate::name::Name;
use crate::DocumentBuilder;
use crate::StackedEntity;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

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
    pub(crate) interfaces: IndexSet<Name>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<InterfaceTypeDef> for ast::Definition {
    fn from(x: InterfaceTypeDef) -> Self {
        if x.extend {
            ast::InterfaceTypeExtension {
                name: x.name.into(),
                implements_interfaces: x.interfaces.into_iter().map(Into::into).collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        } else {
            ast::InterfaceTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                implements_interfaces: x.interfaces.into_iter().map(Into::into).collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::InterfaceTypeDefinition> for InterfaceTypeDef {
    type Error = crate::FromError;

    fn try_from(
        interface_def: apollo_parser::cst::InterfaceTypeDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: interface_def.description().map(Description::from),
            directives: interface_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: false,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl TryFrom<apollo_parser::cst::InterfaceTypeExtension> for InterfaceTypeDef {
    type Error = crate::FromError;

    fn try_from(
        interface_def: apollo_parser::cst::InterfaceTypeExtension,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: interface_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: true,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `InterfaceTypeDef`
    pub fn interface_type_definition(&mut self) -> ArbitraryResult<InterfaceTypeDef> {
        let extend = !self.interface_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = if extend {
            let available_itfs: Vec<&Name> = self
                .interface_type_defs
                .iter()
                .filter_map(|itf| if itf.extend { None } else { Some(&itf.name) })
                .collect();
            (*self.u.choose(&available_itfs)?).clone()
        } else {
            self.type_name()?
        };
        let fields_def = self.fields_definition(&[])?;
        let directives = self.directives(DirectiveLocation::Interface)?;
        let interfaces = self.implements_interfaces()?;

        Ok(InterfaceTypeDef {
            description,
            name,
            fields_def,
            directives,
            extend,
            interfaces,
        })
    }

    /// Create an arbitrary `IndexSet` of implemented interfaces
    pub fn implements_interfaces(&mut self) -> ArbitraryResult<IndexSet<Name>> {
        if self.interface_type_defs.is_empty() {
            return Ok(IndexSet::new());
        }

        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let mut interface_impls = IndexSet::with_capacity(num_itf);

        for _ in 0..num_itf {
            interface_impls.insert(self.u.choose(&self.interface_type_defs)?.name.clone());
        }

        Ok(interface_impls)
    }
}

impl StackedEntity for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}
