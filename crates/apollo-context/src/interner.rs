use crate::values;

#[salsa::query_group(InternerDatabase)]
pub trait Interner {
    #[salsa::interned]
    fn intern_document(&self, document: values::DocumentData) -> values::Document;

    #[salsa::interned]
    fn intern_definition(&self, definition: values::DefinitionData) -> values::Definition;

    #[salsa::interned]
    fn intern_operation_definition(
        &self,
        operation: values::OperationDefinitionData,
    ) -> values::OperationDefinition;

    #[salsa::interned]
    fn intern_variable_definition(
        &self,
        variable: values::VariableDefinitionData,
    ) -> values::VariableDefinition;

    #[salsa::interned]
    fn intern_selection(&self, selection: values::SelectionData) -> values::Selection;

    #[salsa::interned]
    fn intern_field(&self, selection: values::FieldData) -> values::Field;

    #[salsa::interned]
    fn intern_directive(&self, directive: values::DirectiveData) -> values::Directive;
}
