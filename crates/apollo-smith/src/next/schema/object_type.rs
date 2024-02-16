use apollo_compiler::ast::OperationType;
use apollo_compiler::schema::ObjectType;

pub(crate) trait ObjectTypeExt {
    fn operation_type(&self) -> Option<OperationType>;
}

impl ObjectTypeExt for ObjectType {
    fn operation_type(&self) -> Option<OperationType> {
        match self.name.as_str() {
            "Query" => Some(OperationType::Query),
            "Mutation" => Some(OperationType::Mutation),
            "Subscription" => Some(OperationType::Subscription),
            _ => None,
        }
    }
}
