use super::type_system::TypeSystem;
use crate::FileId;
use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::OperationType;
use apollo_parser::mir::Ranged;
use indexmap::map::Entry;
use indexmap::IndexMap;
use std::fmt;

/// Executable definitions, annotated with type information
#[derive(Clone, Debug)]
pub struct ExecutableDocument {
    pub file_id: FileId,
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
    pub selection_set: SelectionSet,
}

#[derive(Clone, Debug)]
pub struct Fragment {
    pub type_condition: mir::NamedType,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub selection_set: SelectionSet,
}

#[derive(Clone, Debug)]
pub struct SelectionSet {
    parent_is_root_operation: Option<OperationType>,
    parent_type: mir::NamedType,
    selections: Vec<Selection>,
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
    pub selection_set: SelectionSet,
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
    pub selection_set: SelectionSet,
}

/// Run validation for details
#[non_exhaustive]
pub struct TypeError {}

impl ExecutableDocument {
    pub fn from_mir(
        type_system: &TypeSystem,
        document: mir::Document,
        file_id: FileId,
    ) -> Result<Self, TypeError> {
        let mut named_operations = IndexMap::new();
        let mut anonymous_operation = None;
        let mut fragments = IndexMap::new();
        for definition in &document.definitions {
            match definition {
                mir::Definition::OperationDefinition(operation) => {
                    if let Some(name) = &operation.name {
                        if let Entry::Vacant(entry) = named_operations.entry(name.clone()) {
                            entry.insert(Operation::from_mir(type_system, operation)?);
                        }
                    } else {
                        if anonymous_operation.is_none() {
                            anonymous_operation =
                                Some(Operation::from_mir(type_system, operation)?);
                        }
                    }
                }
                mir::Definition::FragmentDefinition(fragment) => {
                    if let Entry::Vacant(entry) = fragments.entry(fragment.name.clone()) {
                        entry.insert(Fragment::from_mir(type_system, fragment)?);
                    }
                }
                _ => {}
            }
        }
        Ok(ExecutableDocument {
            file_id,
            named_operations,
            anonymous_operation,
            fragments,
        })
    }

    pub fn to_mir(&self) -> mir::Document {
        let mut doc = mir::Document::new();
        if let Some(operation) = &self.anonymous_operation {
            doc.definitions.push(operation.to_mir(None))
        }
        for (name, operation) in &self.named_operations {
            doc.definitions.push(operation.to_mir(Some(name)))
        }
        for (name, fragment) in &self.fragments {
            doc.definitions.push(fragment.to_mir(name))
        }
        doc
    }
}

impl fmt::Display for ExecutableDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: this can be done without allocating temporary MIR nodes,
        // but ideally (implementation-wise) this would share private helpers
        // with MIR serialization.
        // These can’t be both private and shared when MIR and HIR are in separate crates.
        self.to_mir().fmt(f)
    }
}

impl Operation {
    fn from_mir(
        type_system: &TypeSystem,
        mir: &mir::OperationDefinition,
    ) -> Result<Self, TypeError> {
        let ty = type_system
            .schema
            .root_operation(mir.operation_type)
            .ok_or(TypeError {})?;
        let mut selection_set = SelectionSet::new_for_root(ty.clone(), mir.operation_type);
        selection_set.extend_from_mir(type_system, &mir.selection_set)?;
        Ok(Self {
            selection_set,
            ty: ty.clone(),
            operation_type: mir.operation_type,
            variables: mir.variables.clone(),
            directives: mir.directives.clone(),
        })
    }
}

impl Fragment {
    fn from_mir(
        type_system: &TypeSystem,
        mir: &mir::FragmentDefinition,
    ) -> Result<Self, TypeError> {
        let ty = &mir.type_condition;
        let mut selection_set = SelectionSet::new_for_non_root(ty.clone());
        selection_set.extend_from_mir(type_system, &mir.selection_set)?;
        Ok(Self {
            selection_set,
            type_condition: ty.clone(),
            directives: mir.directives.clone(),
        })
    }
}

impl SelectionSet {
    /// Create a new selection set for the root of an operation
    pub fn new_for_root(parent_type: mir::NamedType, operation_type: OperationType) -> Self {
        Self {
            parent_is_root_operation: Some(operation_type),
            parent_type,
            selections: Vec::new(),
        }
    }

    /// Create a new selection set for a field, fragment, or inline fragment
    pub fn new_for_non_root(parent_type: mir::NamedType) -> Self {
        Self {
            parent_is_root_operation: None,
            parent_type,
            selections: Vec::new(),
        }
    }

    pub fn selections(&self) -> &[Selection] {
        &self.selections
    }

    pub fn push(&mut self, selection: impl Into<Selection>) {
        self.selections.push(selection.into())
    }

    pub fn extend(&mut self, selections: impl IntoIterator<Item = impl Into<Selection>>) {
        self.selections
            .extend(selections.into_iter().map(|sel| sel.into()))
    }

