use crate::parser::grammar::{directive, name, ty};
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

/// See: https://spec.graphql.org/June2018/#UnionTypeExtension
///
/// ```txt
/// UnionTypeExtension
///     extend union Name Directives[Const][opt] UnionMemberTypes
///     extend union Name Directives[Const]
/// ```
pub(crate) fn union_type_extension(parser: &mut Parser) {
    let _guard = parser.start_node(SyntaxKind::UNION_TYPE_EXTENSION);
    parser.bump(SyntaxKind::extend_KW);
    parser.bump(SyntaxKind::union_KW);

    let mut meets_requirements = false;

    match parser.peek() {
        Some(TokenKind::Node) => name::name(parser),
        _ => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expected Union Type Extension to have a Name, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }

    if let Some(TokenKind::At) = parser.peek() {
        meets_requirements = true;
        directive::directives(parser);
    }

    if let Some(TokenKind::Eq) = parser.peek() {
        meets_requirements = true;
        union_member_types(parser, false);
    }

    if !meets_requirements {
        parser.push_err(create_err!(
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
            "Expected Union Type Extension to have Directives or Union Member Types, got {}",
            parser
                .peek_data()
                .unwrap_or_else(|| String::from("no further data")),
        ));
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
    fn it_creates_an_error_when_name_is_missing_in_definition() {
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
    fn it_creates_an_error_when_union_definition_is_missing_in_definition() {
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

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "extend union SearchResult @deprecated = Photo | Person",
            r#"
            - DOCUMENT@0..47
                - UNION_TYPE_EXTENSION@0..47
                    - extend_KW@0..6 "extend"
                    - union_KW@6..11 "union"
                    - NAME@11..23
                        - IDENT@11..23 "SearchResult"
                    - DIRECTIVES@23..34
                        - DIRECTIVE@23..34
                            - AT@23..24 "@"
                            - NAME@24..34
                                - IDENT@24..34 "deprecated"
                    - UNION_MEMBER_TYPES@34..47
                        - EQ@34..35 "="
                        - NAMED_TYPE@35..40
                            - NAME@35..40
                                - IDENT@35..40 "Photo"
                        - UNION_MEMBER_TYPES@40..47
                            - EQ@40..41 "|"
                            - NAMED_TYPE@41..47
                                - NAME@41..47
                                    - IDENT@41..47 "Person"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_extension() {
        utils::check_ast(
            "extend union = Photo | Person",
            r#"
            - DOCUMENT@0..24
                - UNION_TYPE_EXTENSION@0..24
                    - extend_KW@0..6 "extend"
                    - union_KW@6..11 "union"
                    - UNION_MEMBER_TYPES@11..24
                        - EQ@11..12 "="
                        - NAMED_TYPE@12..17
                            - NAME@12..17
                                - IDENT@12..17 "Photo"
                        - UNION_MEMBER_TYPES@17..24
                            - EQ@17..18 "|"
                            - NAMED_TYPE@18..24
                                - NAME@18..24
                                    - IDENT@18..24 "Person"
            - ERROR@0:1 "Expected Union Type Extension to have a Name, got ="
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_requirements_are_missing_in_extension() {
        utils::check_ast(
            "extend union SearchResult",
            r#"
            - DOCUMENT@0..23
                - UNION_TYPE_EXTENSION@0..23
                    - extend_KW@0..6 "extend"
                    - union_KW@6..11 "union"
                    - NAME@11..23
                        - IDENT@11..23 "SearchResult"
            - ERROR@0:15 "Expected Union Type Extension to have Directives or Union Member Types, got no further data"
            "#,
        )
    }
}
