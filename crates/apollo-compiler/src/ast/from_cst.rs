use crate::ast;
use crate::ast::Document;
use crate::parser::FileId;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::Node;
use apollo_parser::cst;
use apollo_parser::cst::CstNode;
use apollo_parser::GraphQLLanguage;
use apollo_parser::SyntaxKind as K;
use apollo_parser::SyntaxNode;
use apollo_parser::S;
use rowan::GreenNodeData;
use rowan::GreenTokenData;
use rowan::Language;
use rowan::NodeOrToken;
use rowan::TextRange;
use rowan::TextSize;

impl Document {
    pub(crate) fn from_cst(document: cst::Document, file_id: FileId, sources: SourceMap) -> Self {
        let syntax = document.syntax();
        let green = syntax.green();
        let cx = Cx { file_id };
        Self {
            sources,
            definitions: Green::root(syntax, &green)
                .children()
                .filter_map(|def| cx.definition(def))
                .collect(),
        }
    }
}

pub(crate) fn convert_selection_set(
    selection_set: &cst::SelectionSet,
    file_id: FileId,
) -> Vec<ast::Selection> {
    let syntax = selection_set.syntax();
    let green = syntax.green();
    Cx { file_id }.selection_set(Green::root(syntax, &green))
}

pub(crate) fn convert_type(ty: &cst::Type, file_id: FileId) -> Option<ast::Type> {
    let syntax = ty.syntax();
    let green = syntax.green();
    Cx { file_id }.ty(Green::root(syntax, &green))
}

/// A CST node read straight from the green tree.
///
/// Navigating with rowan's `SyntaxNode` allocates a cursor for every node it
/// visits, and the typed CST accessors rescan children from the start on
/// every call. That made up most of the conversion time. Green nodes can be
/// walked without allocating, as long as we track text offsets ourselves.
#[derive(Clone, Copy)]
struct Green<'a> {
    node: &'a GreenNodeData,
    offset: TextSize,
}

impl<'a> Green<'a> {
    fn root(syntax: &SyntaxNode, green: &'a GreenNodeData) -> Self {
        Self {
            node: green,
            offset: syntax.text_range().start(),
        }
    }

    fn kind(self) -> K {
        GraphQLLanguage::kind_from_raw(self.node.kind())
    }

    fn range(self) -> TextRange {
        TextRange::at(self.offset, self.node.text_len())
    }

    fn children(self) -> impl Iterator<Item = Green<'a>> {
        let mut offset = self.offset;
        self.node.children().filter_map(move |element| {
            let start = offset;
            offset += element.text_len();
            match element {
                NodeOrToken::Node(node) => Some(Green {
                    node,
                    offset: start,
                }),
                NodeOrToken::Token(_) => None,
            }
        })
    }

    /// The first child node of `kind`, like the typed CST accessors.
    fn child(self, kind: K) -> Option<Green<'a>> {
        self.children().find(|child| child.kind() == kind)
    }

    fn child_where(self, pred: impl Fn(K) -> bool) -> Option<Green<'a>> {
        self.children().find(|child| pred(child.kind()))
    }

    fn children_of(self, kind: K) -> impl Iterator<Item = Green<'a>> {
        self.children().filter(move |child| child.kind() == kind)
    }

    fn has_token(self, kind: K) -> bool {
        self.node.children().any(|element| match element {
            NodeOrToken::Token(token) => GraphQLLanguage::kind_from_raw(token.kind()) == kind,
            NodeOrToken::Node(_) => false,
        })
    }

    /// Follows the first child down to a token, matching `SyntaxNode::first_token`.
    fn first_token(self) -> Option<&'a GreenTokenData> {
        let mut node = self.node;
        loop {
            match node.children().next()? {
                NodeOrToken::Token(token) => return Some(token),
                NodeOrToken::Node(child) => node = child,
            }
        }
    }

    fn first_token_kind(self) -> Option<K> {
        Some(GraphQLLanguage::kind_from_raw(self.first_token()?.kind()))
    }
}

fn is_type(kind: K) -> bool {
    matches!(kind, K::NAMED_TYPE | K::LIST_TYPE | K::NON_NULL_TYPE)
}

