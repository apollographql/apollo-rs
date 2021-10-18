use crate::{
    parser::grammar::{description, directive, document::is_definition, name, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#UnionTypeDefinition
///
/// *UnionTypeDefinition*:
///     Description<sub>opt</sub> **union** Name Directives<sub>\[Const\] opt</sub> UnionDefMemberTypes<sub>opt</sub>
pub(crate) fn union_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("union") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::union_KW);
    }

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

/// See: https://spec.graphql.org/draft/#UnionTypeExtension
///
/// *UnionTypeExtension*:
///     **extend** **union** Name Directives<sub>\[Const\] opt</sub> UnionDefMemberTypes
///     **extend** **union** Name Directives<sub>\[Const\]</sub>
pub(crate) fn union_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_TYPE_EXTENSION);
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

/// See: https://spec.graphql.org/draft/#UnionMemberTypes
///
/// *UnionMemberTypes*:
///     **=** **|**<sub>opt</sub> NamedType
///     UnionMemberTypes **|** NamedType
pub(crate) fn union_member_types(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_MEMBER_TYPES);
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
            if let Some(node) = p.peek_data() {
                if !is_definition(node) {
                    union_member_type(p, true);
                }

                return;
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
    use super::*;
    use crate::ast;
    use crate::parser::utils;

    #[test]
    fn union_member_types() {
        let input = "union SearchResult = Photo | Person | Cat | Dog";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert!(ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let ast::Definition::UnionTypeDefinition(union_type) = def {
                assert_eq!(union_type.name().unwrap().text(), "SearchResult");
                for union_member in union_type.union_member_types().unwrap().named_types() {
                    println!("{}", union_member.name().unwrap().text()); // Photo Person Cat Dog
                }
            }
        }
    }

    #[test]
    fn it_parses_union_type_definition() {
        utils::check_ast(
            "union SearchResult = Photo | Person",
            r#"
            - DOCUMENT@0..35
                - UNION_TYPE_DEFINITION@0..35
                    - union_KW@0..5 "union"
                    - WHITESPACE@5..6 " "
                    - NAME@6..19
                        - IDENT@6..18 "SearchResult"
                        - WHITESPACE@18..19 " "
                    - UNION_MEMBER_TYPES@19..35
                        - EQ@19..20 "="
                        - WHITESPACE@20..21 " "
                        - NAMED_TYPE@21..27
                            - NAME@21..27
                                - IDENT@21..26 "Photo"
                                - WHITESPACE@26..27 " "
                        - PIPE@27..28 "|"
                        - WHITESPACE@28..29 " "
                        - NAMED_TYPE@29..35
                            - NAME@29..35
                                - IDENT@29..35 "Person"
            "#,
        )
    }

    #[test]
    fn it_parses_union_type_definition_followed_by_object_definition() {
        utils::check_ast(
            "union SearchResult = Photo | Person
type Error {
    code: Int
}",
            r#"
            - DOCUMENT@0..64
                - UNION_TYPE_DEFINITION@0..36
                    - union_KW@0..5 "union"
                    - WHITESPACE@5..6 " "
                    - NAME@6..19
                        - IDENT@6..18 "SearchResult"
                        - WHITESPACE@18..19 " "
                    - UNION_MEMBER_TYPES@19..36
                        - EQ@19..20 "="
                        - WHITESPACE@20..21 " "
                        - NAMED_TYPE@21..27
                            - NAME@21..27
                                - IDENT@21..26 "Photo"
                                - WHITESPACE@26..27 " "
                        - PIPE@27..28 "|"
                        - WHITESPACE@28..29 " "
                        - NAMED_TYPE@29..36
                            - NAME@29..36
                                - IDENT@29..35 "Person"
                                - WHITESPACE@35..36 "\n"
                - OBJECT_TYPE_DEFINITION@36..64
                    - type_KW@36..40 "type"
                    - WHITESPACE@40..41 " "
                    - NAME@41..47
                        - IDENT@41..46 "Error"
                        - WHITESPACE@46..47 " "
                    - FIELDS_DEFINITION@47..64
                        - L_CURLY@47..48 "{"
                        - WHITESPACE@48..53 "\n    "
                        - FIELD_DEFINITION@53..63
                            - NAME@53..57
                                - IDENT@53..57 "code"
                            - COLON@57..58 ":"
                            - WHITESPACE@58..59 " "
                            - TYPE@59..63
                                - WHITESPACE@59..60 "\n"
                                - NAMED_TYPE@60..63
                                    - NAME@60..63
                                        - IDENT@60..63 "Int"
                        - R_CURLY@63..64 "}"
            "#,
        );
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_definition() {
        utils::check_ast(
            "union = Photo | Person",
            r#"
            - DOCUMENT@0..22
                - UNION_TYPE_DEFINITION@0..22
                    - union_KW@0..5 "union"
                    - WHITESPACE@5..6 " "
                    - UNION_MEMBER_TYPES@6..22
                        - EQ@6..7 "="
                        - WHITESPACE@7..8 " "
                        - NAMED_TYPE@8..14
                            - NAME@8..14
                                - IDENT@8..13 "Photo"
                                - WHITESPACE@13..14 " "
                        - PIPE@14..15 "|"
                        - WHITESPACE@15..16 " "
                        - NAMED_TYPE@16..22
                            - NAME@16..22
                                - IDENT@16..22 "Person"
            - ERROR@6:7 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_union_definition_is_missing_in_definition() {
        utils::check_ast(
            "union = ",
            r#"
            - DOCUMENT@0..8
                - UNION_TYPE_DEFINITION@0..8
                    - union_KW@0..5 "union"
                    - WHITESPACE@5..6 " "
                    - UNION_MEMBER_TYPES@6..8
                        - EQ@6..7 "="
                        - WHITESPACE@7..8 " "
            - ERROR@6:7 "expected a Name"
            - ERROR@8:8 "expected Union Member Types"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "extend union SearchResult @deprecated = Photo | Person",
            r#"
            - DOCUMENT@0..54
                - UNION_TYPE_EXTENSION@0..54
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - union_KW@7..12 "union"
                    - WHITESPACE@12..13 " "
                    - NAME@13..26
                        - IDENT@13..25 "SearchResult"
                        - WHITESPACE@25..26 " "
                    - DIRECTIVES@26..38
                        - DIRECTIVE@26..38
                            - AT@26..27 "@"
                            - NAME@27..38
                                - IDENT@27..37 "deprecated"
                                - WHITESPACE@37..38 " "
                    - UNION_MEMBER_TYPES@38..54
                        - EQ@38..39 "="
                        - WHITESPACE@39..40 " "
                        - NAMED_TYPE@40..46
                            - NAME@40..46
                                - IDENT@40..45 "Photo"
                                - WHITESPACE@45..46 " "
                        - PIPE@46..47 "|"
                        - WHITESPACE@47..48 " "
                        - NAMED_TYPE@48..54
                            - NAME@48..54
                                - IDENT@48..54 "Person"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_name_is_missing_in_extension() {
        utils::check_ast(
            "extend union = Photo | Person",
            r#"
            - DOCUMENT@0..29
                - UNION_TYPE_EXTENSION@0..29
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - union_KW@7..12 "union"
                    - WHITESPACE@12..13 " "
                    - UNION_MEMBER_TYPES@13..29
                        - EQ@13..14 "="
                        - WHITESPACE@14..15 " "
                        - NAMED_TYPE@15..21
                            - NAME@15..21
                                - IDENT@15..20 "Photo"
                                - WHITESPACE@20..21 " "
                        - PIPE@21..22 "|"
                        - WHITESPACE@22..23 " "
                        - NAMED_TYPE@23..29
                            - NAME@23..29
                                - IDENT@23..29 "Person"
            - ERROR@13:14 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_creates_an_error_when_requirements_are_missing_in_extension() {
        utils::check_ast(
            "extend union SearchResult",
            r#"
            - DOCUMENT@0..25
                - UNION_TYPE_EXTENSION@0..25
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - union_KW@7..12 "union"
                    - WHITESPACE@12..13 " "
                    - NAME@13..25
                        - IDENT@13..25 "SearchResult"
            - ERROR@25:25 "expected Directives or Union Member Types"
            "#,
        )
    }
}
