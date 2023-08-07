use super::Ref;
use crate::ast;
use crate::ast::AstNode;
use crate::mir;
use crate::BowString;

impl From<ast::Document> for mir::Document {
    fn from(value: ast::Document) -> Self {
        Self {
            definitions: value
                .definitions()
                .filter_map(|def| def.convert())
                .collect(),
        }
    }
}

/// Similar to `TryFrom`, but with an `Option` return type because AST uses Option a lot.
trait Convert {
    type Target;
    fn convert(&self) -> Option<Self::Target>;
}

/// Convert and collect, silently skipping entries with conversion errors
/// as they have corresponding parse errors in `SyntaxTree::errors`
#[inline]
fn collect<AstType, MirType>(iter: impl IntoIterator<Item = AstType>) -> Vec<Ref<MirType>>
where
    AstType: Convert<Target = MirType>,
{
    iter.into_iter()
        .filter_map(|value| value.convert())
        .map(Ref::new)
        .collect()
}

#[inline]
fn collect_opt<AstType1, AstType2, MirType, F, I>(
    opt: Option<AstType1>,
    convert: F,
) -> Vec<Ref<MirType>>
where
    F: FnOnce(AstType1) -> I,
    I: IntoIterator<Item = AstType2>,
    AstType2: Convert<Target = MirType>,
{
    if let Some(ast) = opt {
        collect(convert(ast))
    } else {
        Vec::new()
    }
}

impl<T: Convert> Convert for Option<T> {
    type Target = Option<T::Target>;

    fn convert(&self) -> Option<Self::Target> {
        Some(if let Some(inner) = self {
            Some(inner.convert()?)
        } else {
            None
        })
    }
}

impl Convert for ast::Definition {
    type Target = mir::Definition;

    fn convert(&self) -> Option<Self::Target> {
        use ast::Definition as A;
        use mir::Definition as M;
        fn arc<T>(x: T) -> Ref<T> {
            Ref::new(x)
        }
        Some(match self {
            A::OperationDefinition(def) => M::OperationDefinition(arc(def.convert()?)),
            A::FragmentDefinition(def) => M::FragmentDefinition(arc(def.convert()?)),
            A::DirectiveDefinition(def) => M::DirectiveDefinition(arc(def.convert()?)),
            A::SchemaDefinition(def) => M::SchemaDefinition(arc(def.convert()?)),
            A::ScalarTypeDefinition(def) => M::ScalarTypeDefinition(arc(def.convert()?)),
            A::ObjectTypeDefinition(def) => M::ObjectTypeDefinition(arc(def.convert()?)),
            A::InterfaceTypeDefinition(def) => M::InterfaceTypeDefinition(arc(def.convert()?)),
            A::UnionTypeDefinition(def) => M::UnionTypeDefinition(arc(def.convert()?)),
            A::EnumTypeDefinition(def) => M::EnumTypeDefinition(arc(def.convert()?)),
            A::InputObjectTypeDefinition(def) => M::InputObjectTypeDefinition(arc(def.convert()?)),
            A::SchemaExtension(def) => M::SchemaExtension(arc(def.convert()?)),
            A::ScalarTypeExtension(def) => M::ScalarTypeExtension(arc(def.convert()?)),
            A::ObjectTypeExtension(def) => M::ObjectTypeExtension(arc(def.convert()?)),
            A::InterfaceTypeExtension(def) => M::InterfaceTypeExtension(arc(def.convert()?)),
            A::UnionTypeExtension(def) => M::UnionTypeExtension(arc(def.convert()?)),
            A::EnumTypeExtension(def) => M::EnumTypeExtension(arc(def.convert()?)),
            A::InputObjectTypeExtension(def) => M::InputObjectTypeExtension(arc(def.convert()?)),
        })
    }
}

impl Convert for ast::OperationDefinition {
    type Target = mir::OperationDefinition;

    fn convert(&self) -> Option<Self::Target> {
        let operation_type = if let Some(ty) = self.operation_type() {
            ty.convert()?
        } else {
            mir::OperationType::Query
        };
        Some(Self::Target {
            operation_type,
            name: self.name().map(From::from),
            variables: collect_opt(self.variable_definitions(), |x| x.variable_definitions()),
            directives: collect_opt(self.directives(), |x| x.directives()),
            selection_set: self
                .selection_set()?
                .selections()
                .filter_map(|sel| sel.convert())
                .collect(),
        })
    }
}

