use crate::ast;
use crate::ast::Document;
use crate::Arc;
use crate::FileId;
use crate::Node;
use crate::NodeLocation;
use crate::SourceFile;
use apollo_parser::cst;
use apollo_parser::cst::CstNode;
use apollo_parser::SyntaxNode;
use apollo_parser::S;

impl Document {
    pub(crate) fn from_cst(
        document: cst::Document,
        file_id: FileId,
        source_file: Arc<SourceFile>,
    ) -> Self {
        Self {
            source: Some((file_id, source_file)),
            definitions: document
                .definitions()
                .filter_map(|def| def.convert(file_id))
                .collect(),
        }
    }
}

/// Similar to `TryFrom`, but with an `Option` return type because AST uses Option a lot.
trait Convert {
    type Target;
    fn convert(&self, file_id: FileId) -> Option<Self::Target>;
}

fn with_location<T>(file_id: FileId, syntax_node: &SyntaxNode, node: T) -> Node<T> {
    Node::new_parsed(node, NodeLocation::new(file_id, syntax_node))
}

/// Convert and collect, silently skipping entries with conversion errors
/// as they have corresponding parse errors in `SyntaxTree::errors`
#[inline]
fn collect<CstType, AstType>(
    file_id: FileId,
    iter: impl IntoIterator<Item = CstType>,
) -> Vec<Node<AstType>>
where
    CstType: CstNode + Convert<Target = AstType>,
{
    iter.into_iter()
        .filter_map(|value| {
            Some(with_location(
                file_id,
                value.syntax(),
                value.convert(file_id)?,
            ))
        })
        .collect()
}

#[inline]
fn collect_opt<CstType1, CstType2, AstType, F, I>(
    file_id: FileId,
    opt: Option<CstType1>,
    convert: F,
) -> Vec<Node<AstType>>
where
    F: FnOnce(CstType1) -> I,
    I: IntoIterator<Item = CstType2>,
    CstType2: CstNode + Convert<Target = AstType>,
{
    if let Some(cst) = opt {
        collect(file_id, convert(cst))
    } else {
        Vec::new()
    }
}

impl<T: Convert> Convert for Option<T> {
    type Target = Option<T::Target>;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(if let Some(inner) = self {
            Some(inner.convert(file_id)?)
        } else {
            None
        })
    }
}

impl Convert for cst::Definition {
    type Target = ast::Definition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        use ast::Definition as A;
        use cst::Definition as C;
        macro_rules! r {
            ($def: ident) => {
                with_location(file_id, $def.syntax(), $def.convert(file_id)?)
            };
        }
        Some(match self {
            C::OperationDefinition(def) => A::OperationDefinition(r!(def)),
            C::FragmentDefinition(def) => A::FragmentDefinition(r!(def)),
            C::DirectiveDefinition(def) => A::DirectiveDefinition(r!(def)),
            C::SchemaDefinition(def) => A::SchemaDefinition(r!(def)),
            C::ScalarTypeDefinition(def) => A::ScalarTypeDefinition(r!(def)),
            C::ObjectTypeDefinition(def) => A::ObjectTypeDefinition(r!(def)),
            C::InterfaceTypeDefinition(def) => A::InterfaceTypeDefinition(r!(def)),
            C::UnionTypeDefinition(def) => A::UnionTypeDefinition(r!(def)),
            C::EnumTypeDefinition(def) => A::EnumTypeDefinition(r!(def)),
            C::InputObjectTypeDefinition(def) => A::InputObjectTypeDefinition(r!(def)),
            C::SchemaExtension(def) => A::SchemaExtension(r!(def)),
            C::ScalarTypeExtension(def) => A::ScalarTypeExtension(r!(def)),
            C::ObjectTypeExtension(def) => A::ObjectTypeExtension(r!(def)),
            C::InterfaceTypeExtension(def) => A::InterfaceTypeExtension(r!(def)),
            C::UnionTypeExtension(def) => A::UnionTypeExtension(r!(def)),
            C::EnumTypeExtension(def) => A::EnumTypeExtension(r!(def)),
            C::InputObjectTypeExtension(def) => A::InputObjectTypeExtension(r!(def)),
        })
    }
}

impl Convert for cst::OperationDefinition {
    type Target = ast::OperationDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let operation_type = if let Some(ty) = self.operation_type() {
            ty.convert(file_id)?
        } else {
            ast::OperationType::Query
        };
        Some(Self::Target {
            operation_type,
            name: self.name().convert(file_id)?,
            variables: collect_opt(file_id, self.variable_definitions(), |x| {
                x.variable_definitions()
            }),
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            selection_set: self
                .selection_set()?
                .selections()
                .filter_map(|sel| sel.convert(file_id))
                .collect(),
        })
    }
}

