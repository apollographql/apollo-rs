use std::collections::{HashMap, HashSet};

use apollo_encoder::UnionDefinition;
use arbitrary::Result;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    ty::Ty,
    DocumentBuilder,
};

/// UnionDefs are an abstract type where no common fields are declared.
///
/// *UnionDefTypeDefinition*:
///     Description? **union** Name Directives? UnionDefMemberTypes?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-UnionDef).
#[derive(Debug)]
pub struct UnionTypeDef {
    pub(crate) name: Name,
    pub(crate) description: Option<Description>,
    pub(crate) members: HashSet<Name>,
    pub(crate) directives: HashMap<Name, Directive>,
    pub(crate) extend: bool,
}

impl From<UnionTypeDef> for UnionDefinition {
    fn from(union_ty_def: UnionTypeDef) -> Self {
        let mut new_union_ty_def = Self::new(union_ty_def.name.into());
        if let Some(description) = union_ty_def.description {
            new_union_ty_def.description(description.into())
        }
        union_ty_def
            .members
            .into_iter()
            .for_each(|member| new_union_ty_def.member(member.into()));
        union_ty_def
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_union_ty_def.directive(directive.into()));

        if union_ty_def.extend {
            new_union_ty_def.extend();
        }

        new_union_ty_def
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::UnionTypeDefinition> for UnionTypeDef {
    type Error = crate::FromError;

    fn try_from(union_def: apollo_parser::ast::UnionTypeDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            name: union_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: union_def.description().map(Description::from),
            directives: union_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: false,
            members: union_def
                .union_member_types()
                .map(|members| {
                    members
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::UnionTypeExtension> for UnionTypeDef {
    type Error = crate::FromError;

    fn try_from(union_def: apollo_parser::ast::UnionTypeExtension) -> Result<Self, Self::Error> {
        Ok(Self {
            name: union_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: union_def
                .directives()
                .map(|d| {
                    d.directives()
                        .map(|d| Ok((d.name().unwrap().into(), Directive::try_from(d)?)))
                        .collect::<Result<_, crate::FromError>>()
                })
                .transpose()?
                .unwrap_or_default(),
            extend: true,
            members: union_def
                .union_member_types()
                .map(|members| {
                    members
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `UnionTypeDef`
    pub fn union_type_definition(&mut self) -> Result<UnionTypeDef> {
        let extend = !self.union_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let name = if extend {
            let available_unions: Vec<&Name> = self
                .union_type_defs
                .iter()
                .filter_map(|union| {
                    if union.extend {
                        None
                    } else {
                        Some(&union.name)
                    }
                })
                .collect();
            (*self.u.choose(&available_unions)?).clone()
        } else {
            self.type_name()?
        };
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let directives = self.directives(DirectiveLocation::Union)?;
        let extend = self.u.arbitrary().unwrap_or(false);
        let mut existing_types = self.list_existing_object_types();
        existing_types.extend(
            self.union_type_defs
                .iter()
                .map(|u| Ty::Named(u.name.clone())),
        );

        let members = (0..self.u.int_in_range(2..=10)?)
            .map(|_| Ok(self.choose_named_ty(&existing_types)?.name().clone()))
            .collect::<Result<HashSet<_>>>()?;

        Ok(UnionTypeDef {
            name,
            description,
            members,
            directives,
            extend,
        })
    }
}