impl Convert for ast::FragmentDefinition {
    type Target = mir::FragmentDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.fragment_name()?.name()?.into(),
            type_condition: self.type_condition()?.convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            selection_set: self.selection_set().convert()??,
        })
    }
}

impl Convert for ast::TypeCondition {
    type Target = BowString;

    fn convert(&self) -> Option<Self::Target> {
        Some(self.named_type()?.name()?.into())
    }
}

impl Convert for ast::DirectiveDefinition {
    type Target = mir::DirectiveDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            arguments: collect_opt(self.arguments_definition(), |x| x.input_value_definitions()),
            repeatable: self.repeatable_token().is_some(),
            locations: self
                .directive_locations()
                .map(|x| {
                    x.directive_locations()
                        .filter_map(|location| location.convert())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl Convert for ast::SchemaDefinition {
    type Target = mir::SchemaDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            root_operations: self
                .root_operation_type_definitions()
                .filter_map(|x| x.convert())
                .collect(),
        })
    }
}

impl Convert for ast::ScalarTypeDefinition {
    type Target = mir::ScalarTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::ObjectTypeDefinition {
    type Target = mir::ObjectTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            implements_interfaces: self.implements_interfaces().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.fields_definition()?.field_definitions()),
        })
    }
}

impl Convert for ast::InterfaceTypeDefinition {
    type Target = mir::InterfaceTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            implements_interfaces: self.implements_interfaces().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.fields_definition()?.field_definitions()),
        })
    }
}

impl Convert for ast::UnionTypeDefinition {
    type Target = mir::UnionTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            members: self
                .union_member_types()?
                .named_types()
                .filter_map(|n| n.name())
                .map(From::from)
                .collect(),
        })
    }
}

impl Convert for ast::EnumTypeDefinition {
    type Target = mir::EnumTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            values: collect(self.enum_values_definition()?.enum_value_definitions()),
        })
    }
}

impl Convert for ast::InputObjectTypeDefinition {
    type Target = mir::InputObjectTypeDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.input_fields_definition()?.input_value_definitions()),
        })
    }
}

impl Convert for ast::SchemaExtension {
    type Target = mir::SchemaExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            directives: collect_opt(self.directives(), |x| x.directives()),
            root_operations: self
                .root_operation_type_definitions()
                .filter_map(|x| x.convert())
                .collect(),
        })
    }
}

impl Convert for ast::ScalarTypeExtension {
    type Target = mir::ScalarTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::ObjectTypeExtension {
    type Target = mir::ObjectTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            implements_interfaces: self.implements_interfaces().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.fields_definition()?.field_definitions()),
        })
    }
}

impl Convert for ast::InterfaceTypeExtension {
    type Target = mir::InterfaceTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            implements_interfaces: self.implements_interfaces().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.fields_definition()?.field_definitions()),
        })
    }
}

impl Convert for ast::UnionTypeExtension {
    type Target = mir::UnionTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            members: self
                .union_member_types()?
                .named_types()
                .filter_map(|n| n.name())
                .map(From::from)
                .collect(),
        })
    }
}

impl Convert for ast::EnumTypeExtension {
    type Target = mir::EnumTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            values: collect(self.enum_values_definition()?.enum_value_definitions()),
        })
    }
}

impl Convert for ast::InputObjectTypeExtension {
    type Target = mir::InputObjectTypeExtension;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            fields: collect(self.input_fields_definition()?.input_value_definitions()),
        })
    }
}

impl Convert for ast::Description {
    type Target = BowString;

    fn convert(&self) -> Option<Self::Target> {
        Some(String::from(self.string_value()?).into())
    }
}

impl Convert for ast::Directive {
    type Target = mir::Directive;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.into(),
            arguments: self
                .arguments()
                .map(|x| x.arguments().filter_map(|arg| arg.convert()).collect())
                .unwrap_or_default(),
        })
    }
}

impl Convert for ast::OperationType {
    type Target = mir::OperationType;

    fn convert(&self) -> Option<Self::Target> {
        if self.query_token().is_some() {
            Some(mir::OperationType::Query)
        } else if self.mutation_token().is_some() {
            Some(mir::OperationType::Mutation)
        } else if self.subscription_token().is_some() {
            Some(mir::OperationType::Subscription)
        } else {
            None // TODO: unreachable?
        }
    }
}

impl Convert for ast::RootOperationTypeDefinition {
    type Target = (mir::OperationType, mir::NamedType);

    fn convert(&self) -> Option<Self::Target> {
        let ty = self.operation_type()?.convert()?;
        let name = self.named_type()?.name()?.into();
        Some((ty, name))
    }
}