impl Convert for cst::FragmentDefinition {
    type Target = ast::FragmentDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.fragment_name()?.name()?.convert(file_id)?,
            type_condition: self.type_condition()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            selection_set: self.selection_set().convert(file_id)??,
        })
    }
}

impl Convert for cst::TypeCondition {
    type Target = ast::NamedType;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        self.named_type()?.name()?.convert(file_id)
    }
}

impl Convert for cst::DirectiveDefinition {
    type Target = ast::DirectiveDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            arguments: collect_opt(file_id, self.arguments_definition(), |x| {
                x.input_value_definitions()
            }),
            repeatable: self.repeatable_token().is_some(),
            locations: self
                .directive_locations()
                .map(|x| {
                    x.directive_locations()
                        .filter_map(|location| location.convert(file_id))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl Convert for cst::SchemaDefinition {
    type Target = ast::SchemaDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            // This may represent a syntactically invalid thing: a schema without any root
            // operation definitions. However the presence of a broken schema definition does
            // affect whether a default schema definition should be inserted, so we bubble up the
            // potentially invalid definition.
            root_operations: self
                .root_operation_type_definitions()
                .filter_map(|x| x.convert(file_id))
                .collect(),
        })
    }
}

impl Convert for cst::ScalarTypeDefinition {
    type Target = ast::ScalarTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::ObjectTypeDefinition {
    type Target = ast::ObjectTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            implements_interfaces: self.implements_interfaces().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.fields_definition(), |x| x.field_definitions()),
        })
    }
}

impl Convert for cst::InterfaceTypeDefinition {
    type Target = ast::InterfaceTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            implements_interfaces: self.implements_interfaces().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.fields_definition(), |x| x.field_definitions()),
        })
    }
}

impl Convert for cst::UnionTypeDefinition {
    type Target = ast::UnionTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            members: self
                .union_member_types()
                .map_or_else(Default::default, |member_types| {
                    member_types
                        .named_types()
                        .filter_map(|n| n.name()?.convert(file_id))
                        .collect()
                }),
        })
    }
}

impl Convert for cst::EnumTypeDefinition {
    type Target = ast::EnumTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            values: collect_opt(file_id, self.enum_values_definition(), |x| {
                x.enum_value_definitions()
            }),
        })
    }
}

impl Convert for cst::InputObjectTypeDefinition {
    type Target = ast::InputObjectTypeDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.input_fields_definition(), |x| {
                x.input_value_definitions()
            }),
        })
    }
}

impl Convert for cst::SchemaExtension {
    type Target = ast::SchemaExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            root_operations: self
                .root_operation_type_definitions()
                .filter_map(|x| x.convert(file_id))
                .collect(),
        })
    }
}

impl Convert for cst::ScalarTypeExtension {
    type Target = ast::ScalarTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::ObjectTypeExtension {
    type Target = ast::ObjectTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            implements_interfaces: self.implements_interfaces().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.fields_definition(), |x| x.field_definitions()),
        })
    }
}

impl Convert for cst::InterfaceTypeExtension {
    type Target = ast::InterfaceTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            implements_interfaces: self.implements_interfaces().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.fields_definition(), |x| x.field_definitions()),
        })
    }
}

impl Convert for cst::UnionTypeExtension {
    type Target = ast::UnionTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            members: self
                .union_member_types()
                .map_or_else(Default::default, |member_types| {
                    member_types
                        .named_types()
                        .filter_map(|n| n.name()?.convert(file_id))
                        .collect()
                }),
        })
    }
}

impl Convert for cst::EnumTypeExtension {
    type Target = ast::EnumTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            values: collect_opt(file_id, self.enum_values_definition(), |x| {
                x.enum_value_definitions()
            }),
        })
    }
}

impl Convert for cst::InputObjectTypeExtension {
    type Target = ast::InputObjectTypeExtension;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            fields: collect_opt(file_id, self.input_fields_definition(), |x| {
                x.input_value_definitions()
            }),
        })
    }
}

impl Convert for cst::Description {
    type Target = ast::NodeStr;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(ast::NodeStr::new_parsed(
            &String::from(self.string_value()?),
            NodeLocation::new(file_id, self.syntax()),
        ))
    }
}

