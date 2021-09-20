use crate::parser::grammar::{directive, name};
use crate::{Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/June2018/#ScalarTypeDefinition
///
/// ```txt
/// ScalarTypeDefinition
///     Description[opt] scalar Name Directives[Const][opt]
/// ```
pub(crate) fn scalar_type_definition(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::SCALAR_TYPE_DEFINITION);
    p.bump(SyntaxKind::scalar_KW);
    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }
}

/// See: https://spec.graphql.org/June2018/#ScalarTypeExtension
///
/// ```txt
/// ScalarTypeExtension
///     extend scalar Name Directives[const]
/// ```
pub(crate) fn scalar_type_extension(p: &mut Parser) {
    let _guard = p.start_node(SyntaxKind::SCALAR_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::scalar_KW);
    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    match p.peek() {
        Some(T![@]) => directive::directives(p),
        _ => p.err("expected Directives"),
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;

    #[test]
    fn it_parses_scalar_type_definition() {
        utils::check_ast(
            "
            scalar Time @deprecated
            ",
            r#"
            - DOCUMENT@0..21
                - SCALAR_TYPE_DEFINITION@0..21
                    - scalar_KW@0..6 "scalar"
                    - NAME@6..10
                        - IDENT@6..10 "Time"
                    - DIRECTIVES@10..21
                        - DIRECTIVE@10..21
                            - AT@10..11 "@"
                            - NAME@11..21
                                - IDENT@11..21 "deprecated"
            "#,
        )
    }

    #[test]
    fn it_errors_scalars_with_no_name() {
        utils::check_ast(
            "
            scalar @deprecated
            ",
            r#"
            - DOCUMENT@0..17
                - SCALAR_TYPE_DEFINITION@0..17
                    - scalar_KW@0..6 "scalar"
                    - DIRECTIVES@6..17
                        - DIRECTIVE@6..17
                            - AT@6..7 "@"
                            - NAME@7..17
                                - IDENT@7..17 "deprecated"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "
            extend scalar Time @deprecated
            ",
            r#"
            - DOCUMENT@0..27
                - SCALAR_TYPE_EXTENSION@0..27
                    - extend_KW@0..6 "extend"
                    - scalar_KW@6..12 "scalar"
                    - NAME@12..16
                        - IDENT@12..16 "Time"
                    - DIRECTIVES@16..27
                        - DIRECTIVE@16..27
                            - AT@16..17 "@"
                            - NAME@17..27
                                - IDENT@17..27 "deprecated"
            "#,
        )
    }

    #[test]
    fn it_errors_extension_with_no_name() {
        utils::check_ast(
            "
            extend scalar @deprecated
            ",
            r#"
            - DOCUMENT@0..23
                - SCALAR_TYPE_EXTENSION@0..23
                    - extend_KW@0..6 "extend"
                    - scalar_KW@6..12 "scalar"
                    - DIRECTIVES@12..23
                        - DIRECTIVE@12..23
                            - AT@12..13 "@"
                            - NAME@13..23
                                - IDENT@13..23 "deprecated"
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }
}