    fn extend_from_mir(
        &mut self,
        type_system: &TypeSystem,
        mir_selections: &[mir::Selection],
    ) -> Result<(), TypeError> {
        for selection in mir_selections {
            match selection {
                mir::Selection::Field(mir) => {
                    let mut field = self
                        .new_field(type_system, mir.name.clone())?
                        .with_alias(mir.alias.clone())
                        .with_arguments(mir.arguments.iter().cloned())
                        .with_directives(mir.directives.iter().cloned());
                    field
                        .selection_set
                        .extend_from_mir(type_system, &mir.selection_set)?;
                    self.push(field)
                }
                mir::Selection::FragmentSpread(mir) => self.push(
                    FragmentSpread::new(mir.fragment_name.clone())
                        .with_directives(mir.directives.iter().cloned()),
                ),
                mir::Selection::InlineFragment(mir) => {
                    let mut inline_fragment =
                        InlineFragment::new(&self.parent_type, mir.type_condition.clone())
                            .with_directives(mir.directives.iter().cloned());
                    inline_fragment
                        .selection_set
                        .extend_from_mir(type_system, &mir.selection_set)?;
                    self.push(inline_fragment)
                }
            }
        }
        Ok(())
    }

    /// Create a new field to be added to this selection set with [`push`][Self::push]
    pub fn new_field(&self, type_system: &TypeSystem, name: Name) -> Result<Field, TypeError> {
        let ty = TypeSystem::meta_field_definitions(self.parent_is_root_operation)
            .iter()
            .find(|field_def| field_def.name == name)
            .or_else(|| {
                type_system
                    .types
                    .get(&self.parent_type)?
                    .fields()?
                    .get(&name)
            })
            .ok_or(TypeError {})?
            .ty
            .clone();
        let selection_set = SelectionSet::new_for_non_root(ty.inner_named_type().clone());
        Ok(Field {
            ty,
            alias: None,
            name,
            arguments: Vec::new(),
            directives: Vec::new(),
            selection_set,
        })
    }
}

impl From<Field> for Selection {
    fn from(value: Field) -> Self {
        Self::Field(value)
    }
}

impl From<InlineFragment> for Selection {
    fn from(value: InlineFragment) -> Self {
        Self::InlineFragment(value)
    }
}

impl From<FragmentSpread> for Selection {
    fn from(value: FragmentSpread) -> Self {
        Self::FragmentSpread(value)
    }
}

impl Field {
    pub fn with_alias(mut self, alias: impl Into<Option<Name>>) -> Self {
        self.alias = alias.into();
        self
    }

    pub fn with_directive(mut self, directive: impl Into<Harc<Ranged<mir::Directive>>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Harc<Ranged<mir::Directive>>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn with_argument(mut self, name: impl Into<Name>, value: impl Into<mir::Value>) -> Self {
        self.arguments.push((name.into(), value.into()));
        self
    }

    pub fn with_arguments(
        mut self,
        arguments: impl IntoIterator<Item = (impl Into<Name>, impl Into<mir::Value>)>,
    ) -> Self {
        self.arguments.extend(
            arguments
                .into_iter()
                .map(|(name, value)| (name.into(), value.into())),
        );
        self
    }

    pub fn with_selection(mut self, selection: impl Into<Selection>) -> Self {
        self.selection_set.push(selection);
        self
    }

    pub fn with_selections(
        mut self,
        selections: impl IntoIterator<Item = impl Into<Selection>>,
    ) -> Self {
        self.selection_set.extend(selections);
        self
    }
}

impl InlineFragment {
    pub fn new(parent_type: &mir::NamedType, type_condition: Option<mir::NamedType>) -> Self {
        if let Some(ty) = type_condition {
            Self::with_type_condition(ty)
        } else {
            Self::no_type_condition(parent_type.clone())
        }
    }

    pub fn no_type_condition(parent_type: mir::NamedType) -> Self {
        let selection_set = SelectionSet::new_for_non_root(parent_type);
        InlineFragment {
            type_condition: None,
            directives: Vec::new(),
            selection_set,
        }
    }

    pub fn with_type_condition(type_condition: mir::NamedType) -> Self {
        let selection_set = SelectionSet::new_for_non_root(type_condition.clone());
        InlineFragment {
            type_condition: Some(type_condition),
            directives: Vec::new(),
            selection_set,
        }
    }

    pub fn with_directive(mut self, directive: impl Into<Harc<Ranged<mir::Directive>>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Harc<Ranged<mir::Directive>>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }

    pub fn with_selection(mut self, selection: impl Into<Selection>) -> Self {
        self.selection_set.push(selection);
        self
    }

    pub fn with_selections(
        mut self,
        selections: impl IntoIterator<Item = impl Into<Selection>>,
    ) -> Self {
        self.selection_set.extend(selections);
        self
    }
}

impl FragmentSpread {
    pub fn new(fragment_name: Name) -> Self {
        Self {
            fragment_name,
            directives: Vec::new(),
        }
    }

    pub fn with_directive(mut self, directive: impl Into<Harc<Ranged<mir::Directive>>>) -> Self {
        self.directives.push(directive.into());
        self
    }

    pub fn with_directives(
        mut self,
        directives: impl IntoIterator<Item = Harc<Ranged<mir::Directive>>>,
    ) -> Self {
        self.directives.extend(directives);
        self
    }
}