impl Convert for cst::Directive {
    type Target = ast::Directive;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            name: self.name()?.convert(file_id)?,
            arguments: collect_opt(file_id, self.arguments(), |x| x.arguments()),
        })
    }
}

impl Convert for cst::OperationType {
    type Target = ast::OperationType;

    fn convert(&self, _file_id: FileId) -> Option<Self::Target> {
        let token = self.syntax().first_token()?;
        match token.kind() {
            S![query] => Some(ast::OperationType::Query),
            S![mutation] => Some(ast::OperationType::Mutation),
            S![subscription] => Some(ast::OperationType::Subscription),
            _ => None, // TODO: unreachable?
        }
    }
}

impl Convert for cst::RootOperationTypeDefinition {
    type Target = Node<(ast::OperationType, ast::NamedType)>;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let ty = self.operation_type()?.convert(file_id)?;
        let name = self.named_type()?.name()?.convert(file_id)?;
        Some(with_location(file_id, self.syntax(), (ty, name)))
    }
}

impl Convert for cst::DirectiveLocation {
    type Target = ast::DirectiveLocation;

    fn convert(&self, _file_id: FileId) -> Option<Self::Target> {
        let token = self.syntax().first_token()?;
        match token.kind() {
            S![QUERY] => Some(ast::DirectiveLocation::Query),
            S![MUTATION] => Some(ast::DirectiveLocation::Mutation),
            S![SUBSCRIPTION] => Some(ast::DirectiveLocation::Subscription),
            S![FIELD] => Some(ast::DirectiveLocation::Field),
            S![FRAGMENT_DEFINITION] => Some(ast::DirectiveLocation::FragmentDefinition),
            S![FRAGMENT_SPREAD] => Some(ast::DirectiveLocation::FragmentSpread),
            S![INLINE_FRAGMENT] => Some(ast::DirectiveLocation::InlineFragment),
            S![VARIABLE_DEFINITION] => Some(ast::DirectiveLocation::VariableDefinition),
            S![SCHEMA] => Some(ast::DirectiveLocation::Schema),
            S![SCALAR] => Some(ast::DirectiveLocation::Scalar),
            S![OBJECT] => Some(ast::DirectiveLocation::Object),
            S![FIELD_DEFINITION] => Some(ast::DirectiveLocation::FieldDefinition),
            S![ARGUMENT_DEFINITION] => Some(ast::DirectiveLocation::ArgumentDefinition),
            S![INTERFACE] => Some(ast::DirectiveLocation::Interface),
            S![UNION] => Some(ast::DirectiveLocation::Union),
            S![ENUM] => Some(ast::DirectiveLocation::Enum),
            S![ENUM_VALUE] => Some(ast::DirectiveLocation::EnumValue),
            S![INPUT_OBJECT] => Some(ast::DirectiveLocation::InputObject),
            S![INPUT_FIELD_DEFINITION] => Some(ast::DirectiveLocation::InputFieldDefinition),
            _ => None, // TODO: unreachable?
        }
    }
}

impl Convert for Option<cst::ImplementsInterfaces> {
    type Target = Vec<ast::NamedType>;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(if let Some(inner) = self {
            inner
                .named_types()
                .filter_map(|n| n.name()?.convert(file_id))
                .collect()
        } else {
            Vec::new()
        })
    }
}

