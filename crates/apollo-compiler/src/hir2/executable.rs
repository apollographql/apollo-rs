use super::type_system::TypeSystem;
use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::OperationType;
use apollo_parser::mir::Ranged;
use indexmap::map::Entry;
use indexmap::IndexMap;

/// Executable definitions, annotated with type information
#[derive(Clone, Debug)]
pub struct Executable {
    pub named_operations: IndexMap<Name, Operation>,
    pub anonymous_operation: Option<Operation>,
    pub fragments: IndexMap<Name, Fragment>,
}

#[derive(Clone, Debug)]
pub struct Operation {
    pub operation_type: OperationType,
    /// The name of the object type for this root operation
    pub ty: mir::NamedType,
    pub variables: Vec<Harc<Ranged<mir::VariableDefinition>>>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug)]
pub struct Fragment {
    pub type_condition: mir::NamedType,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}

#[derive(Clone, Debug)]
pub struct Field {
    /// The type of this field, resolved from context and type system information
    pub ty: mir::Type,
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<(Name, mir::Value)>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub selection_set: Vec<Selection>,
}

#[derive(Clone, Debug)]
pub struct FragmentSpread {
    pub fragment_name: Name,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
}

#[derive(Clone, Debug)]
pub struct InlineFragment {
    pub type_condition: Option<mir::NamedType>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub selection_set: Vec<Selection>,
}

/// Run validation for details
#[non_exhaustive]
pub struct TypeError {}

impl Executable {
    pub fn new(type_system: &TypeSystem, input_files: &[mir::Document]) -> Result<Self, TypeError> {
        let mut named_operations = IndexMap::new();
        let mut anonymous_operation = None;
        let mut fragments = IndexMap::new();
        for document in input_files {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::OperationDefinition(operation) => {
                        if let Some(name) = &operation.name {
                            if let Entry::Vacant(entry) = named_operations.entry(name.clone()) {
                                entry.insert(Operation::new(type_system, operation)?);
                            }
                        } else {
                            if anonymous_operation.is_none() {
                                anonymous_operation = Some(Operation::new(type_system, operation)?);
                            }
                        }
                    }
                    mir::Definition::FragmentDefinition(fragment) => {
                        if let Entry::Vacant(entry) = fragments.entry(fragment.name.clone()) {
                            entry.insert(Fragment::new(type_system, fragment)?);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(Executable {
            named_operations,
            anonymous_operation,
            fragments,
        })
    }
}

impl Operation {
    fn new(
        type_system: &TypeSystem,
        definition: &mir::OperationDefinition,
    ) -> Result<Self, TypeError> {
        let ty = type_system
            .schema
            .root_operation(definition.operation_type)
            .ok_or(TypeError {})?;
        Ok(Self {
            selection_set: Selection::new_set(
                type_system,
                Some(definition.operation_type),
                ty,
                &definition.selection_set,
            )?,
            ty: ty.clone(),
            operation_type: definition.operation_type,
            variables: definition.variables.clone(),
            directives: definition.directives.clone(),
        })
    }
}

impl Fragment {
    fn new(
        type_system: &TypeSystem,
        definition: &mir::FragmentDefinition,
    ) -> Result<Self, TypeError> {
        let ty = &definition.type_condition;
        Ok(Self {
            selection_set: Selection::new_set(type_system, None, ty, &definition.selection_set)?,
            type_condition: ty.clone(),
            directives: definition.directives.clone(),
        })
    }
}

impl Selection {
    fn new_set(
        type_system: &TypeSystem,
        parent_is_root_operation: Option<OperationType>,
        parent_type: &mir::NamedType,
        selection_set: &[mir::Selection],
    ) -> Result<Vec<Self>, TypeError> {
        if selection_set.is_empty() {
            return Ok(Vec::new());
        }
        let parent_fields_def = type_system.field_definitions(parent_type);
        selection_set
            .iter()
            .map(|selection| {
                Self::new(
                    type_system,
                    parent_is_root_operation,
                    parent_type,
                    parent_fields_def,
                    selection,
                )
            })
            .collect()
    }

    fn new(
        type_system: &TypeSystem,
        parent_is_root_operation: Option<OperationType>,
        parent_type: &Name,
        parent_fields_def: Option<&IndexMap<Name, Harc<Ranged<mir::FieldDefinition>>>>,
        selection: &mir::Selection,
    ) -> Result<Selection, TypeError> {
        Ok(match selection {
            mir::Selection::Field(field) => {
                let ty = &TypeSystem::meta_field_definitions(parent_is_root_operation)
                    .iter()
                    .find(|field_def| field_def.name == field.name)
                    .or_else(|| parent_fields_def?.get(&field.name))
                    .ok_or(TypeError {})?
                    .ty;
                Selection::Field(Field {
                    selection_set: Selection::new_set(
                        type_system,
                        None,
                        ty.inner_named_type(),
                        &field.selection_set,
                    )?,
                    alias: field.alias.clone(),
                    name: field.name.clone(),
                    arguments: field.arguments.clone(),
                    directives: field.directives.clone(),
                    ty: ty.clone(),
                })
            }
            mir::Selection::InlineFragment(inline_fragment) => {
                let parent_type = inline_fragment
                    .type_condition
                    .as_ref()
                    .unwrap_or(parent_type);
                Selection::InlineFragment(InlineFragment {
                    selection_set: Selection::new_set(
                        type_system,
                        None,
                        parent_type,
                        &inline_fragment.selection_set,
                    )?,
                    type_condition: inline_fragment.type_condition.clone(),
                    directives: inline_fragment.directives.clone(),
                })
            }
            mir::Selection::FragmentSpread(fragment_spread) => {
                Selection::FragmentSpread(FragmentSpread {
                    fragment_name: fragment_spread.fragment_name.clone(),
                    directives: fragment_spread.directives.clone(),
                })
            }
        })
    }
}
