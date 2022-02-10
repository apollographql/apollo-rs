use std::collections::HashSet;

use arbitrary::Result;

use crate::{
    argument::{Argument, ArgumentsDef},
    description::Description,
    directive::Directive,
    name::Name,
    selection_set::SelectionSet,
    ty::Ty,
    DocumentBuilder,
};

/// The __FieldDef type represents each field definition in an Object definition or Interface type definition.
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FieldDefinition).
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) arguments_definition: Option<ArgumentsDef>,
    pub(crate) ty: Ty,
    pub(crate) directives: Vec<Directive>,
}

impl From<FieldDef> for apollo_encoder::FieldDef {
    fn from(val: FieldDef) -> Self {
        let mut field = Self::new(val.name.into(), val.ty.into());
        if let Some(arg) = val.arguments_definition {
            arg.input_value_definitions
                .into_iter()
                .for_each(|input_val| field.arg(input_val.into()));
        }
        field.description(val.description.map(String::from));
        val.directives
            .into_iter()
            .for_each(|directive| field.directive(directive.into()));

        field
    }
}

/// The __Field type represents each field in an Object or Interface type.
///
/// *Field*:
///     Alias? Name Arguments? Directives? SelectionSet?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Fields).
#[derive(Debug)]
pub struct Field {
    pub(crate) alias: Option<Name>,
    pub(crate) name: Name,
    pub(crate) args: Vec<Argument>,
    pub(crate) directives: Vec<Directive>,
    pub(crate) selection_set: Option<SelectionSet>,
}

impl From<Field> for apollo_encoder::Field {
    fn from(field: Field) -> Self {
        let mut new_field = Self::new(field.name.into());
        new_field.alias(field.alias.map(String::from));
        field
            .args
            .into_iter()
            .for_each(|arg| new_field.argument(arg.into()));
        field
            .directives
            .into_iter()
            .for_each(|directive| new_field.directive(directive.into()));
        new_field.selection_set(field.selection_set.map(Into::into));

        new_field
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary list of `FieldDef`
    pub fn fields_definition(&mut self, exclude: &[&Name]) -> Result<Vec<FieldDef>> {
        let num_fields = self.u.int_in_range(2..=50usize)?;
        let mut fields_names = HashSet::with_capacity(num_fields);

        for i in 0..num_fields {
            let name = self.name_with_index(i)?;
            if !exclude.contains(&&name) {
                fields_names.insert(name);
            }
        }

        // TODO add mechanism to add own type for recursive type
        let available_types: Vec<Ty> = self.list_existing_types();

        fields_names
            .into_iter()
            .map(|field_name| {
                Ok(FieldDef {
                    description: self
                        .u
                        .arbitrary()
                        .unwrap_or(false)
                        .then(|| self.description())
                        .transpose()?,
                    name: field_name,
                    arguments_definition: self
                        .u
                        .arbitrary()
                        .unwrap_or(false)
                        .then(|| self.arguments_definition())
                        .transpose()?,
                    ty: self.choose_ty(&available_types)?,
                    directives: self.directives()?,
                })
            })
            .collect()
    }

    /// Create an arbitrary `Field` given an index to put in the name of the field (to avoid duplicated fields)
    pub fn field_with_index(&mut self, index: usize) -> Result<Field> {
        let alias = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.name_with_index(index))
            .transpose()?;
        let name = self.name_with_index(index)?;
        let args = self.arguments()?;
        let directives = self.directives()?;
        let selection_set = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.selection_set())
            .transpose()?;

        Ok(Field {
            alias,
            name,
            args,
            directives,
            selection_set,
        })
    }
}