fn is_value(kind: K) -> bool {
    matches!(
        kind,
        K::VARIABLE
            | K::STRING_VALUE
            | K::FLOAT_VALUE
            | K::INT_VALUE
            | K::BOOLEAN_VALUE
            | K::NULL_VALUE
            | K::ENUM_VALUE
            | K::LIST_VALUE
            | K::OBJECT_VALUE
    )
}

/// Conversion functions return `None` for nodes with syntax errors. Those
/// nodes are silently skipped, as they have corresponding parse errors in
/// `SyntaxTree::errors`.
#[derive(Clone, Copy)]
struct Cx {
    file_id: FileId,
}

impl Cx {
    fn location(self, node: Green<'_>) -> SourceSpan {
        SourceSpan::new(self.file_id, node.range())
    }

    fn with_location<T>(self, node: Green<'_>, value: T) -> Node<T> {
        Node::new_parsed(value, self.location(node))
    }

    /// Converts every child of `kind` inside the optional `list` node.
    fn collect<T>(
        self,
        list: Option<Green<'_>>,
        kind: K,
        convert: impl Fn(Self, Green<'_>) -> Option<T>,
    ) -> Vec<Node<T>> {
        let Some(list) = list else {
            return Vec::new();
        };
        list.children_of(kind)
            .filter_map(|item| Some(self.with_location(item, convert(self, item)?)))
            .collect()
    }

    fn directives(self, parent: Green<'_>) -> ast::DirectiveList {
        ast::DirectiveList(self.collect(parent.child(K::DIRECTIVES), K::DIRECTIVE, Self::directive))
    }

    fn description(self, parent: Green<'_>) -> Option<Option<Node<str>>> {
        let Some(description) = parent.child(K::DESCRIPTION) else {
            return Some(None);
        };
        let token = description.child(K::STRING_VALUE)?.first_token()?;
        Some(Some(Node::new_str_parsed(
            &cst::StringValue::unescape(token.text()),
            self.location(description),
        )))
    }

    fn name(self, node: Green<'_>) -> Option<crate::Name> {
        let text = node.first_token()?.text();
        debug_assert!(crate::Name::is_valid_syntax(text));
        Some(
            crate::Name::new(text)
                .ok()?
                .with_location(self.location(node)),
        )
    }

    fn child_name(self, parent: Green<'_>) -> Option<crate::Name> {
        self.name(parent.child(K::NAME)?)
    }

    fn opt_child_name(self, parent: Green<'_>) -> Option<Option<crate::Name>> {
        match parent.child(K::NAME) {
            Some(name) => Some(Some(self.name(name)?)),
            None => Some(None),
        }
    }

    /// The name inside a `NamedType`, `FragmentName`, `Variable` or similar wrapper node.
    fn wrapped_name(self, parent: Green<'_>, wrapper: K) -> Option<crate::Name> {
        self.child_name(parent.child(wrapper)?)
    }

    fn named_types(self, list: Option<Green<'_>>) -> Vec<ast::NamedType> {
        let Some(list) = list else {
            return Vec::new();
        };
        list.children_of(K::NAMED_TYPE)
            .filter_map(|named| self.child_name(named))
            .collect()
    }

    fn definition(self, node: Green<'_>) -> Option<ast::Definition> {
        use ast::Definition as A;
        macro_rules! r {
            ($convert: ident) => {
                self.with_location(node, self.$convert(node)?)
            };
        }
        Some(match node.kind() {
            K::OPERATION_DEFINITION => A::OperationDefinition(r!(operation_definition)),
            K::FRAGMENT_DEFINITION => A::FragmentDefinition(r!(fragment_definition)),
            K::DIRECTIVE_DEFINITION => A::DirectiveDefinition(r!(directive_definition)),
            K::SCHEMA_DEFINITION => A::SchemaDefinition(r!(schema_definition)),
            K::SCALAR_TYPE_DEFINITION => A::ScalarTypeDefinition(r!(scalar_type_definition)),
            K::OBJECT_TYPE_DEFINITION => A::ObjectTypeDefinition(r!(object_type_definition)),
            K::INTERFACE_TYPE_DEFINITION => {
                A::InterfaceTypeDefinition(r!(interface_type_definition))
            }
            K::UNION_TYPE_DEFINITION => A::UnionTypeDefinition(r!(union_type_definition)),
            K::ENUM_TYPE_DEFINITION => A::EnumTypeDefinition(r!(enum_type_definition)),
            K::INPUT_OBJECT_TYPE_DEFINITION => {
                A::InputObjectTypeDefinition(r!(input_object_type_definition))
            }
            K::SCHEMA_EXTENSION => A::SchemaExtension(r!(schema_extension)),
            K::DIRECTIVE_EXTENSION => A::DirectiveExtension(r!(directive_extension)),
            K::SCALAR_TYPE_EXTENSION => A::ScalarTypeExtension(r!(scalar_type_extension)),
            K::OBJECT_TYPE_EXTENSION => A::ObjectTypeExtension(r!(object_type_extension)),
            K::INTERFACE_TYPE_EXTENSION => A::InterfaceTypeExtension(r!(interface_type_extension)),
            K::UNION_TYPE_EXTENSION => A::UnionTypeExtension(r!(union_type_extension)),
            K::ENUM_TYPE_EXTENSION => A::EnumTypeExtension(r!(enum_type_extension)),
            K::INPUT_OBJECT_TYPE_EXTENSION => {
                A::InputObjectTypeExtension(r!(input_object_type_extension))
            }
            _ => return None,
        })
    }

    fn operation_definition(self, node: Green<'_>) -> Option<ast::OperationDefinition> {
        let operation_type = if let Some(ty) = node.child(K::OPERATION_TYPE) {
            self.operation_type(ty)?
        } else {
            ast::OperationType::Query
        };
        Some(ast::OperationDefinition {
            description: self.description(node)?,
            operation_type,
            name: self.opt_child_name(node)?,
            variables: self.collect(
                node.child(K::VARIABLES_DEFINITION),
                K::VARIABLE_DEFINITION,
                Self::variable_definition,
            ),
            directives: self.directives(node),
            selection_set: self.selection_set(node.child(K::SELECTION_SET)?),
        })
    }

    fn fragment_definition(self, node: Green<'_>) -> Option<ast::FragmentDefinition> {
        Some(ast::FragmentDefinition {
            description: self.description(node)?,
            name: self.wrapped_name(node, K::FRAGMENT_NAME)?,
            type_condition: self.type_condition(node.child(K::TYPE_CONDITION)?)?,
            directives: self.directives(node),
            selection_set: self.selection_set(node.child(K::SELECTION_SET)?),
        })
    }

    fn type_condition(self, node: Green<'_>) -> Option<ast::NamedType> {
        self.wrapped_name(node, K::NAMED_TYPE)
    }

    fn directive_definition(self, node: Green<'_>) -> Option<ast::DirectiveDefinition> {
        Some(ast::DirectiveDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            arguments: self.collect(
                node.child(K::ARGUMENTS_DEFINITION),
                K::INPUT_VALUE_DEFINITION,
                Self::input_value_definition,
            ),
            repeatable: node.has_token(S![repeatable]),
            locations: node
                .child(K::DIRECTIVE_LOCATIONS)
                .map(|locations| {
                    locations
                        .children_of(K::DIRECTIVE_LOCATION)
                        .filter_map(directive_location)
                        .collect()
                })
                .unwrap_or_default(),
            directives: self.directives(node),
        })
    }

