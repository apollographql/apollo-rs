use crate::{
    parser::grammar::{name, variable},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#Value
///
/// *Value*<sub>\[Const\]</sub>
///     <sub>\[if not Const\]</sub> Variable
///     IntValue
///     FloatValue
///     StringValue
///     BooleanValue
///     NullValue
///     EnumValue
///     ListValue<sub>\[?Const\]</sub>
///     ObjectValue<sub>\[?Const\]</sub>
pub(crate) fn value(p: &mut Parser) {
    match p.peek() {
        Some(T![$]) => variable::variable(p),
        Some(TokenKind::Int) => {
            let _g = p.start_node(SyntaxKind::INT_VALUE);
            p.bump(SyntaxKind::INT);
        }
        Some(TokenKind::Float) => {
            let _g = p.start_node(SyntaxKind::FLOAT_VALUE);
            p.bump(SyntaxKind::FLOAT);
        }
        Some(TokenKind::StringValue) => {
            let _g = p.start_node(SyntaxKind::STRING_VALUE);
            p.bump(SyntaxKind::STRING);
        }
        Some(TokenKind::Name) => {
            let node = p.peek_data().unwrap();
            match node.as_str() {
                "true" => {
                    let _g = p.start_node(SyntaxKind::BOOLEAN_VALUE);
                    p.bump(SyntaxKind::true_KW);
                }
                "false" => {
                    let _g = p.start_node(SyntaxKind::BOOLEAN_VALUE);
                    p.bump(SyntaxKind::false_KW);
                }
                "null" => {
                    let _g = p.start_node(SyntaxKind::NULL_VALUE);
                    p.bump(SyntaxKind::null_KW)
                }
                _ => enum_value(p),
            }
        }
        Some(T!['[']) => list_value(p),
        Some(T!['{']) => object_value(p),
        _ => p.err("expected a valid Value"),
    }
}
/// See: https://spec.graphql.org/draft/#EnumValue
///
/// *EnumValue*:
///     Name *but not* **true** *or* **false** *or* **null**
pub(crate) fn enum_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::ENUM_VALUE);
    let name = p.peek_data().unwrap();

    if matches!(name.as_str(), "true" | "false" | "null") {
        p.err("unexpected Enum Value");
    }

    name::name(p);
}

/// See: https://spec.graphql.org/draft/#ListValue
///
/// *ListValue*<sub>\[Const\]</sub>:
///     **[** **]**
///     **[** Value<sub>\[?Const\] list</sub> **]**
pub(crate) fn list_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::LIST_VALUE);
    p.bump(S!['[']);

    while let Some(node) = p.peek() {
        if node == T![']'] {
            p.bump(S![']']);
            break;
        } else {
            value(p);
        }
    }
}

/// See: https://spec.graphql.org/draft/#ObjectValue
///
/// *ObjectValue*<sub>\[Const\]</sub>:
///     **{** **}**
///     **{** ObjectField<sub>\[?Const\] list</sub> **}**
pub(crate) fn object_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_VALUE);
    p.bump(S!['{']);

    match p.peek() {
        Some(TokenKind::Name) => {
            object_field(p);
            if let Some(T!['}']) = p.peek() {
                p.bump(S!['}']);
            } else {
                p.err("expected }");
            }
        }
        Some(T!['}']) => {
            p.bump(S!['}']);
        }
        _ => p.err("expected Object Value"),
    }
}

/// See: https://spec.graphql.org/draft/#ObjectField
///
/// *ObjectField*<sub>\[Const\]</sub>:
///     Name **:** Value<sub>\[?Const\]</sub>
pub(crate) fn object_field(p: &mut Parser) {
    if let Some(TokenKind::Name) = p.peek() {
        let guard = p.start_node(SyntaxKind::OBJECT_FIELD);
        name::name(p);

        if let Some(T![:]) = p.peek() {
            p.bump(S![:]);
            value(p);
            if p.peek().is_some() {
                guard.finish_node();
                return object_field(p);
            }
        }
    }
}

/// See: https://spec.graphql.org/draft/#DefaultValue
///
/// *DefaultValue*:
///     **=** Value<sub>\[Const\]</sub>
pub(crate) fn default_value(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DEFAULT_VALUE);
    p.bump(S![=]);
    value(p);
}