impl Convert for ast::DirectiveLocation {
    type Target = mir::DirectiveLocation;

    fn convert(&self) -> Option<Self::Target> {
        if self.query_token().is_some() {
            Some(mir::DirectiveLocation::Query)
        } else if self.mutation_token().is_some() {
            Some(mir::DirectiveLocation::Mutation)
        } else if self.subscription_token().is_some() {
            Some(mir::DirectiveLocation::Subscription)
        } else if self.field_token().is_some() {
            Some(mir::DirectiveLocation::Field)
        } else if self.fragment_definition_token().is_some() {
            Some(mir::DirectiveLocation::FragmentDefinition)
        } else if self.fragment_spread_token().is_some() {
            Some(mir::DirectiveLocation::FragmentSpread)
        } else if self.inline_fragment_token().is_some() {
            Some(mir::DirectiveLocation::InlineFragment)
        } else if self.variable_definition_token().is_some() {
            Some(mir::DirectiveLocation::VariableDefinition)
        } else if self.schema_token().is_some() {
            Some(mir::DirectiveLocation::Schema)
        } else if self.scalar_token().is_some() {
            Some(mir::DirectiveLocation::Scalar)
        } else if self.object_token().is_some() {
            Some(mir::DirectiveLocation::Object)
        } else if self.field_definition_token().is_some() {
            Some(mir::DirectiveLocation::FieldDefinition)
        } else if self.argument_definition_token().is_some() {
            Some(mir::DirectiveLocation::ArgumentDefinition)
        } else if self.interface_token().is_some() {
            Some(mir::DirectiveLocation::Interface)
        } else if self.union_token().is_some() {
            Some(mir::DirectiveLocation::Union)
        } else if self.enum_token().is_some() {
            Some(mir::DirectiveLocation::Enum)
        } else if self.enum_value_token().is_some() {
            Some(mir::DirectiveLocation::EnumValue)
        } else if self.input_object_token().is_some() {
            Some(mir::DirectiveLocation::InputObject)
        } else if self.input_field_definition_token().is_some() {
            Some(mir::DirectiveLocation::InputFieldDefinition)
        } else {
            None // TODO: unreachable?
        }
    }
}

impl Convert for Option<ast::ImplementsInterfaces> {
    type Target = Vec<mir::NamedType>;

    fn convert(&self) -> Option<Self::Target> {
        Some(if let Some(inner) = self {
            inner
                .named_types()
                .filter_map(|n| n.name())
                .map(From::from)
                .collect()
        } else {
            Vec::new()
        })
    }
}

impl Convert for ast::VariableDefinition {
    type Target = mir::VariableDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.variable()?.name()?.into(),
            ty: self.ty()?.try_into().ok()?,
            default_value: self.default_value().and_then(|x| x.value()).convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::Type {
    type Target = mir::Type;

    fn convert(&self) -> Option<Self::Target> {
        use ast::Type as A;
        use mir::Type as M;
        match self {
            A::NamedType(name) => Some(M::Named(name.name()?.into())),
            A::ListType(inner) => Some(M::List(Box::new(inner.ty()?.convert()?))),
            A::NonNullType(inner) => {
                if let Some(named) = inner.named_type() {
                    Some(M::NonNullNamed(named.name()?.into()))
                } else if let Some(list) = inner.list_type() {
                    Some(M::NonNullList(Box::new(list.ty()?.convert()?)))
                } else {
                    None
                }
            }
        }
    }
}

impl Convert for ast::FieldDefinition {
    type Target = mir::FieldDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            arguments: collect_opt(self.arguments_definition(), |x| x.input_value_definitions()),
            ty: self.ty()?.convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::Argument {
    type Target = (mir::Name, mir::Value);

    fn convert(&self) -> Option<Self::Target> {
        let name = self.name()?.into();
        let value = self.value()?.convert()?;
        Some((name, value))
    }
}

impl Convert for ast::InputValueDefinition {
    type Target = mir::InputValueDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            name: self.name()?.into(),
            ty: self.ty()?.convert()?,
            default_value: self.default_value().and_then(|x| x.value()).convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::EnumValueDefinition {
    type Target = mir::EnumValueDefinition;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert()?,
            value: self.enum_value()?.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::SelectionSet {
    type Target = Vec<mir::Selection>;

    fn convert(&self) -> Option<Self::Target> {
        Some(
            self.selections()
                .filter_map(|selection| selection.convert())
                .collect(),
        )
    }
}

impl Convert for ast::Selection {
    type Target = mir::Selection;

    fn convert(&self) -> Option<Self::Target> {
        use ast::Selection as A;
        use mir::Selection as M;

        Some(match self {
            A::Field(x) => M::Field(Ref::new(x.convert()?)),
            A::FragmentSpread(x) => M::FragmentSpread(Ref::new(x.convert()?)),
            A::InlineFragment(x) => M::InlineFragment(Ref::new(x.convert()?)),
        })
    }
}

impl Convert for ast::Field {
    type Target = mir::Field;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            alias: self.alias().convert()?,
            name: self.name()?.into(),
            arguments: self
                .arguments()
                .map(|x| x.arguments().filter_map(|arg| arg.convert()).collect())
                .unwrap_or_default(),
            directives: collect_opt(self.directives(), |x| x.directives()),
            // Use an empty Vec for a field without sub-selections
            selection_set: self.selection_set().convert()?.unwrap_or_default(),
        })
    }
}

impl Convert for ast::FragmentSpread {
    type Target = mir::FragmentSpread;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            fragment_name: self.fragment_name()?.name()?.into(),
            directives: collect_opt(self.directives(), |x| x.directives()),
        })
    }
}

