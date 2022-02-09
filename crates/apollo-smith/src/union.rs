use std::collections::HashSet;

use apollo_encoder::UnionDef;
use arbitrary::Result;

use crate::{description::Description, directive::Directive, name::Name, ty::Ty, DocumentBuilder};

#[derive(Debug)]
pub struct UnionTypeDef {
    pub(crate) name: Name,
    pub(crate) description: Option<Description>,
    pub(crate) members: HashSet<Ty>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) extend: bool,
}

impl From<UnionTypeDef> for UnionDef {
    fn from(union_ty_def: UnionTypeDef) -> Self {
        let mut new_union_ty_def = Self::new(union_ty_def.name.into());
        new_union_ty_def.description(union_ty_def.description.map(String::from));
        union_ty_def
            .members
            .into_iter()
            .for_each(|member| new_union_ty_def.member(member.name().clone().into()));
        union_ty_def
            .directives
            .into_iter()
            .for_each(|directive| new_union_ty_def.directive(directive.into()));

        if union_ty_def.extend {
            new_union_ty_def.extend();
        }

        new_union_ty_def
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `UnionTypeDef`
    pub fn union_type_definition(&mut self) -> Result<UnionTypeDef> {
        let name = self.type_name()?;
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let directives = self.directives()?;
        let extend = self.u.arbitrary().unwrap_or(false);
        let mut existing_types = self.list_existing_object_types();
        existing_types.extend(
            self.union_type_defs
                .iter()
                .map(|u| Ty::Named(u.name.clone())),
        );

        let members = (0..self.u.int_in_range(2..=10)?)
            .map(|_| self.choose_named_ty(&existing_types))
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
