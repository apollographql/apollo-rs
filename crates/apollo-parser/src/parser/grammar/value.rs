use crate::parser::grammar::{name, variable};
use crate::{create_err, Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#Value
///
/// ```txt
/// Value [Const]
///     [~Const] Variable
///     IntValue
///     FloatValue
///     StringValue
///     BooleanValue
///     NullValue
///     EnumValue
///     ListValue [Const]
///     ObjectValue [Const]
/// ```
pub(crate) fn value(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::VALUE);
    match p.peek() {
        Some(T![$]) => variable::variable(p),
        Some(TokenKind::Int) => p.bump(SyntaxKind::INT_VALUE),
        Some(TokenKind::Float) => p.bump(SyntaxKind::FLOAT_VALUE),
        Some(TokenKind::StringValue) => p.bump(SyntaxKind::STRING_VALUE),
        Some(TokenKind::Boolean) => p.bump(SyntaxKind::BOOLEAN_VALUE),
        Some(TokenKind::Null) => p.bump(SyntaxKind::NULL_VALUE),
        Some(TokenKind::Name) => enum_value(p),
        Some(T!['[']) => list_value(p),
        Some(T!['{']) => object_value(p),
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected a valid Value, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    }
}
/// See: https://spec.graphql.org/June2018/#EnumValue
/// ```txt
/// EnumValue
/// Name but not true or false or null
/// ```
pub(crate) fn enum_value(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::ENUM_VALUE);
    let name = p.peek_data().unwrap();

    if matches!(name.as_str(), "true" | "false" | "null") {
        p.push_err(create_err!(
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Enum Value cannot be {}",
            p.peek_data()
                .unwrap_or_else(|| String::from("no further data"))
        ));
    }
    name::name(p);
}

/// See: https://spec.graphql.org/June2018/#ListValue
/// ```txt
/// ListValue[Const]
///     [ ]
///     [ Value [?const][list] ]
/// ```
pub(crate) fn list_value(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::LIST_VALUE);
    p.bump(S!['[']);
    while let Some(node) = p.peek() {
        if node == T![']'] {
            p.bump(S![']']);
            guard.finish_node();
            break;
        } else if node == T![,] {
            p.bump(S![,]);
            value(p);
        } else {
            value(p);
        }
    }
}

/// See: https://spec.graphql.org/June2018/#ObjectValue
///
/// ```txt
/// ObjectValue [Const]
///     { }
///     { ObjectField [Const][list] }
pub(crate) fn object_value(p: &mut Parser) {
    let guard = p.start_node(SyntaxKind::OBJECT_VALUE);
    p.bump(S!['{']);
    match p.peek() {
        Some(TokenKind::Name) => {
            object_field(p);
            if let Some(T!['}']) = p.peek() {
                p.bump(S!['}']);
                guard.finish_node()
            } else {
                p.push_err(create_err!(
                    p.peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected a closing }} to follow an Object Value , got {}",
                    p.peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
        Some(T!['}']) => {
            p.bump(S!['}']);
            guard.finish_node()
        }
        _ => {
            p.push_err(create_err!(
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected an Object Value, got {}",
                p.peek_data()
                    .unwrap_or_else(|| String::from("no further data"))
            ));
        }
    }
}

/// See: https://spec.graphql.org/June2018/#ObjectField
///
/// ```txt
/// ObjectField [Const]
///     Name : Value [const]
/// ```
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
    if let Some(T![,]) = p.peek() {
        p.bump(S![,]);
        return object_field(p);
    }
}

pub(crate) fn default_value(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::DEFAULT_VALUE);
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
            - DOCUMENT@0..91
                - OPERATION_DEFINITION@0..91
                    - SELECTION_SET@0..91
                        - L_CURLY@0..1 "{"
                        - SELECTION@1..90
                            - FIELD@1..90
                                - NAME@1..5
                                    - IDENT@1..5 "user"
                                - ARGUMENTS@5..90
                                    - L_PAREN@5..6 "("
                                    - ARGUMENT@6..10
                                        - NAME@6..8
                                            - IDENT@6..8 "id"
                                        - COLON@8..9 ":"
                                        - VALUE@9..10
                                            - INT_VALUE@9..10 "4"
                                    - COMMA@10..11 ","
                                    - ARGUMENT@11..21
                                        - NAME@11..15
                                            - IDENT@11..15 "size"
                                        - COLON@15..16 ":"
                                        - VALUE@16..21
                                            - VARIABLE@16..21
                                                - DOLLAR@16..17 "$"
                                                - NAME@17..21
                                                    - IDENT@17..21 "size"
                                    - ARGUMENT@21..35
                                        - NAME@21..26
                                            - IDENT@21..26 "value"
                                        - COLON@26..27 ":"
                                        - VALUE@27..35
                                            - STRING_VALUE@27..35 "\"string\""
                                    - COMMA@35..36 ","
                                    - ARGUMENT@36..54
                                        - NAME@36..41
                                            - IDENT@36..41 "input"
                                        - COLON@41..42 ":"
                                        - VALUE@42..54
                                            - LIST_VALUE@42..54
                                                - L_BRACK@42..43 "["
                                                - VALUE@43..48
                                                    - STRING_VALUE@43..48 "\"one\""
                                                - COMMA@48..49 ","
                                                - VALUE@49..53
                                                    - FLOAT_VALUE@49..53 "1.34"
                                                - R_BRACK@53..54 "]"
                                    - COMMA@54..55 ","
                                    - ARGUMENT@55..89
                                        - NAME@55..65
                                            - IDENT@55..65 "otherInput"
                                        - COLON@65..66 ":"
                                        - VALUE@66..89
                                            - OBJECT_VALUE@66..89
                                                - L_CURLY@66..67 "{"
                                                - OBJECT_FIELD@67..76
                                                    - NAME@67..70
                                                        - IDENT@67..70 "key"
                                                    - COLON@70..71 ":"
                                                    - VALUE@71..76
                                                        - BOOLEAN_VALUE@71..76 "false"
                                                - COMMA@76..77 ","
                                                - OBJECT_FIELD@77..88
                                                    - NAME@77..83
                                                        - IDENT@77..83 "output"
                                                    - COLON@83..84 ":"
                                                    - VALUE@84..88
                                                        - NULL_VALUE@84..88 "null"
                                                - R_CURLY@88..89 "}"
                                    - R_PAREN@89..90 ")"
                        - R_CURLY@90..91 "}"
            "#,
        );
    }
}
