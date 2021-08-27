use crate::parser::{directive, name, ty};
use crate::{create_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#UnionTypeDefinition
///
/// ```txt
/// UnionTypeDefinition
///     Description[opt] union Name Directives[Const][opt] UnionMemberTypes[opt]
/// ```
pub(crate) fn union_type_definition(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::UNION_TYPE_DEFINITION);
    parser.bump(SyntaxKind::union_KW);

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Union Type Definition to have a Name, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser);
    }

    if let Some(TokenKind::Eq) = parser.peek() {
        union_member_types(parser, false);
    }
}

/// See: https://spec.graphql.org/June2018/#UnionMemberTypes
///
/// ```txt
/// UnionMemberTypes
///     = |[opt] NamedType
///     UnionMemberTypes | NamedType
/// ```
pub(crate) fn union_member_types(parser: &mut Parser, is_union: bool) {
    let _guard = parser.start_node(SyntaxKind::UNION_MEMBER_TYPES);
    parser.bump(SyntaxKind::EQ);

    match parser.peek() {
        Some(TokenKind::Pipe) => {
            parser.bump(SyntaxKind::PIPE);
            union_member_types(parser, is_union);
        }
        Some(TokenKind::Node) => {
            ty::named_type(parser);
            if parser.peek_data().is_some() {
                union_member_types(parser, true)
            }
        }
        _ => {
            if !is_union {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected to have Union Member Types in a Union Type Definition, got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                ));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_union_type_definition() {
        utils::check_ast(
            "union SearchResult = Photo | Person",
            r#"
            - DOCUMENT@0..30
                - UNION_TYPE_DEFINITION@0..30
                    - union_KW@0..5 "union"
                    - NAME@5..17
                        - IDENT@5..17 "SearchResult"
                    - UNION_MEMBER_TYPES@17..30
                        - EQ@17..18 "="
                        - NAMED_TYPE@18..23
                            - NAME@18..23
                                - IDENT@18..23 "Photo"
                        - UNION_MEMBER_TYPES@23..30
                            - EQ@23..24 "|"
                            - NAMED_TYPE@24..30
                                - NAME@24..30
                                    - IDENT@24..30 "Person"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing() {
        utils::check_ast(
            "union = Photo | Person",
            r#"
            - DOCUMENT@0..18
                - UNION_TYPE_DEFINITION@0..18
                    - union_KW@0..5 "union"
                    - UNION_MEMBER_TYPES@5..18
                        - EQ@5..6 "="
                        - NAMED_TYPE@6..11
                            - NAME@6..11
                                - IDENT@6..11 "Photo"
                        - UNION_MEMBER_TYPES@11..18
                            - EQ@11..12 "|"
                            - NAMED_TYPE@12..18
                                - NAME@12..18
                                    - IDENT@12..18 "Person"
            - ERROR@0:1 "Expected Union Type Definition to have a Name, got ="
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_union_definition_is_missing() {
        utils::check_ast(
            "union = ",
            r#"
            - DOCUMENT@0..6
                - UNION_TYPE_DEFINITION@0..6
                    - union_KW@0..5 "union"
                    - UNION_MEMBER_TYPES@5..6
                        - EQ@5..6 "="
            - ERROR@0:1 "Expected Union Type Definition to have a Name, got ="
            - ERROR@0:15 "Expected to have Union Member Types in a Union Type Definition, got no further data"
            "#,
        )
    }
}
