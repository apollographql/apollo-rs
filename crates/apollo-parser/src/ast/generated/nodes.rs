//! This is a generated file, please do not edit manually. Changes can be
//! made in codegeneration that lives in `xtask` top-level dir.

use crate::{
    ast::{support, AstChildren, AstNode},
    SyntaxKind::{self, *},
    SyntaxNode, SyntaxToken, S,
};
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) syntax: SyntaxNode,
}
impl Name {
    pub fn ident_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![ident])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Document {
    pub(crate) syntax: SyntaxNode,
}
impl Document {
    pub fn definitions(&self) -> AstChildren<Definition> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl OperationDefinition {
    pub fn operation_type(&self) -> Option<OperationType> {
        support::child(&self.syntax)
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn variable_definitions(&self) -> Option<VariableDefinitions> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentDefinition {
    pub fn fragment_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![fragment])
    }
    pub fn fragment_name(&self) -> Option<FragmentName> {
        support::child(&self.syntax)
    }
    pub fn type_condition(&self) -> Option<TypeCondition> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl DirectiveDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn directive_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![directive])
    }
    pub fn at_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![@])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn arguments_definition(&self) -> Option<ArgumentsDefinition> {
        support::child(&self.syntax)
    }
    pub fn repeatable_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![repeatable])
    }
    pub fn on_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![on])
    }
    pub fn directive_locations(&self) -> Option<DirectiveLocations> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl SchemaDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn schema_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![schema])
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn root_operation_type_definitions(&self) -> AstChildren<RootOperationTypeDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl ScalarTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn scalar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![scalar])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn type_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![type])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn implements_interfaces(&self) -> Option<ImplementsInterfaces> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fields_definition(&self) -> Option<FieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl InterfaceTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn interface_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![interface])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn implements_interfaces(&self) -> Option<ImplementsInterfaces> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fields_definition(&self) -> Option<FieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl UnionTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn union_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![union])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn union_member_types(&self) -> Option<UnionMemberTypes> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl EnumTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn enum_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![enum])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn enum_values_definition(&self) -> Option<EnumValuesDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputObjectTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl InputObjectTypeDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn input_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![input])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn input_fields_definition(&self) -> Option<InputFieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SchemaExtension {
    pub(crate) syntax: SyntaxNode,
}
impl SchemaExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn schema_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![schema])
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn root_operation_type_definitions(&self) -> AstChildren<RootOperationTypeDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScalarTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl ScalarTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn scalar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![scalar])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn type_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![type])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn implements_interfaces(&self) -> Option<ImplementsInterfaces> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fields_definition(&self) -> Option<FieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl InterfaceTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn interface_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![interface])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn implements_interfaces(&self) -> Option<ImplementsInterfaces> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn fields_definition(&self) -> Option<FieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl UnionTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn union_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![union])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn union_member_types(&self) -> Option<UnionMemberTypes> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl EnumTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn enum_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![enum])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn enum_values_definition(&self) -> Option<EnumValuesDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputObjectTypeExtension {
    pub(crate) syntax: SyntaxNode,
}
impl InputObjectTypeExtension {
    pub fn extend_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![extend])
    }
    pub fn input_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![input])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn input_fields_definition(&self) -> Option<InputFieldsDefinition> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OperationType {
    pub(crate) syntax: SyntaxNode,
}
impl OperationType {
    pub fn query_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![query])
    }
    pub fn mutation_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![mutation])
    }
    pub fn subscription_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![subscription])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDefinitions {
    pub(crate) syntax: SyntaxNode,
}
impl VariableDefinitions {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['('])
    }
    pub fn variable_definitions(&self) -> AstChildren<VariableDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![')'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Directives {
    pub(crate) syntax: SyntaxNode,
}
impl Directives {
    pub fn directives(&self) -> AstChildren<Directive> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SelectionSet {
    pub(crate) syntax: SyntaxNode,
}
impl SelectionSet {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn selections(&self) -> AstChildren<Selection> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Field {
    pub(crate) syntax: SyntaxNode,
}
impl Field {
    pub fn alias(&self) -> Option<Alias> {
        support::child(&self.syntax)
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn arguments(&self) -> Option<Arguments> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentSpread {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentSpread {
    pub fn dotdotdot_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![...])
    }
    pub fn fragment_name(&self) -> Option<FragmentName> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InlineFragment {
    pub(crate) syntax: SyntaxNode,
}
impl InlineFragment {
    pub fn dotdotdot_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![...])
    }
    pub fn type_condition(&self) -> Option<TypeCondition> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
    pub fn selection_set(&self) -> Option<SelectionSet> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Alias {
    pub(crate) syntax: SyntaxNode,
}
impl Alias {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Arguments {
    pub(crate) syntax: SyntaxNode,
}
impl Arguments {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['('])
    }
    pub fn arguments(&self) -> AstChildren<Argument> {
        support::children(&self.syntax)
    }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![')'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Argument {
    pub(crate) syntax: SyntaxNode,
}
impl Argument {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FragmentName {
    pub(crate) syntax: SyntaxNode,
}
impl FragmentName {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TypeCondition {
    pub(crate) syntax: SyntaxNode,
}
impl TypeCondition {
    pub fn on_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![on])
    }
    pub fn named_type(&self) -> Option<NamedType> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NamedType {
    pub(crate) syntax: SyntaxNode,
}
impl NamedType {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable {
    pub(crate) syntax: SyntaxNode,
}
impl Variable {
    pub fn dollar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![$])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StringValue {
    pub(crate) syntax: SyntaxNode,
}
impl StringValue {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FloatValue {
    pub(crate) syntax: SyntaxNode,
}
impl FloatValue {
    pub fn float_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![float])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IntValue {
    pub(crate) syntax: SyntaxNode,
}
impl IntValue {
    pub fn int_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![int])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BooleanValue {
    pub(crate) syntax: SyntaxNode,
}
impl BooleanValue {
    pub fn true_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![true])
    }
    pub fn false_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![false])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NullValue {
    pub(crate) syntax: SyntaxNode,
}
impl NullValue {
    pub fn null_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![null])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValue {
    pub(crate) syntax: SyntaxNode,
}
impl EnumValue {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListValue {
    pub(crate) syntax: SyntaxNode,
}
impl ListValue {
    pub fn l_brack_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['['])
    }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![']'])
    }
    pub fn values(&self) -> AstChildren<Value> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectValue {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectValue {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
    pub fn object_fields(&self) -> AstChildren<ObjectField> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ObjectField {
    pub(crate) syntax: SyntaxNode,
}
impl ObjectField {
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VariableDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl VariableDefinition {
    pub fn variable(&self) -> Option<Variable> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
    pub fn default_value(&self) -> Option<DefaultValue> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DefaultValue {
    pub(crate) syntax: SyntaxNode,
}
impl DefaultValue {
    pub fn eq_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![=])
    }
    pub fn value(&self) -> Option<Value> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ListType {
    pub(crate) syntax: SyntaxNode,
}
impl ListType {
    pub fn l_brack_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['['])
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
    pub fn r_brack_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![']'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NonNullType {
    pub(crate) syntax: SyntaxNode,
}
impl NonNullType {
    pub fn named_type(&self) -> Option<NamedType> {
        support::child(&self.syntax)
    }
    pub fn excl_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![!])
    }
    pub fn list_type(&self) -> Option<ListType> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Directive {
    pub(crate) syntax: SyntaxNode,
}
impl Directive {
    pub fn at_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![@])
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn arguments(&self) -> Option<Arguments> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Description {
    pub(crate) syntax: SyntaxNode,
}
impl Description {
    pub fn string_value(&self) -> Option<StringValue> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RootOperationTypeDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl RootOperationTypeDefinition {
    pub fn operation_type(&self) -> Option<OperationType> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn named_type(&self) -> Option<NamedType> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ImplementsInterfaces {
    pub(crate) syntax: SyntaxNode,
}
impl ImplementsInterfaces {
    pub fn implements_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![implements])
    }
    pub fn amp_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![&])
    }
    pub fn named_types(&self) -> AstChildren<NamedType> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldsDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl FieldsDefinition {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn field_definitions(&self) -> AstChildren<FieldDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl FieldDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn arguments_definition(&self) -> Option<ArgumentsDefinition> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ArgumentsDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl ArgumentsDefinition {
    pub fn l_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['('])
    }
    pub fn input_value_definitions(&self) -> AstChildren<InputValueDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![')'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputValueDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl InputValueDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn name(&self) -> Option<Name> {
        support::child(&self.syntax)
    }
    pub fn colon_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![:])
    }
    pub fn ty(&self) -> Option<Type> {
        support::child(&self.syntax)
    }
    pub fn default_value(&self) -> Option<DefaultValue> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UnionMemberTypes {
    pub(crate) syntax: SyntaxNode,
}
impl UnionMemberTypes {
    pub fn eq_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![=])
    }
    pub fn pipe_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![|])
    }
    pub fn named_types(&self) -> AstChildren<NamedType> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValuesDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl EnumValuesDefinition {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn enum_value_definitions(&self) -> AstChildren<EnumValueDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EnumValueDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl EnumValueDefinition {
    pub fn description(&self) -> Option<Description> {
        support::child(&self.syntax)
    }
    pub fn enum_value(&self) -> Option<EnumValue> {
        support::child(&self.syntax)
    }
    pub fn directives(&self) -> Option<Directives> {
        support::child(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputFieldsDefinition {
    pub(crate) syntax: SyntaxNode,
}
impl InputFieldsDefinition {
    pub fn l_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['{'])
    }
    pub fn input_value_definitions(&self) -> AstChildren<InputValueDefinition> {
        support::children(&self.syntax)
    }
    pub fn r_curly_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S!['}'])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveLocations {
    pub(crate) syntax: SyntaxNode,
}
impl DirectiveLocations {
    pub fn directive_locations(&self) -> AstChildren<DirectiveLocation> {
        support::children(&self.syntax)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectiveLocation {
    pub(crate) syntax: SyntaxNode,
}
impl DirectiveLocation {
    pub fn query_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![QUERY])
    }
    pub fn mutation_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![MUTATION])
    }
    pub fn subscription_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![SUBSCRIPTION])
    }
    pub fn field_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![FIELD])
    }
    pub fn fragment_definition_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![FRAGMENT_DEFINITION])
    }
    pub fn fragment_spread_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![FRAGMENT_SPREAD])
    }
    pub fn inline_fragment_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![INLINE_FRAGMENT])
    }
    pub fn variable_definition_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![VARIABLE_DEFINITION])
    }
    pub fn schema_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![SCHEMA])
    }
    pub fn scalar_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![SCALAR])
    }
    pub fn object_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![OBJECT])
    }
    pub fn field_definition_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![FIELD_DEFINITION])
    }
    pub fn argument_definition_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![ARGUMENT_DEFINITION])
    }
    pub fn interface_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![INTERFACE])
    }
    pub fn union_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![UNION])
    }
    pub fn enum_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![ENUM])
    }
    pub fn enum_value_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![ENUM_VALUE])
    }
    pub fn input_object_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![INPUT_OBJECT])
    }
    pub fn input_field_definition_token(&self) -> Option<SyntaxToken> {
        support::token(&self.syntax, S![INPUT_FIELD_DEFINITION])
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Definition {
    OperationDefinition(OperationDefinition),
    FragmentDefinition(FragmentDefinition),
    DirectiveDefinition(DirectiveDefinition),
    SchemaDefinition(SchemaDefinition),
    ScalarTypeDefinition(ScalarTypeDefinition),
    ObjectTypeDefinition(ObjectTypeDefinition),
    InterfaceTypeDefinition(InterfaceTypeDefinition),
    UnionTypeDefinition(UnionTypeDefinition),
    EnumTypeDefinition(EnumTypeDefinition),
    InputObjectTypeDefinition(InputObjectTypeDefinition),
    SchemaExtension(SchemaExtension),
    ScalarTypeExtension(ScalarTypeExtension),
    ObjectTypeExtension(ObjectTypeExtension),
    InterfaceTypeExtension(InterfaceTypeExtension),
    UnionTypeExtension(UnionTypeExtension),
    EnumTypeExtension(EnumTypeExtension),
    InputObjectTypeExtension(InputObjectTypeExtension),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Selection {
    Field(Field),
    FragmentSpread(FragmentSpread),
    InlineFragment(InlineFragment),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Value {
    Variable(Variable),
    StringValue(StringValue),
    FloatValue(FloatValue),
    IntValue(IntValue),
    BooleanValue(BooleanValue),
    NullValue(NullValue),
    EnumValue(EnumValue),
    ListValue(ListValue),
    ObjectValue(ObjectValue),
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    NamedType(NamedType),
    ListType(ListType),
    NonNullType(NonNullType),
}
impl AstNode for Name {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NAME
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Document {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DOCUMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for OperationDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OPERATION_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FragmentDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for DirectiveDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for SchemaDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SCHEMA_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ScalarTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SCALAR_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ObjectTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InterfaceTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INTERFACE_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for UnionTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == UNION_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for EnumTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InputObjectTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INPUT_OBJECT_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for SchemaExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SCHEMA_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ScalarTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SCALAR_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ObjectTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InterfaceTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INTERFACE_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for UnionTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == UNION_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for EnumTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InputObjectTypeExtension {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INPUT_OBJECT_TYPE_EXTENSION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for OperationType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OPERATION_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for VariableDefinitions {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE_DEFINITIONS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Directives {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for SelectionSet {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == SELECTION_SET
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Field {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FIELD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FragmentSpread {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_SPREAD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InlineFragment {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INLINE_FRAGMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Alias {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ALIAS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Arguments {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARGUMENTS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Argument {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARGUMENT
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FragmentName {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FRAGMENT_NAME
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for TypeCondition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == TYPE_CONDITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for NamedType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NAMED_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Variable {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for StringValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == STRING_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FloatValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FLOAT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for IntValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for BooleanValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == BOOLEAN_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for NullValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NULL_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for EnumValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ListValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == LIST_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ObjectValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ObjectField {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == OBJECT_FIELD
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for VariableDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == VARIABLE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for DefaultValue {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DEFAULT_VALUE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ListType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == LIST_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for NonNullType {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == NON_NULL_TYPE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Directive {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVE
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for Description {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DESCRIPTION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for RootOperationTypeDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ROOT_OPERATION_TYPE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ImplementsInterfaces {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == IMPLEMENTS_INTERFACES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FieldsDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FIELDS_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for FieldDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == FIELD_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for ArgumentsDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ARGUMENTS_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InputValueDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INPUT_VALUE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for UnionMemberTypes {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == UNION_MEMBER_TYPES
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for EnumValuesDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_VALUES_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for EnumValueDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == ENUM_VALUE_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for InputFieldsDefinition {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == INPUT_FIELDS_DEFINITION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for DirectiveLocations {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVE_LOCATIONS
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl AstNode for DirectiveLocation {
    fn can_cast(kind: SyntaxKind) -> bool {
        kind == DIRECTIVE_LOCATION
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode {
        &self.syntax
    }
}
impl From<OperationDefinition> for Definition {
    fn from(node: OperationDefinition) -> Definition {
        Definition::OperationDefinition(node)
    }
}
impl From<FragmentDefinition> for Definition {
    fn from(node: FragmentDefinition) -> Definition {
        Definition::FragmentDefinition(node)
    }
}
impl From<DirectiveDefinition> for Definition {
    fn from(node: DirectiveDefinition) -> Definition {
        Definition::DirectiveDefinition(node)
    }
}
impl From<SchemaDefinition> for Definition {
    fn from(node: SchemaDefinition) -> Definition {
        Definition::SchemaDefinition(node)
    }
}
impl From<ScalarTypeDefinition> for Definition {
    fn from(node: ScalarTypeDefinition) -> Definition {
        Definition::ScalarTypeDefinition(node)
    }
}
impl From<ObjectTypeDefinition> for Definition {
    fn from(node: ObjectTypeDefinition) -> Definition {
        Definition::ObjectTypeDefinition(node)
    }
}
impl From<InterfaceTypeDefinition> for Definition {
    fn from(node: InterfaceTypeDefinition) -> Definition {
        Definition::InterfaceTypeDefinition(node)
    }
}
impl From<UnionTypeDefinition> for Definition {
    fn from(node: UnionTypeDefinition) -> Definition {
        Definition::UnionTypeDefinition(node)
    }
}
impl From<EnumTypeDefinition> for Definition {
    fn from(node: EnumTypeDefinition) -> Definition {
        Definition::EnumTypeDefinition(node)
    }
}
impl From<InputObjectTypeDefinition> for Definition {
    fn from(node: InputObjectTypeDefinition) -> Definition {
        Definition::InputObjectTypeDefinition(node)
    }
}
impl From<SchemaExtension> for Definition {
    fn from(node: SchemaExtension) -> Definition {
        Definition::SchemaExtension(node)
    }
}
impl From<ScalarTypeExtension> for Definition {
    fn from(node: ScalarTypeExtension) -> Definition {
        Definition::ScalarTypeExtension(node)
    }
}
impl From<ObjectTypeExtension> for Definition {
    fn from(node: ObjectTypeExtension) -> Definition {
        Definition::ObjectTypeExtension(node)
    }
}
impl From<InterfaceTypeExtension> for Definition {
    fn from(node: InterfaceTypeExtension) -> Definition {
        Definition::InterfaceTypeExtension(node)
    }
}
impl From<UnionTypeExtension> for Definition {
    fn from(node: UnionTypeExtension) -> Definition {
        Definition::UnionTypeExtension(node)
    }
}
impl From<EnumTypeExtension> for Definition {
    fn from(node: EnumTypeExtension) -> Definition {
        Definition::EnumTypeExtension(node)
    }
}
impl From<InputObjectTypeExtension> for Definition {
    fn from(node: InputObjectTypeExtension) -> Definition {
        Definition::InputObjectTypeExtension(node)
    }
}
impl AstNode for Definition {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            OPERATION_DEFINITION
                | FRAGMENT_DEFINITION
                | DIRECTIVE_DEFINITION
                | SCHEMA_DEFINITION
                | SCALAR_TYPE_DEFINITION
                | OBJECT_TYPE_DEFINITION
                | INTERFACE_TYPE_DEFINITION
                | UNION_TYPE_DEFINITION
                | ENUM_TYPE_DEFINITION
                | INPUT_OBJECT_TYPE_DEFINITION
                | SCHEMA_EXTENSION
                | SCALAR_TYPE_EXTENSION
                | OBJECT_TYPE_EXTENSION
                | INTERFACE_TYPE_EXTENSION
                | UNION_TYPE_EXTENSION
                | ENUM_TYPE_EXTENSION
                | INPUT_OBJECT_TYPE_EXTENSION
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            OPERATION_DEFINITION => Definition::OperationDefinition(OperationDefinition { syntax }),
            FRAGMENT_DEFINITION => Definition::FragmentDefinition(FragmentDefinition { syntax }),
            DIRECTIVE_DEFINITION => Definition::DirectiveDefinition(DirectiveDefinition { syntax }),
            SCHEMA_DEFINITION => Definition::SchemaDefinition(SchemaDefinition { syntax }),
            SCALAR_TYPE_DEFINITION => {
                Definition::ScalarTypeDefinition(ScalarTypeDefinition { syntax })
            }
            OBJECT_TYPE_DEFINITION => {
                Definition::ObjectTypeDefinition(ObjectTypeDefinition { syntax })
            }
            INTERFACE_TYPE_DEFINITION => {
                Definition::InterfaceTypeDefinition(InterfaceTypeDefinition { syntax })
            }
            UNION_TYPE_DEFINITION => {
                Definition::UnionTypeDefinition(UnionTypeDefinition { syntax })
            }
            ENUM_TYPE_DEFINITION => Definition::EnumTypeDefinition(EnumTypeDefinition { syntax }),
            INPUT_OBJECT_TYPE_DEFINITION => {
                Definition::InputObjectTypeDefinition(InputObjectTypeDefinition { syntax })
            }
            SCHEMA_EXTENSION => Definition::SchemaExtension(SchemaExtension { syntax }),
            SCALAR_TYPE_EXTENSION => {
                Definition::ScalarTypeExtension(ScalarTypeExtension { syntax })
            }
            OBJECT_TYPE_EXTENSION => {
                Definition::ObjectTypeExtension(ObjectTypeExtension { syntax })
            }
            INTERFACE_TYPE_EXTENSION => {
                Definition::InterfaceTypeExtension(InterfaceTypeExtension { syntax })
            }
            UNION_TYPE_EXTENSION => Definition::UnionTypeExtension(UnionTypeExtension { syntax }),
            ENUM_TYPE_EXTENSION => Definition::EnumTypeExtension(EnumTypeExtension { syntax }),
            INPUT_OBJECT_TYPE_EXTENSION => {
                Definition::InputObjectTypeExtension(InputObjectTypeExtension { syntax })
            }
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Definition::OperationDefinition(it) => it.syntax(),
            Definition::FragmentDefinition(it) => it.syntax(),
            Definition::DirectiveDefinition(it) => it.syntax(),
            Definition::SchemaDefinition(it) => it.syntax(),
            Definition::ScalarTypeDefinition(it) => it.syntax(),
            Definition::ObjectTypeDefinition(it) => it.syntax(),
            Definition::InterfaceTypeDefinition(it) => it.syntax(),
            Definition::UnionTypeDefinition(it) => it.syntax(),
            Definition::EnumTypeDefinition(it) => it.syntax(),
            Definition::InputObjectTypeDefinition(it) => it.syntax(),
            Definition::SchemaExtension(it) => it.syntax(),
            Definition::ScalarTypeExtension(it) => it.syntax(),
            Definition::ObjectTypeExtension(it) => it.syntax(),
            Definition::InterfaceTypeExtension(it) => it.syntax(),
            Definition::UnionTypeExtension(it) => it.syntax(),
            Definition::EnumTypeExtension(it) => it.syntax(),
            Definition::InputObjectTypeExtension(it) => it.syntax(),
        }
    }
}
impl From<Field> for Selection {
    fn from(node: Field) -> Selection {
        Selection::Field(node)
    }
}
impl From<FragmentSpread> for Selection {
    fn from(node: FragmentSpread) -> Selection {
        Selection::FragmentSpread(node)
    }
}
impl From<InlineFragment> for Selection {
    fn from(node: InlineFragment) -> Selection {
        Selection::InlineFragment(node)
    }
}
impl AstNode for Selection {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, FIELD | FRAGMENT_SPREAD | INLINE_FRAGMENT)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            FIELD => Selection::Field(Field { syntax }),
            FRAGMENT_SPREAD => Selection::FragmentSpread(FragmentSpread { syntax }),
            INLINE_FRAGMENT => Selection::InlineFragment(InlineFragment { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Selection::Field(it) => it.syntax(),
            Selection::FragmentSpread(it) => it.syntax(),
            Selection::InlineFragment(it) => it.syntax(),
        }
    }
}
impl From<Variable> for Value {
    fn from(node: Variable) -> Value {
        Value::Variable(node)
    }
}
impl From<StringValue> for Value {
    fn from(node: StringValue) -> Value {
        Value::StringValue(node)
    }
}
impl From<FloatValue> for Value {
    fn from(node: FloatValue) -> Value {
        Value::FloatValue(node)
    }
}
impl From<IntValue> for Value {
    fn from(node: IntValue) -> Value {
        Value::IntValue(node)
    }
}
impl From<BooleanValue> for Value {
    fn from(node: BooleanValue) -> Value {
        Value::BooleanValue(node)
    }
}
impl From<NullValue> for Value {
    fn from(node: NullValue) -> Value {
        Value::NullValue(node)
    }
}
impl From<EnumValue> for Value {
    fn from(node: EnumValue) -> Value {
        Value::EnumValue(node)
    }
}
impl From<ListValue> for Value {
    fn from(node: ListValue) -> Value {
        Value::ListValue(node)
    }
}
impl From<ObjectValue> for Value {
    fn from(node: ObjectValue) -> Value {
        Value::ObjectValue(node)
    }
}
impl AstNode for Value {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(
            kind,
            VARIABLE
                | STRING_VALUE
                | FLOAT_VALUE
                | INT_VALUE
                | BOOLEAN_VALUE
                | NULL_VALUE
                | ENUM_VALUE
                | LIST_VALUE
                | OBJECT_VALUE
        )
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            VARIABLE => Value::Variable(Variable { syntax }),
            STRING_VALUE => Value::StringValue(StringValue { syntax }),
            FLOAT_VALUE => Value::FloatValue(FloatValue { syntax }),
            INT_VALUE => Value::IntValue(IntValue { syntax }),
            BOOLEAN_VALUE => Value::BooleanValue(BooleanValue { syntax }),
            NULL_VALUE => Value::NullValue(NullValue { syntax }),
            ENUM_VALUE => Value::EnumValue(EnumValue { syntax }),
            LIST_VALUE => Value::ListValue(ListValue { syntax }),
            OBJECT_VALUE => Value::ObjectValue(ObjectValue { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Value::Variable(it) => it.syntax(),
            Value::StringValue(it) => it.syntax(),
            Value::FloatValue(it) => it.syntax(),
            Value::IntValue(it) => it.syntax(),
            Value::BooleanValue(it) => it.syntax(),
            Value::NullValue(it) => it.syntax(),
            Value::EnumValue(it) => it.syntax(),
            Value::ListValue(it) => it.syntax(),
            Value::ObjectValue(it) => it.syntax(),
        }
    }
}
impl From<NamedType> for Type {
    fn from(node: NamedType) -> Type {
        Type::NamedType(node)
    }
}
impl From<ListType> for Type {
    fn from(node: ListType) -> Type {
        Type::ListType(node)
    }
}
impl From<NonNullType> for Type {
    fn from(node: NonNullType) -> Type {
        Type::NonNullType(node)
    }
}
impl AstNode for Type {
    fn can_cast(kind: SyntaxKind) -> bool {
        matches!(kind, NAMED_TYPE | LIST_TYPE | NON_NULL_TYPE)
    }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            NAMED_TYPE => Type::NamedType(NamedType { syntax }),
            LIST_TYPE => Type::ListType(ListType { syntax }),
            NON_NULL_TYPE => Type::NonNullType(NonNullType { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Type::NamedType(it) => it.syntax(),
            Type::ListType(it) => it.syntax(),
            Type::NonNullType(it) => it.syntax(),
        }
    }
}
