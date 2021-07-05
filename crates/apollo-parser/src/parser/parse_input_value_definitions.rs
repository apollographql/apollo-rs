use crate::{Parser, TokenKind};

/// See: https://spec.graphql.org/June2018/#InputValueDefinition
///
/// ```txt
/// InputValueDefinition
///     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
/// ```
pub(crate) fn parse_input_value_definitions(parser: &mut Parser, is_input: bool) -> Result<(), ()> {
    // TODO: parse description
    // TODO: parse default value
    // TODO: parse directives
    match parser.peek() {
        // Name
        Some(TokenKind::Node) => {
            parser.bump();
            match parser.peek() {
                // Colon
                Some(TokenKind::Colon) => {
                    parser.bump();
                    match parser.peek() {
                        // Type
                        Some(TokenKind::Node) => {
                            parser.bump();
                            match parser.peek() {
                                Some(_) => parse_input_value_definitions(parser, true),
                                _ => Ok(()),
                            }
                        }
                        _ => return Err(()),
                    }
                }
                _ => return Err(()),
            }
        }
        Some(TokenKind::Comma) => {
            parser.bump();
            parse_input_value_definitions(parser, is_input)
        }
        _ => {
            // if we already have an input, can proceed without returning an error
            if is_input {
                Ok(())
            } else {
                // if there is no input, and a LPAREN was supplied, send an error
                return Err(());
            }
        }
    }
}
