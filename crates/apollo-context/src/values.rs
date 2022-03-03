use std::sync::Arc;

use crate::SourceDatabase;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Error {
    pub message: String,
    pub data: String,
    pub index: usize,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Definition {
    OperationDefinition(Arc<OperationDefinition>),
    FragmentDefinition(Arc<FragmentDefinition>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FragmentDefinition {
    pub name: String,
    pub type_condition: String,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Arc<Vec<Selection>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub ty: OperationType,
    pub name: String, // TODO @lrlna: Option<String>
    pub variables: Option<Arc<Vec<VariableDefinition>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Arc<Vec<Selection>>,
}

impl OperationDefinition {
    pub fn find_one(db: &dyn SourceDatabase, name: String) -> Option<Arc<OperationDefinition>> {
        db.operations().iter().find_map(|op| {
            if op.name == name {
                Some(Arc::new(op.clone()))
            } else {
                None
            }
        })
    }

    pub fn variables(
        db: &dyn SourceDatabase,
        name: String,
    ) -> Option<Arc<Vec<VariableDefinition>>> {
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct VariableDefinition {
    pub name: String,
    // ty: Type_,
    // default_value: Option<Arc<Value>>,
    pub directives: Option<Arc<Vec<Directive>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Directive {
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
}
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Argument {
    pub name: String,
    // pub value: Arc<Value>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Selection {
    Field(Arc<Field>),
    // FragmentSpread(Arc<FragmentSpread>),
    // InlineFragment(Arc<InlineFragment>),
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub alias: Option<Arc<Alias>>,
    pub name: String,
    pub arguments: Option<Arc<Vec<Argument>>>,
    pub directives: Option<Arc<Vec<Directive>>>,
    pub selection_set: Option<Arc<Vec<Selection>>>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);
