use crate::{format_err, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#FragmentDefinition
///
/// ```txt
/// FragmentDefinition
///     fragment FragmentName TypeCondition Directives(opt) SelectionSet
/// ```
pub(crate) fn fragment_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::FRAGMENT_DEFINITION);
    // TODO @lrlna: parse description???
    parser.bump(SyntaxKind::fragment_KW);
    // parser.parse_whitespace();
    fragment_name(parser)?;

    // TODO(lrlna): parse TypeCondition, Directives, SelectionSet

    Ok(())
}

/// See: https://spec.graphql.org/June2018/#FragmentName
///
/// ```txt
/// FragmentName
///     Name *but not* on
/// ```
pub(crate) fn fragment_name(parser: &mut Parser) -> Result<(), crate::Error> {
    match parser.peek() {
        Some(TokenKind::Node) => {
            if parser.peek_data().unwrap() == "on" {
                // fragment name cannot have "on" as part of its definition
                return format_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Fragment Name cannot have 'on' as part of its definition"
                );
            }
            // TODO @lrlna: parse fragment name function
            parser.bump(SyntaxKind::NAME);
            Ok(())
        }
        // missing fragment name
        _ => {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Fragment name, got {}",
                parser.peek_data().unwrap()
            )
        }
    }
}