impl Convert for cst::VariableDefinition {
    type Target = ast::VariableDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let default_value = if let Some(default) = self.default_value() {
            let value = default.value()?;
            Some(with_location(
                file_id,
                value.syntax(),
                value.convert(file_id)?,
            ))
        } else {
            None
        };
        let ty = &self.ty()?;
        Some(Self::Target {
            name: self.variable()?.name()?.convert(file_id)?,
            ty: with_location(file_id, ty.syntax(), ty.convert(file_id)?),
            default_value,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::Type {
    type Target = ast::Type;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        use ast::Type as A;
        use cst::Type as C;
        match self {
            C::NamedType(name) => Some(A::Named(name.name()?.convert(file_id)?)),
            C::ListType(inner) => Some(A::List(Box::new(inner.ty()?.convert(file_id)?))),
            C::NonNullType(inner) => {
                if let Some(named) = inner.named_type() {
                    Some(A::NonNullNamed(named.name()?.convert(file_id)?))
                } else if let Some(list) = inner.list_type() {
                    Some(A::NonNullList(Box::new(list.ty()?.convert(file_id)?)))
                } else {
                    None
                }
            }
        }
    }
}

impl Convert for cst::FieldDefinition {
    type Target = ast::FieldDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            arguments: collect_opt(file_id, self.arguments_definition(), |x| {
                x.input_value_definitions()
            }),
            ty: self.ty()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::Argument {
    type Target = ast::Argument;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let name = self.name()?.convert(file_id)?;
        let value = self.value()?;
        let value = with_location(file_id, value.syntax(), value.convert(file_id)?);
        Some(ast::Argument { name, value })
    }
}

impl Convert for cst::InputValueDefinition {
    type Target = ast::InputValueDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let default_value = if let Some(default) = self.default_value() {
            let value = default.value()?;
            Some(with_location(
                file_id,
                value.syntax(),
                value.convert(file_id)?,
            ))
        } else {
            None
        };
        let ty = &self.ty()?;
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            ty: with_location(file_id, ty.syntax(), ty.convert(file_id)?),
            default_value,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::EnumValueDefinition {
    type Target = ast::EnumValueDefinition;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            description: self.description().convert(file_id)?,
            value: self.enum_value()?.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::SelectionSet {
    type Target = Vec<ast::Selection>;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(
            self.selections()
                .filter_map(|selection| selection.convert(file_id))
                .collect(),
        )
    }
}

impl Convert for cst::Selection {
    type Target = ast::Selection;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        use ast::Selection as A;
        use cst::Selection as C;

        Some(match self {
            C::Field(x) => A::Field(with_location(file_id, x.syntax(), x.convert(file_id)?)),
            C::FragmentSpread(x) => {
                A::FragmentSpread(with_location(file_id, x.syntax(), x.convert(file_id)?))
            }
            C::InlineFragment(x) => {
                A::InlineFragment(with_location(file_id, x.syntax(), x.convert(file_id)?))
            }
        })
    }
}

impl Convert for cst::Field {
    type Target = ast::Field;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            alias: self.alias().convert(file_id)?,
            name: self.name()?.convert(file_id)?,
            arguments: collect_opt(file_id, self.arguments(), |x| x.arguments()),
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            // Use an empty Vec for a field without sub-selections
            selection_set: self.selection_set().convert(file_id)?.unwrap_or_default(),
        })
    }
}

impl Convert for cst::FragmentSpread {
    type Target = ast::FragmentSpread;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            fragment_name: self.fragment_name()?.name()?.convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
        })
    }
}

impl Convert for cst::InlineFragment {
    type Target = ast::InlineFragment;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(Self::Target {
            type_condition: self.type_condition().convert(file_id)?,
            directives: ast::Directives(collect_opt(file_id, self.directives(), |x| {
                x.directives()
            })),
            selection_set: self.selection_set().convert(file_id)??,
        })
    }
}

impl Convert for cst::Value {
    type Target = ast::Value;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        use ast::Value as A;
        use cst::Value as C;

        Some(match self {
            C::Variable(v) => A::Variable(v.name()?.convert(file_id)?),
            C::StringValue(v) => A::String(ast::NodeStr::new_parsed(
                &String::from(v),
                NodeLocation::new(file_id, self.syntax()),
            )),
            C::FloatValue(v) => A::Float(ast::FloatValue::new_parsed(
                v.syntax().first_token()?.text(),
            )),
            C::IntValue(v) => A::Int(ast::IntValue::new_parsed(v.syntax().first_token()?.text())),
            C::BooleanValue(v) => A::Boolean(bool::try_from(v).ok()?),
            C::NullValue(_) => A::Null,
            C::EnumValue(v) => A::Enum(v.name()?.convert(file_id)?),
            C::ListValue(v) => A::List(collect(file_id, v.values())),
            C::ObjectValue(v) => A::Object(
                v.object_fields()
                    .filter_map(|x| x.convert(file_id))
                    .collect(),
            ),
        })
    }
}

impl Convert for cst::ObjectField {
    type Target = (ast::Name, Node<ast::Value>);

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        let name = self.name()?.convert(file_id)?;
        let value = with_location(file_id, self.syntax(), self.value()?.convert(file_id)?);
        Some((name, value))
    }
}

impl Convert for cst::Alias {
    type Target = ast::Name;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        self.name()?.convert(file_id)
    }
}

impl Convert for cst::Name {
    type Target = ast::Name;

    fn convert(&self, file_id: FileId) -> Option<Self::Target> {
        Some(ast::Name::new_parsed(
            self.text().as_str(),
            NodeLocation::new(file_id, self.syntax()),
        ))
    }
}
