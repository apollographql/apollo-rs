use crate::{format_err, name, Parser, SyntaxKind, TokenKind};

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// InputValueDefinition
///     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
/// ```
pub(crate) fn input_value_definitions(
    parser: &mut Parser,
    is_input: bool,
) -> Result<(), crate::Error> {
    // TODO lrlna: parse description
    // TODO lrlna: parse default value
    // TODO lrlna: parse directives
    match parser.peek() {
        // Name
        Some(TokenKind::Node) => {
            // TODO lrlna: use parse input value name function
            let guard = parser.start_node(SyntaxKind::INPUT_VALUE_DEFINITION);
            name(parser)?;
            match parser.peek() {
                // Colon
                Some(TokenKind::Colon) => {
                    parser.bump(SyntaxKind::COLON);
                    match parser.peek() {
                        // Type
                        Some(TokenKind::Node) => {
                            // TODO lrlna: use parse input value type function
                            parser.bump(SyntaxKind::TYPE);
                            match parser.peek() {
                                Some(_) => {
                                    guard.finish_node();
                                    input_value_definitions(parser, true)
                                }
                                _ => Ok(()),
                            }
                        }
                        _ => {
                            return format_err!(
                                parser.peek_data().unwrap(),
                                "Expected InputValue definition to have a Type, got {}",
                                parser.peek_data().unwrap()
                            )
                        }
                    }
                }
                _ => {
                    return format_err!(
                        parser.peek_data().unwrap(),
                        "Expected InputValue definition to have a Name, got {}",
                        parser.peek_data().unwrap()
                    )
                }
            }
        }
        Some(TokenKind::Comma) => {
            parser.bump(SyntaxKind::COMMA);
            input_value_definitions(parser, is_input)
        }
        _ => {
            // if we already have an input, can proceed without returning an error
            if is_input {
                Ok(())
            } else {
                // if there is no input, and a LPAREN was supplied, send an error
                return format_err!(
                    parser.peek_data().unwrap(),
                    "Expected to have an InputValue definition, got {}",
                    parser.peek_data().unwrap()
                );
            }
        }
    }
}
