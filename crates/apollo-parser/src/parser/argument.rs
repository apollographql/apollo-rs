use crate::{format_err, name, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#Argument
///
/// ```txt
/// Argument
///    Name : Value
/// ```
pub(crate) fn argument(parser: &mut Parser, is_argument: bool) -> Result<(), crate::Error> {
    match parser.peek() {
        // Name
        Some(TokenKind::Node) => {
            let guard = parser.start_node(SyntaxKind::ARGUMENT);
            name(parser)?;
            match parser.peek() {
                // Colon
                Some(TokenKind::Colon) => {
                    parser.bump(SyntaxKind::COLON);
                    match parser.peek() {
                        // Type
                        Some(TokenKind::Node) => {
                            // TODO lrlna: use value type function
                            parser.bump(SyntaxKind::VALUE);
                            match parser.peek() {
                                Some(_) => {
                                    guard.finish_node();
                                    argument(parser, true)
                                }
                                _ => Ok(()),
                            }
                        }
                        _ => {
                            return format_err!(
                                parser
                                    .peek_data()
                                    .unwrap_or_else(|| String::from("no further data")),
                                "Expected Argument to have a Value, got {}",
                                parser
                                    .peek_data()
                                    .unwrap_or_else(|| String::from("no further data"))
                            )
                        }
                    }
                }
                _ => {
                    return format_err!(
                        parser
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data")),
                        "Expected Argument to have a Name, got {}",
                        parser
                            .peek_data()
                            .unwrap_or_else(|| String::from("no further data"))
                    )
                }
            }
        }
        Some(TokenKind::Comma) => {
            parser.bump(SyntaxKind::COMMA);
            argument(parser, is_argument)
        }
        _ => {
            // if we already have an input, can proceed without returning an error
            if is_argument {
                Ok(())
            } else {
                // if there is no input, and a LPAREN was supplied, send an error
                return format_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Expected to have an Argument, got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                );
            }
        }
    }
}
