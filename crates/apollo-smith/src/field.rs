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

    /// Whether selecting `def` under its own name could merge with what's
    /// already selected. Arguments are compared only when they're cached,
    /// since otherwise they're generated after the field is chosen.
    fn could_merge(&self, parent: &Name, def: &FieldDef) -> bool {
        let coord = TypeAttributeCoordinate {
            ty: parent.clone().into(),
            attribute: def.name.clone().into(),
        };
        let Some(args) = self.chosen_arguments.get(&coord) else {
            return self.can_merge_ignoring_args(&def.name, &def.ty);
        };
        self.can_merge(
            &def.name,
            &SelectedField {
                parent: parent.clone(),
                ty: def.ty.clone(),
                args: args.clone(),
            },
        )
    }

    /// Create an arbitrary `Field` on the composite type at the top of the stack
    pub fn field(&mut self, _index: usize) -> ArbitraryResult<Field> {
        let parent = self
            .stack
            .last()
            .expect("a composite type must be added on the stack");
        let parent_name = parent.name().clone();
        // Picking only fields that can merge with what's already selected
        // keeps conflicts from turning most selections into `__typename`.
        let candidates: Vec<FieldDef> = parent
            .fields_def()
            .iter()
            .filter(|def| self.could_merge(&parent_name, def))
            .cloned()
            .collect();
        if candidates.is_empty() {
            return Ok(self.typename_field());
        }

        let chosen_field_def = self.u.choose(&candidates)?.clone();

        let name = chosen_field_def.name.clone();
        let coord = TypeAttributeCoordinate {
            ty: parent_name.clone().into(),
            attribute: name.clone().into(),
        };
        // Reusing arguments per field makes same-named selections mergeable
        let cached_args = self
            .operation_variables
            .as_ref()
            .and_then(|vars| vars.arguments.get(&coord))
            .or_else(|| self.chosen_arguments.get(&coord))
            .cloned();
        let variable_count = self
            .operation_variables
            .as_ref()
            .map(|v| v.definitions.len());
        let args = match &cached_args {
            Some(args) => args.clone(),
            None => chosen_field_def
                .arguments_definition
                .as_ref()
                .map(|args_def| self.field_arguments(args_def))
                .unwrap_or_else(|| Ok(vec![]))?,
        };
        let selected = SelectedField {
            parent: parent_name,
            ty: chosen_field_def.ty.clone(),
            args: args.clone(),
        };
        if !self.can_merge(&name, &selected) {
            // Variables first used by these arguments would be left unused
            if let (Some(vars), Some(count)) = (&mut self.operation_variables, variable_count) {
                vars.definitions.truncate(count);
            }
            return Ok(self.typename_field());
        }
        if cached_args.is_none() {
            self.cache_arguments(coord, args.clone());
        }
        self.record_selection(name.clone(), selected);
        let directives = self.directives(DirectiveLocation::Field)?;

        let selection_set = if !chosen_field_def.ty.is_builtin() {
            // Put current ty on the stack
            if self.stack_ty(&chosen_field_def.ty) {
                let res = self.with_child_scope(&name, |b| b.selection_set());
                self.stack.pop();
                Some(res?)
            } else {
                None
            }
        } else {
            None
        };

        // TODO: Reintroduce alias generation logic which respects aliases on other fields
        // or fragments. For now, we will not generate aliases to avoid conflicts.
        // See <https://spec.graphql.org/September2025/#sec-Field-Selection-Merging> for merge
        // requirements.

        Ok(Field {
            alias: None,
            name,
            args,
            directives,
            selection_set,
        })
    }
}
