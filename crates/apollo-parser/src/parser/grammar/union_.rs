use crate::parser::grammar::{directive, name, ty};
use crate::{Parser, SyntaxKind, TokenKind, S, T};

/// See: https://spec.graphql.org/June2018/#UnionTypeDefinition
///
/// ```txt
/// UnionTypeDefinition
///     Description[opt] union Name Directives[Const][opt] UnionMemberTypes[opt]
/// ```
pub(crate) fn union_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::UNION_TYPE_DEFINITION);
    p.bump(SyntaxKind::union_KW);

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T![=]) = p.peek() {
        union_member_types(p);
    }
}

/// See: https://spec.graphql.org/June2018/#UnionTypeExtension
///
/// ```txt
/// UnionTypeExtension
///     extend union Name Directives[Const][opt] UnionMemberTypes
///     extend union Name Directives[Const]
/// ```
pub(crate) fn union_type_extension(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::UNION_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::union_KW);

    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T![=]) = p.peek() {
        meets_requirements = true;
        union_member_types(p);
    }

    if !meets_requirements {
        p.err("expected Directives or Union Member Types");
    }
}

/// See: https://spec.graphql.org/June2018/#UnionMemberTypes
///
/// ```txt
/// UnionMemberTypes
///     = |[opt] NamedType
///     UnionMemberTypes | NamedType
/// ```
pub(crate) fn union_member_types(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::UNION_MEMBER_TYPES);
    p.bump(S![=]);

    union_member_type(p, false);
}

fn union_member_type(p: &mut Parser, is_union: bool) {
    match p.peek() {
        Some(T![|]) => {
            p.bump(S![|]);
            union_member_type(p, is_union);
        }
        Some(TokenKind::Name) => {
            ty::named_type(p);
            if p.peek_data().is_some() {
                union_member_type(p, true)
            }
        }
        _ => {
            if !is_union {
                p.err("expected Union Member Types");
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
                        - PIPE@23..24 "|"
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
                        - PIPE@11..12 "|"
                        - NAMED_TYPE@12..18
                            - NAME@12..18
                                - IDENT@12..18 "Person"
            - ERROR@0:1 "expected a Name"
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
            - ERROR@0:1 "expected a Name"
            - ERROR@0:3 "expected Union Member Types"
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
                        - PIPE@40..41 "|"
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
                        - PIPE@17..18 "|"
                        - NAMED_TYPE@18..24
                            - NAME@18..24
                                - IDENT@18..24 "Person"
            - ERROR@0:1 "expected a Name"
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
            - ERROR@0:3 "expected Directives or Union Member Types"
            "#,
        )
    }
}
