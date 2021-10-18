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
    let _g = p.start_node(SyntaxKind::VALUE);
    match p.peek() {
        Some(T![$]) => variable::variable(p),
        Some(TokenKind::Int) => p.bump(SyntaxKind::INT_VALUE),
        Some(TokenKind::Float) => p.bump(SyntaxKind::FLOAT_VALUE),
        Some(TokenKind::StringValue) => p.bump(SyntaxKind::STRING_VALUE),
        Some(TokenKind::Name) => {
            let node = p.peek_data().unwrap();
            if matches!(node.as_str(), "true" | "false") {
                p.bump(SyntaxKind::BOOLEAN_VALUE);
            } else if matches!(node.as_str(), "null") {
                p.bump(SyntaxKind::NULL_VALUE);
            } else {
                enum_value(p);
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

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_returns_values() {
        utils::check_ast(
            r#"
            {
              user(id: 4, size: $size value: "string", input: [ "one", 1.34 ], otherInput: { key: false, output: null })
            }"#,
            r#"
            - DOCUMENT@0..149
                - WHITESPACE@0..13 "\n            "
                - OPERATION_DEFINITION@13..149
                    - SELECTION_SET@13..149
                        - L_CURLY@13..14 "{"
                        - WHITESPACE@14..29 "\n              "
                        - SELECTION@29..148
                            - FIELD@29..148
                                - NAME@29..33
                                    - IDENT@29..33 "user"
                                - ARGUMENTS@33..148
                                    - L_PAREN@33..34 "("
                                    - ARGUMENT@34..41
                                        - NAME@34..36
                                            - IDENT@34..36 "id"
                                        - COLON@36..37 ":"
                                        - WHITESPACE@37..38 " "
                                        - VALUE@38..41
                                            - INT_VALUE@38..39 "4"
                                            - COMMA@39..40 ","
                                            - WHITESPACE@40..41 " "
                                    - ARGUMENT@41..53
                                        - NAME@41..45
                                            - IDENT@41..45 "size"
                                        - COLON@45..46 ":"
                                        - WHITESPACE@46..47 " "
                                        - VALUE@47..53
                                            - VARIABLE@47..53
                                                - DOLLAR@47..48 "$"
                                                - NAME@48..53
                                                    - IDENT@48..52 "size"
                                                    - WHITESPACE@52..53 " "
                                    - ARGUMENT@53..70
                                        - NAME@53..58
                                            - IDENT@53..58 "value"
                                        - COLON@58..59 ":"
                                        - WHITESPACE@59..60 " "
                                        - VALUE@60..70
                                            - STRING_VALUE@60..68 "\"string\""
                                            - COMMA@68..69 ","
                                            - WHITESPACE@69..70 " "
                                    - ARGUMENT@70..94
                                        - NAME@70..75
                                            - IDENT@70..75 "input"
                                        - COLON@75..76 ":"
                                        - WHITESPACE@76..77 " "
                                        - VALUE@77..94
                                            - LIST_VALUE@77..94
                                                - L_BRACK@77..78 "["
                                                - WHITESPACE@78..79 " "
                                                - VALUE@79..86
                                                    - STRING_VALUE@79..84 "\"one\""
                                                    - COMMA@84..85 ","
                                                    - WHITESPACE@85..86 " "
                                                - VALUE@86..91
                                                    - FLOAT_VALUE@86..90 "1.34"
                                                    - WHITESPACE@90..91 " "
                                                - R_BRACK@91..92 "]"
                                                - COMMA@92..93 ","
                                                - WHITESPACE@93..94 " "
                                    - ARGUMENT@94..134
                                        - NAME@94..104
                                            - IDENT@94..104 "otherInput"
                                        - COLON@104..105 ":"
                                        - WHITESPACE@105..106 " "
                                        - VALUE@106..134
                                            - OBJECT_VALUE@106..134
                                                - L_CURLY@106..107 "{"
                                                - WHITESPACE@107..108 " "
                                                - OBJECT_FIELD@108..120
                                                    - NAME@108..111
                                                        - IDENT@108..111 "key"
                                                    - COLON@111..112 ":"
                                                    - WHITESPACE@112..113 " "
                                                    - VALUE@113..120
                                                        - BOOLEAN_VALUE@113..118 "false"
                                                        - COMMA@118..119 ","
                                                        - WHITESPACE@119..120 " "
                                                - OBJECT_FIELD@120..133
                                                    - NAME@120..126
                                                        - IDENT@120..126 "output"
                                                    - COLON@126..127 ":"
                                                    - WHITESPACE@127..128 " "
                                                    - VALUE@128..133
                                                        - NULL_VALUE@128..132 "null"
                                                        - WHITESPACE@132..133 " "
                                                - R_CURLY@133..134 "}"
                                    - R_PAREN@134..135 ")"
                                    - WHITESPACE@135..148 "\n            "
                        - R_CURLY@148..149 "}"
            "#,
        );
    }
}
