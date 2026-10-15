use crate::argument::Argument;
use crate::argument::ArgumentsDef;
use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::selection_set::SelectedField;
use crate::selection_set::SelectionSet;
use crate::ty::Ty;
use crate::DocumentBuilder;
use crate::StackedEntity;
use apollo_compiler::ast;
use apollo_compiler::coordinate::TypeAttributeCoordinate;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

/// The __FieldDef type represents each field definition in an Object definition or Interface type definition.
///
/// *FieldDefinition*:
///     Description? Name ArgumentsDefinition? **:** Type Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#FieldDefinition).
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) arguments_definition: Option<ArgumentsDef>,
    pub(crate) ty: Ty,
    pub(crate) directives: IndexMap<Name, Directive>,
}

impl From<FieldDef> for ast::FieldDefinition {
    fn from(x: FieldDef) -> Self {
        Self {
            description: x.description.map(Into::into),
            name: x.name.into(),
            directives: Directive::to_ast(x.directives),
            arguments: x.arguments_definition.map(Into::into).unwrap_or_default(),
            ty: x.ty.into(),
        }
    }
}

impl TryFrom<apollo_parser::cst::FieldDefinition> for FieldDef {
    type Error = crate::FromError;

    fn try_from(field_def: apollo_parser::cst::FieldDefinition) -> Result<Self, Self::Error> {
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
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Language.Fields).
#[derive(Debug, Clone)]
pub struct Field {
    pub(crate) alias: Option<Name>,
    pub(crate) name: Name,
    pub(crate) args: Vec<Argument>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) selection_set: Option<SelectionSet>,
}

impl From<Field> for ast::Field {
    fn from(x: Field) -> Self {
        Self {
            alias: x.alias.map(Into::into),
            name: x.name.into(),
            directives: Directive::to_ast(x.directives),
            arguments: x.args.into_iter().map(|x| Node::new(x.into())).collect(),
            selection_set: x.selection_set.map(Into::into),
        }
    }
}

impl TryFrom<apollo_parser::cst::Field> for Field {
    type Error = crate::FromError;

    fn try_from(field: apollo_parser::cst::Field) -> Result<Self, Self::Error> {
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

impl DocumentBuilder<'_> {
    /// Create an arbitrary list of `FieldDef`
    pub fn fields_definition(
        &mut self,
        exclude: &IndexSet<Name>,
    ) -> ArbitraryResult<Vec<FieldDef>> {
        let num_fields = self.u.int_in_range(2..=50usize)?;
        let mut fields_names = IndexSet::with_capacity(num_fields);

        for i in 0..num_fields {
            let name = self.name_with_index(i)?;
            if !exclude.contains(&name) {
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

    /// The parent's field definitions that can be selected under their own
    /// name without conflicting with what's already selected here. This is a
    /// first pass before arguments are generated, so arguments are compared
    /// only when they're cached. `can_merge` makes the final check.
    fn mergeable_field_defs(&self, parent: &dyn StackedEntity) -> Vec<FieldDef> {
        parent
            .fields_def()
            .iter()
            .filter(|def| {
                let coord = TypeAttributeCoordinate {
                    ty: parent.name().clone().into(),
                    attribute: def.name.clone().into(),
                };
                match self.cached_arguments(&coord) {
                    Some(args) => self.can_merge(
                        &def.name,
                        &SelectedField {
                            parent: parent.name().clone(),
                            field_name: def.name.clone(),
                            ty: def.ty.clone(),
                            args: args.clone(),
                        },
                    ),
                    None => {
                        self.can_merge_ignoring_args(&def.name, parent.name(), &def.name, &def.ty)
                    }
                }
            })
            .cloned()
            .collect()
    }

    /// Sometimes alias a field, often to a response name already selected
    /// alongside it so that field merging gets exercised.
    fn alias(&mut self, index: usize) -> ArbitraryResult<Option<Name>> {
        if !self.u.ratio(1, 4)? {
            return Ok(None);
        }
        let existing = self.current_response_names();
        if !existing.is_empty() && self.u.arbitrary()? {
            return Ok(Some(self.u.choose(&existing)?.clone()));
        }
        Ok(Some(self.name_with_index(index)?))
    }

    /// Arguments for a selection of `def`, and whether any were already
    /// cached for it. Reusing cached arguments keeps same-named selections
    /// mergeable, but a field under a new alias has nothing to merge with,
    /// so it gets new arguments.
    fn arguments_for(
        &mut self,
        coord: &TypeAttributeCoordinate,
        def: &FieldDef,
        fresh_alias: bool,
    ) -> ArbitraryResult<(Vec<Argument>, bool)> {
        let cached_args = self.cached_arguments(coord).cloned();
        let cached = cached_args.is_some();
        let args = match cached_args {
            Some(args) if !fresh_alias => args,
            _ => def
                .arguments_definition
                .as_ref()
                .map(|args_def| self.field_arguments(args_def))
                .unwrap_or_else(|| Ok(vec![]))?,
        };
        Ok((args, cached))
    }

    /// Create an arbitrary `Field` on the composite type at the top of the stack
    pub fn field(&mut self, index: usize) -> ArbitraryResult<Field> {
        let parent = self
            .stack
            .last()
            .expect("a composite type must be added on the stack");
        let parent_name = parent.name().clone();
        // Picking only fields that can merge with what's already selected
        // keeps conflicts from turning most selections into `__typename`.
        let candidates = self.mergeable_field_defs(parent.as_ref());
        if candidates.is_empty() {
            return Ok(self.typename_field());
        }

        let chosen_field_def = self.u.choose(&candidates)?.clone();

        let name = chosen_field_def.name.clone();
        let coord = TypeAttributeCoordinate {
            ty: parent_name.clone().into(),
            attribute: name.clone().into(),
        };
        let alias = self.alias(index)?;
        let fresh_alias = alias
            .as_ref()
            .is_some_and(|alias| !self.has_response_name(alias));
        let variables = self.variable_checkpoint();
        let (args, cached) = self.arguments_for(&coord, &chosen_field_def, fresh_alias)?;
        let selected = SelectedField {
            parent: parent_name,
            field_name: name.clone(),
            ty: chosen_field_def.ty.clone(),
            args: args.clone(),
        };
        // An alias that can't merge with what's already selected under it
        // falls back to the field's own name.
        let alias = alias.filter(|alias| self.can_merge(alias, &selected));
        let response_name = alias.clone().unwrap_or_else(|| name.clone());
        if !self.can_merge(&response_name, &selected) {
            // Variables first used by these arguments would be left unused
            self.rollback_variables(variables);
            return Ok(self.typename_field());
        }
        if !cached {
            self.cache_arguments(coord, args.clone());
        }
        self.record_selection(response_name.clone(), selected);
        let directives = self.directives(DirectiveLocation::Field)?;

        let selection_set = if !chosen_field_def.ty.is_builtin() {
            // Put current ty on the stack
            if self.stack_ty(&chosen_field_def.ty) {
                let res = self.in_merged_subselection(&response_name, |b| b.selection_set());
                self.stack.pop();
                Some(res?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Field {
            alias,
            name,
            args,
            directives,
            selection_set,
        })
    }
}