impl Convert for ast::InlineFragment {
    type Target = mir::InlineFragment;

    fn convert(&self) -> Option<Self::Target> {
        Some(Self::Target {
            type_condition: self.type_condition().convert()?,
            directives: collect_opt(self.directives(), |x| x.directives()),
            selection_set: self.selection_set().convert()??,
        })
    }
}

impl Convert for ast::Value {
    type Target = mir::Value;

    fn convert(&self) -> Option<Self::Target> {
        use ast::Value as A;
        use mir::Value as M;

        Some(match self {
            A::Variable(v) => M::Variable(v.name()?.into()),
            A::StringValue(v) => M::String(String::from(v).into()),
            A::FloatValue(v) => M::Float(f64::try_from(v).ok()?),
            A::IntValue(v) => {
                if let Ok(i) = i32::try_from(v) {
                    M::Int(i)
                } else {
                    let text = &ast::text_of_first_token(v.syntax());
                    let text = text.as_str();
                    debug_assert!(text.chars().all(|c| c.is_ascii_digit()));
                    M::BigInt(text.into())
                }
            }
            A::BooleanValue(v) => M::Boolean(bool::try_from(v).ok()?),
            A::NullValue(_) => M::Null,
            A::EnumValue(v) => M::Enum(v.name()?.into()),
            A::ListValue(v) => M::List(collect(v.values())),
            A::ObjectValue(v) => M::Object(v.object_fields().filter_map(|x| x.convert()).collect()),
        })
    }
}

impl Convert for ast::ObjectField {
    type Target = (mir::Name, Ref<mir::Value>);

    fn convert(&self) -> Option<Self::Target> {
        let name = self.name()?.into();
        let value = Ref::new(self.value()?.convert()?);
        Some((name, value))
    }
}

impl Convert for ast::Alias {
    type Target = mir::Name;

    fn convert(&self) -> Option<Self::Target> {
        Some(self.name()?.into())
    }
}

impl From<ast::Name> for BowString {
    fn from(value: ast::Name) -> Self {
        value.text().as_str().into()
    }
}

/// Also implement a public conversion trait for external callers
macro_rules! try_from {
    ($($ty: ident)+) => {
        $(
            impl TryFrom<ast::$ty> for mir::$ty {
                type Error = InvalidAst;

                fn try_from(value: ast::$ty) -> Result<Self, Self::Error> {
                    value.convert().ok_or(InvalidAst)
                }
            }
        )+
    };
}

/// The AST does not conform to the grammar
#[derive(Debug)]
pub struct InvalidAst;

try_from! {
    Definition
    OperationDefinition
    FragmentDefinition
    DirectiveDefinition
    SchemaDefinition
    ScalarTypeDefinition
    ObjectTypeDefinition
    InterfaceTypeDefinition
    UnionTypeDefinition
    EnumTypeDefinition
    InputObjectTypeDefinition
    SchemaExtension
    ScalarTypeExtension
    ObjectTypeExtension
    InterfaceTypeExtension
    UnionTypeExtension
    EnumTypeExtension
    InputObjectTypeExtension
    Directive
    OperationType
    VariableDefinition
    Type
    FieldDefinition
    EnumValueDefinition
    Selection
    Field
    FragmentSpread
    InlineFragment
    Value
}