    fn schema_definition(self, node: Green<'_>) -> Option<ast::SchemaDefinition> {
        Some(ast::SchemaDefinition {
            description: self.description(node)?,
            directives: self.directives(node),
            // This may represent a syntactically invalid thing: a schema without any root
            // operation definitions. However the presence of a broken schema definition does
            // affect whether a default schema definition should be inserted, so we bubble up the
            // potentially invalid definition.
            root_operations: self.root_operations(node),
        })
    }

    fn scalar_type_definition(self, node: Green<'_>) -> Option<ast::ScalarTypeDefinition> {
        Some(ast::ScalarTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            directives: self.directives(node),
        })
    }

    fn object_type_definition(self, node: Green<'_>) -> Option<ast::ObjectTypeDefinition> {
        Some(ast::ObjectTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            implements_interfaces: self.named_types(node.child(K::IMPLEMENTS_INTERFACES)),
            directives: self.directives(node),
            fields: self.field_definitions(node),
        })
    }

    fn interface_type_definition(self, node: Green<'_>) -> Option<ast::InterfaceTypeDefinition> {
        Some(ast::InterfaceTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            implements_interfaces: self.named_types(node.child(K::IMPLEMENTS_INTERFACES)),
            directives: self.directives(node),
            fields: self.field_definitions(node),
        })
    }

    fn union_type_definition(self, node: Green<'_>) -> Option<ast::UnionTypeDefinition> {
        Some(ast::UnionTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            directives: self.directives(node),
            members: self.named_types(node.child(K::UNION_MEMBER_TYPES)),
        })
    }

    fn enum_type_definition(self, node: Green<'_>) -> Option<ast::EnumTypeDefinition> {
        Some(ast::EnumTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            directives: self.directives(node),
            values: self.enum_value_definitions(node),
        })
    }

    fn input_object_type_definition(
        self,
        node: Green<'_>,
    ) -> Option<ast::InputObjectTypeDefinition> {
        Some(ast::InputObjectTypeDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            directives: self.directives(node),
            fields: self.input_field_definitions(node),
        })
    }

    fn schema_extension(self, node: Green<'_>) -> Option<ast::SchemaExtension> {
        Some(ast::SchemaExtension {
            directives: self.directives(node),
            root_operations: self.root_operations(node),
        })
    }

    fn directive_extension(self, node: Green<'_>) -> Option<ast::DirectiveExtension> {
        Some(ast::DirectiveExtension {
            name: self.child_name(node)?,
            directives: self.directives(node),
        })
    }

    fn scalar_type_extension(self, node: Green<'_>) -> Option<ast::ScalarTypeExtension> {
        Some(ast::ScalarTypeExtension {
            name: self.child_name(node)?,
            directives: self.directives(node),
        })
    }

    fn object_type_extension(self, node: Green<'_>) -> Option<ast::ObjectTypeExtension> {
        Some(ast::ObjectTypeExtension {
            name: self.child_name(node)?,
            implements_interfaces: self.named_types(node.child(K::IMPLEMENTS_INTERFACES)),
            directives: self.directives(node),
            fields: self.field_definitions(node),
        })
    }

    fn interface_type_extension(self, node: Green<'_>) -> Option<ast::InterfaceTypeExtension> {
        Some(ast::InterfaceTypeExtension {
            name: self.child_name(node)?,
            implements_interfaces: self.named_types(node.child(K::IMPLEMENTS_INTERFACES)),
            directives: self.directives(node),
            fields: self.field_definitions(node),
        })
    }

    fn union_type_extension(self, node: Green<'_>) -> Option<ast::UnionTypeExtension> {
        Some(ast::UnionTypeExtension {
            name: self.child_name(node)?,
            directives: self.directives(node),
            members: self.named_types(node.child(K::UNION_MEMBER_TYPES)),
        })
    }

    fn enum_type_extension(self, node: Green<'_>) -> Option<ast::EnumTypeExtension> {
        Some(ast::EnumTypeExtension {
            name: self.child_name(node)?,
            directives: self.directives(node),
            values: self.enum_value_definitions(node),
        })
    }

    fn input_object_type_extension(self, node: Green<'_>) -> Option<ast::InputObjectTypeExtension> {
        Some(ast::InputObjectTypeExtension {
            name: self.child_name(node)?,
            directives: self.directives(node),
            fields: self.input_field_definitions(node),
        })
    }

    fn field_definitions(self, parent: Green<'_>) -> Vec<Node<ast::FieldDefinition>> {
        self.collect(
            parent.child(K::FIELDS_DEFINITION),
            K::FIELD_DEFINITION,
            Self::field_definition,
        )
    }

    fn input_field_definitions(self, parent: Green<'_>) -> Vec<Node<ast::InputValueDefinition>> {
        self.collect(
            parent.child(K::INPUT_FIELDS_DEFINITION),
            K::INPUT_VALUE_DEFINITION,
            Self::input_value_definition,
        )
    }

    fn enum_value_definitions(self, parent: Green<'_>) -> Vec<Node<ast::EnumValueDefinition>> {
        self.collect(
            parent.child(K::ENUM_VALUES_DEFINITION),
            K::ENUM_VALUE_DEFINITION,
            Self::enum_value_definition,
        )
    }

    fn root_operations(self, parent: Green<'_>) -> Vec<Node<(ast::OperationType, ast::NamedType)>> {
        parent
            .children_of(K::ROOT_OPERATION_TYPE_DEFINITION)
            .filter_map(|node| {
                let ty = self.operation_type(node.child(K::OPERATION_TYPE)?)?;
                let name = self.wrapped_name(node, K::NAMED_TYPE)?;
                Some(self.with_location(node, (ty, name)))
            })
            .collect()
    }

    fn directive(self, node: Green<'_>) -> Option<ast::Directive> {
        Some(ast::Directive {
            name: self.child_name(node)?,
            arguments: self.arguments(node),
        })
    }

    fn arguments(self, parent: Green<'_>) -> Vec<Node<ast::Argument>> {
        self.collect(parent.child(K::ARGUMENTS), K::ARGUMENT, Self::argument)
    }

    fn operation_type(self, node: Green<'_>) -> Option<ast::OperationType> {
        match node.first_token_kind()? {
            S![query] => Some(ast::OperationType::Query),
            S![mutation] => Some(ast::OperationType::Mutation),
            S![subscription] => Some(ast::OperationType::Subscription),
            _ => None,
        }
    }

    fn variable_definition(self, node: Green<'_>) -> Option<ast::VariableDefinition> {
        let default_value = self.default_value(node)?;
        let ty = node.child_where(is_type)?;
        Some(ast::VariableDefinition {
            description: self.description(node)?,
            name: self.wrapped_name(node, K::VARIABLE)?,
            ty: self.with_location(ty, self.ty(ty)?),
            default_value,
            directives: self.directives(node),
        })
    }

    fn default_value(self, parent: Green<'_>) -> Option<Option<Node<ast::Value>>> {
        let Some(default) = parent.child(K::DEFAULT_VALUE) else {
            return Some(None);
        };
        let value = default.child_where(is_value)?;
        Some(Some(self.with_location(value, self.value(value)?)))
    }

    fn ty(self, node: Green<'_>) -> Option<ast::Type> {
        use ast::Type as A;
        match node.kind() {
            K::NAMED_TYPE => Some(A::Named(self.child_name(node)?)),
            K::LIST_TYPE => Some(A::List(Box::new(self.ty(node.child_where(is_type)?)?))),
            K::NON_NULL_TYPE => {
                if let Some(named) = node.child(K::NAMED_TYPE) {
                    Some(A::NonNullNamed(self.child_name(named)?))
                } else if let Some(list) = node.child(K::LIST_TYPE) {
                    Some(A::NonNullList(Box::new(
                        self.ty(list.child_where(is_type)?)?,
                    )))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn field_definition(self, node: Green<'_>) -> Option<ast::FieldDefinition> {
        Some(ast::FieldDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            arguments: self.collect(
                node.child(K::ARGUMENTS_DEFINITION),
                K::INPUT_VALUE_DEFINITION,
                Self::input_value_definition,
            ),
            ty: self.ty(node.child_where(is_type)?)?,
            directives: self.directives(node),
        })
    }

    fn argument(self, node: Green<'_>) -> Option<ast::Argument> {
        let name = self.child_name(node)?;
        let value = node.child_where(is_value)?;
        let value = self.with_location(value, self.value(value)?);
        Some(ast::Argument { name, value })
    }

    fn input_value_definition(self, node: Green<'_>) -> Option<ast::InputValueDefinition> {
        let default_value = self.default_value(node)?;
        let ty = node.child_where(is_type)?;
        Some(ast::InputValueDefinition {
            description: self.description(node)?,
            name: self.child_name(node)?,
            ty: self.with_location(ty, self.ty(ty)?),
            default_value,
            directives: self.directives(node),
        })
    }

    fn enum_value_definition(self, node: Green<'_>) -> Option<ast::EnumValueDefinition> {
        Some(ast::EnumValueDefinition {
            description: self.description(node)?,
            value: self.wrapped_name(node, K::ENUM_VALUE)?,
            directives: self.directives(node),
        })
    }

    fn selection_set(self, node: Green<'_>) -> Vec<ast::Selection> {
        node.children()
            .filter_map(|selection| self.selection(selection))
            .collect()
    }

    fn selection(self, node: Green<'_>) -> Option<ast::Selection> {
        use ast::Selection as A;
        Some(match node.kind() {
            K::FIELD => A::Field(self.with_location(node, self.field(node)?)),
            K::FRAGMENT_SPREAD => {
                A::FragmentSpread(self.with_location(node, self.fragment_spread(node)?))
            }
            K::INLINE_FRAGMENT => {
                A::InlineFragment(self.with_location(node, self.inline_fragment(node)?))
            }
            _ => return None,
        })
    }

    fn field(self, node: Green<'_>) -> Option<ast::Field> {
        let alias = match node.child(K::ALIAS) {
            Some(alias) => Some(self.child_name(alias)?),
            None => None,
        };
        Some(ast::Field {
            alias,
            name: self.child_name(node)?,
            arguments: self.arguments(node),
            directives: self.directives(node),
            selection_set: node
                .child(K::SELECTION_SET)
                .map(|set| self.selection_set(set)),
        })
    }

    fn fragment_spread(self, node: Green<'_>) -> Option<ast::FragmentSpread> {
        Some(ast::FragmentSpread {
            fragment_name: self.wrapped_name(node, K::FRAGMENT_NAME)?,
            directives: self.directives(node),
        })
    }

    fn inline_fragment(self, node: Green<'_>) -> Option<ast::InlineFragment> {
        let type_condition = match node.child(K::TYPE_CONDITION) {
            Some(condition) => Some(self.type_condition(condition)?),
            None => None,
        };
        Some(ast::InlineFragment {
            type_condition,
            directives: self.directives(node),
            selection_set: self.selection_set(node.child(K::SELECTION_SET)?),
        })
    }

    fn value(self, node: Green<'_>) -> Option<ast::Value> {
        use ast::Value as A;
        Some(match node.kind() {
            K::VARIABLE => A::Variable(self.child_name(node)?),
            K::STRING_VALUE => A::String(cst::StringValue::unescape(node.first_token()?.text())),
            K::FLOAT_VALUE => A::Float(ast::FloatValue::new_parsed(node.first_token()?.text())),
            K::INT_VALUE => A::Int(ast::IntValue::new_parsed(node.first_token()?.text())),
            K::BOOLEAN_VALUE => A::Boolean(node.first_token()?.text().parse().ok()?),
            K::NULL_VALUE => A::Null,
            K::ENUM_VALUE => A::Enum(self.child_name(node)?),
            K::LIST_VALUE => A::List(
                node.children()
                    .filter(|item| is_value(item.kind()))
                    .filter_map(|item| Some(self.with_location(item, self.value(item)?)))
                    .collect(),
            ),
            K::OBJECT_VALUE => A::Object(
                node.children_of(K::OBJECT_FIELD)
                    .filter_map(|field| {
                        let name = self.child_name(field)?;
                        let value = field.child_where(is_value)?;
                        Some((name, self.with_location(value, self.value(value)?)))
                    })
                    .collect(),
            ),
            _ => return None,
        })
    }
}

fn directive_location(node: Green<'_>) -> Option<ast::DirectiveLocation> {
    use ast::DirectiveLocation as L;
    Some(match node.first_token_kind()? {
        S![QUERY] => L::Query,
        S![MUTATION] => L::Mutation,
        S![SUBSCRIPTION] => L::Subscription,
        S![FIELD] => L::Field,
        S![FRAGMENT_DEFINITION] => L::FragmentDefinition,
        S![FRAGMENT_SPREAD] => L::FragmentSpread,
        S![INLINE_FRAGMENT] => L::InlineFragment,
        S![VARIABLE_DEFINITION] => L::VariableDefinition,
        S![SCHEMA] => L::Schema,
        S![SCALAR] => L::Scalar,
        S![OBJECT] => L::Object,
        S![FIELD_DEFINITION] => L::FieldDefinition,
        S![ARGUMENT_DEFINITION] => L::ArgumentDefinition,
        S![INTERFACE] => L::Interface,
        S![UNION] => L::Union,
        S![ENUM] => L::Enum,
        S![ENUM_VALUE] => L::EnumValue,
        S![INPUT_OBJECT] => L::InputObject,
        S![INPUT_FIELD_DEFINITION] => L::InputFieldDefinition,
        S![DIRECTIVE_DEFINITION] => L::DirectiveDefinition,
        _ => return None,
    })
}
