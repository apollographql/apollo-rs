use std::collections::{HashMap, HashSet};

use arbitrary::Result;

use crate::{
    argument::{Argument, ArgumentsDef},
    description::Description,
    directive::{Directive, DirectiveLocation},
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
    pub(crate) directives: HashMap<Name, Directive>,
}

impl From<FieldDef> for apollo_encoder::FieldDefinition {
    fn from(val: FieldDef) -> Self {
        let mut field = Self::new(val.name.into(), val.ty.into());
        if let Some(arg) = val.arguments_definition {
            arg.input_value_definitions
                .into_iter()
                .for_each(|input_val| field.arg(input_val.into()));
        }
        if let Some(description) = val.description {
            field.description(description.into());
        }
        val.directives
            .into_iter()
            .for_each(|(_dir_name, directive)| field.directive(directive.into()));

        field
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::FieldDefinition> for FieldDef {
    type Error = crate::FromError;

    fn try_from(field_def: apollo_parser::ast::FieldDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            description: field_def.description().map(Description::from),
            name: field_def
                .name()
                .expect("field definition must have a name")
                .into(),
            arguments_definition: field_def
                .arguments_definition()
                .map(ArgumentsDef::try_from)
                .transpose()?,
            ty: field_def.ty().unwrap().into(),
            directives: field_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
        })
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
    pub(crate) directives: HashMap<Name, Directive>,
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
            .for_each(|(_, directive)| new_field.directive(directive.into()));
        new_field.selection_set(field.selection_set.map(Into::into));

        new_field
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Field> for Field {
    type Error = crate::FromError;

    fn try_from(field: apollo_parser::ast::Field) -> Result<Self, Self::Error> {
        Ok(Self {
            alias: field.alias().map(|alias| alias.name().unwrap().into()),
            name: field.name().unwrap().into(),
            args: field
                .arguments()
                .map(|arguments| {
                    arguments
                        .arguments()
                        .map(Argument::try_from)
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
            directives: field
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            selection_set: field
                .selection_set()
                .map(SelectionSet::try_from)
                .transpose()?,
        })
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
                    directives: self.directives(DirectiveLocation::FieldDefinition)?,
                })
            })
            .collect()
    }

    /// Create an arbitrary `Field` given an object type
    pub fn field(&mut self, index: usize) -> Result<Field> {
        let fields_defs = self
            .stack
            .last()
            .expect("an object type must be added on the stack")
            .fields_def();

        let chosen_field_def = self.u.choose(fields_defs)?.clone();
        let mut alias = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.name_with_index(index))
            .transpose()?;

        let name = chosen_field_def.name.clone();
        // To not have same selection with different arguments
        let args = match self.chosen_arguments.get(&name) {
            Some(args) => args.clone(),
            None => {
                let args = chosen_field_def
                    .arguments_definition
                    .clone()
                    .map(|args_def| self.arguments_with_def(&args_def))
                    .unwrap_or_else(|| Ok(vec![]))?;
                self.chosen_arguments.insert(name.clone(), args.clone());

                args
            }
        };
        let directives = self.directives(DirectiveLocation::Field)?;

        let selection_set = if !chosen_field_def.ty.is_builtin() {
            // Put current ty on the stack
            if self.stack_ty(&chosen_field_def.ty) {
                let res = Some(self.selection_set()?);
                self.stack.pop();
                res
            } else {
                None
            }
        } else {
            None
        };

        // To not choose different alias name for the same field
        // Useful in this situation
        // {
        //  me {
        //      T1: name
        //  }
        //  me {
        //    T0: id
        //    T1: id
        //  }
        // }
        if let Some(alias_name) = alias.take() {
            match self.chosen_aliases.get(&alias_name) {
                None => {
                    self.chosen_aliases.insert(alias_name.clone(), name.clone());
                    alias = Some(alias_name);
                }
                Some(original_field_name) => {
                    // If the alias point to the same original field name then we can keep this alias, if not we don't use it
                    if original_field_name == &name {
                        alias = Some(alias_name);
                    }
                }
            }
        }

        Ok(Field {
            alias,
            name,
            args,
            directives,
            selection_set,
        })
    }
}
