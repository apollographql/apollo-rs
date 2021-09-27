use crate::parser::grammar::{description, directive, name};
use crate::{Parser, SyntaxKind, TokenKind, T};

/// See: https://spec.graphql.org/June2018/#ScalarTypeDefinition
///
/// ```txt
/// ScalarTypeDefinition
///     Description[opt] scalar Name Directives[Const][opt]
/// ```
pub(crate) fn scalar_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::SCALAR_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("scalar") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::scalar_KW);
    }

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
    let _g = p.start_node(SyntaxKind::SCALAR_TYPE_EXTENSION);
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
            - DOCUMENT@0..49
                - WHITESPACE@0..13 "\n            "
                - SCALAR_TYPE_DEFINITION@13..49
                    - scalar_KW@13..19 "scalar"
                    - WHITESPACE@19..20 " "
                    - NAME@20..25
                        - IDENT@20..24 "Time"
                        - WHITESPACE@24..25 " "
                    - DIRECTIVES@25..49
                        - DIRECTIVE@25..49
                            - AT@25..26 "@"
                            - NAME@26..49
                                - IDENT@26..36 "deprecated"
                                - WHITESPACE@36..49 "\n            "
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
            - DOCUMENT@0..44
                - WHITESPACE@0..13 "\n            "
                - SCALAR_TYPE_DEFINITION@13..44
                    - scalar_KW@13..19 "scalar"
                    - WHITESPACE@19..20 " "
                    - DIRECTIVES@20..44
                        - DIRECTIVE@20..44
                            - AT@20..21 "@"
                            - NAME@21..44
                                - IDENT@21..31 "deprecated"
                                - WHITESPACE@31..44 "\n            "
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
            - DOCUMENT@0..56
                - WHITESPACE@0..13 "\n            "
                - SCALAR_TYPE_EXTENSION@13..56
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - scalar_KW@20..26 "scalar"
                    - WHITESPACE@26..27 " "
                    - NAME@27..32
                        - IDENT@27..31 "Time"
                        - WHITESPACE@31..32 " "
                    - DIRECTIVES@32..56
                        - DIRECTIVE@32..56
                            - AT@32..33 "@"
                            - NAME@33..56
                                - IDENT@33..43 "deprecated"
                                - WHITESPACE@43..56 "\n            "
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
            - DOCUMENT@0..51
                - WHITESPACE@0..13 "\n            "
                - SCALAR_TYPE_EXTENSION@13..51
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - scalar_KW@20..26 "scalar"
                    - WHITESPACE@26..27 " "
                    - DIRECTIVES@27..51
                        - DIRECTIVE@27..51
                            - AT@27..28 "@"
                            - NAME@28..51
                                - IDENT@28..38 "deprecated"
                                - WHITESPACE@38..51 "\n            "
            - ERROR@0:1 "expected a Name"
            "#,
        )
    }
}
