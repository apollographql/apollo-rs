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
        // Only the base def declares `implements`; extensions leave it
        // empty so we don't emit `implements X` twice for the same type.
        let interfaces: IndexSet<Name> = if extend {
            IndexSet::new()
        } else {
            self.implements_interfaces()?
                .into_iter()
                .filter(|n| n != &name)
                .collect()
        };
        let fields_def = self.fields_definition(&[])?;
        let directives = self.directives(DirectiveLocation::Interface)?;

        Ok(InterfaceTypeDef {
            description,
            name,
            fields_def,
            directives,
            extend,
            interfaces,
        })
    }

    /// Pick a random set of interfaces to implement, expanded to its
    /// transitive closure. If we roll `X`, and `X implements Y`, the
    /// caller's `implements` clause is `X & Y` so the validator's
    /// "must also implement" rule is satisfied without a backfill step.
    pub fn implements_interfaces(&mut self) -> ArbitraryResult<IndexSet<Name>> {
        if self.interface_type_defs.is_empty() {
            return Ok(IndexSet::new());
        }
        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let mut picks: IndexSet<Name> = IndexSet::with_capacity(num_itf);
        for _ in 0..num_itf {
            picks.insert(self.u.choose(&self.interface_type_defs)?.name.clone());
        }
        let mut frontier: Vec<Name> = picks.iter().cloned().collect();
        while let Some(next) = frontier.pop() {
            for itf in &self.interface_type_defs {
                if itf.name == next {
                    for parent in &itf.interfaces {
                        if picks.insert(parent.clone()) {
                            frontier.push(parent.clone());
                        }
                    }
                }
            }
        }
        Ok(picks)
    }

    /// After every interface (base + extension) has been generated,
    /// append any fields each implementer is missing from its declared
    /// parents. Single pass: existing fields are left alone, only
    /// missing ones get added.
    pub(crate) fn backfill_inherited_interface_fields(&mut self) {
        let parent_fields = effective_fields_by_name(&self.interface_type_defs);
        for idx in 0..self.interface_type_defs.len() {
            let parents = self.interface_type_defs[idx].interfaces.clone();
            let owned: std::collections::HashSet<String> = self.interface_type_defs[idx]
                .fields_def
                .iter()
                .map(|f| f.name.name.clone())
                .collect();
            for parent in &parents {
                let Some(fields) = parent_fields.get(parent) else {
                    continue;
                };
                for (fname, fdef) in fields {
                    if owned.contains(fname) {
                        continue;
                    }
                    self.interface_type_defs[idx].fields_def.push(fdef.clone());
                }
            }
        }
    }
}

/// Defs with a name and field list. Used by `effective_fields_by_name`
/// so both `InterfaceTypeDef` and `ObjectTypeDef` can share it.
pub(crate) trait NamedDef {
    fn name(&self) -> &Name;
    fn fields(&self) -> &[FieldDef];
}

impl NamedDef for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

impl NamedDef for crate::ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

/// Union of every def's fields by type name, merging base + extensions.
/// First occurrence of each field name wins.
pub(crate) fn effective_fields_by_name<T: NamedDef>(
    defs: &[T],
) -> std::collections::HashMap<Name, IndexMap<String, FieldDef>> {
    use std::collections::HashMap;
    let mut out: HashMap<Name, IndexMap<String, FieldDef>> = HashMap::new();
    for def in defs {
        let entry = out.entry(def.name().clone()).or_default();
        for f in def.fields() {
            entry
                .entry(f.name.name.clone())
                .or_insert_with(|| f.clone());
        }
    }
    out
}

impl StackedEntity for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}
