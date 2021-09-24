use crate::{create_err, Error};
use crate::{ensure, format_err};

pub use location::Location;
pub use token::Token;
pub use token_kind::TokenKind;

mod location;
mod token;
mod token_kind;

/// Parse text into tokens.
pub struct Lexer {
    tokens: Vec<Result<Token, Error>>,
}

impl Lexer {
    /// Create a new instance of `Lexer`.
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();

        let mut index = 0;

        while !input.is_empty() {
            let old_input = input;

            if old_input.len() == input.len() {
                let r = advance(&mut input);
                let loc = Location::new(index);
                // Match on the Result type from the advance function and add
                // location information before pushing a Result to tokens
                // vector.
                match r {
                    Ok(mut t) => {
                        t.loc = loc;
                        index += t.data.len();
                        tokens.push(Ok(t));
                    }
                    Err(mut e) => {
                        e.loc = loc;
                        index += e.data.len();
                        tokens.push(Err(e));
                    }
                };
            }
        }

        tokens.push(Ok(Token::new(TokenKind::Eof, String::from("EOF"))));

        Self { tokens }
    }

    /// Advance the cursor and get the next token.
    // pub(crate) fn next(&mut self) -> Result<Token, Error> {
    //     self.tokens.pop().expect("Unexpected EOF")
    // }

    /// Parse the next token without advancing the cursor.
    // pub(crate) fn peek(&mut self) -> Option<Result<Token, Error>> {
    //     self.tokens.last().cloned()
    // }

    /// Get a reference to the lexer's tokens.
    pub(crate) fn tokens(&self) -> &[Result<Token, Error>] {
        self.tokens.as_slice()
    }
}

fn advance(input: &mut &str) -> Result<Token, Error> {
    let mut chars = input.chars();
    let c = chars.next().unwrap();

    let kind = match c {
        '"' => match chars.next().unwrap() {
            '"' => {
                let mut buf = String::new();
                buf.push(c); // the first " we already matched on
                buf.push(chars.next().unwrap()); // the second " we already matched on

                let c = chars.next().unwrap();
                match c {
                    '"' => {
                        buf.push(c);

                        while let Some(c) = chars.clone().next() {
                            if c != '"' {
                                buf.push(chars.next().unwrap());
                            } else if c == '"' {
                                match (chars.next(), chars.next()) {
                                    (Some('"'), Some('"')) => {
                                        buf.push(chars.next().unwrap());
                                        buf.push(chars.next().unwrap());
                                        break;
                                    }
                                    (Some(a), Some(b)) => {
                                        buf.push(chars.next().unwrap());
                                        buf.push(chars.next().unwrap());
                                        let current = format!("{}{}", a, b);
                                        create_err!(
                                current,
                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                current,
                            );
                                        break;
                                    }
                                    (Some(a), None) => {
                                        buf.push(chars.next().unwrap());
                                        buf.push(chars.next().unwrap());
                                        create_err!(
                                a,
                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                a
                            );
                                        break;
                                    }
                                    (_, _) => {
                                        buf.push(chars.next().unwrap());
                                        buf.push(chars.next().unwrap());
                                        create_err!(
                                "",
                                "Unterminated block comment, expected `\"\"\"`, found `\"`"
                            );
                                        break;
                                    }
                                }
                            } else {
                                break;
                            }
                        }

                        Ok(Token::new(TokenKind::StringValue, buf))
                    }
                    _ => Ok(Token::new(TokenKind::StringValue, buf)),
                }
            }
            _ => {
                // TODO @lrlna: consider using a 'terminated' bool to store whether a string
                // character or block character are terminated (rust's lexer does this).
                let mut buf = String::new();
                buf.push(c); // the first " we already matched on

                while let Some(c) = chars.clone().next() {
                    if is_escaped_char(c)
                        || is_source_char(c) && c != '\\' && c != '"' && !is_line_terminator(c)
                    {
                        buf.push(chars.next().unwrap());
                    } else if c == '"' {
                        buf.push(chars.next().unwrap());
                        break;
                    // TODO @lrlna: this should error if c == \ or has a line terminator
                    } else {
                        break;
                    }
                }

                Ok(Token::new(TokenKind::StringValue, buf))
            }
        },
        '#' => {
            let mut buf = String::new();
            buf.push(c);

            while let Some(c) = chars.clone().next() {
                if !is_line_terminator(c) {
                    buf.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            Ok(Token::new(TokenKind::Comment, buf))
        }
        '.' => match (chars.next(), chars.next()) {
            (Some('.'), Some('.')) => Ok(Token::new(TokenKind::Spread, "...".to_string())),
            (Some(a), Some(b)) => format_err!(
                format!("{}{}", a, b),
                "Unterminated spread operator, expected `...`, found `.{}{}`",
                a,
                b,
            ),
            (Some(a), None) => {
                format_err!(a, "Unterminated spread, expected `...`, found `.{}`", a)
            }
            (_, _) => format_err!(
                "",
                "Unterminated spread operator, expected `...`, found `.`"
            ),
        },
        c if is_whitespace(c) => {
            let mut buf = String::new();
            buf.push(c);

            while let Some(c) = chars.clone().next() {
                if is_whitespace(c) {
                    buf.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            Ok(Token::new(TokenKind::Whitespace, buf))
        }
        c if is_ident_char(c) => {
            let mut buf = String::new();
            buf.push(c);

            while let Some(c) = chars.clone().next() {
                if is_ident_char(c) {
                    buf.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            Ok(Token::new(TokenKind::Name, buf))
        }
        c @ '-' | c if is_digit_char(c) => {
            let mut buf = String::new();
            buf.push(c);

            let mut has_exponent = false;
            let mut has_fractional = false;
            let mut has_digit = is_digit_char(c);

            while let Some(c) = chars.clone().next() {
                match c {
                    'e' | 'E' => {
                        ensure!(!has_digit, c, "Unexpected character `{}` in exponent", c);
                        ensure!(!has_exponent, c, "Unexpected character `{}`", c);
                        buf.push(chars.next().unwrap());
                        has_exponent = true;
                        if let Some(c) = chars.clone().next() {
                            if matches!(c, '+' | '-') {
                                buf.push(chars.next().unwrap());
                            }
                        }
                    }
                    '.' => {
                        ensure!(has_digit, c, "Unexpected character `{}` before a digit", c);
                        ensure!(!has_fractional, c, "Unexpected character `{}`", c);
                        ensure!(!has_exponent, c, "Unexpected character `{}`", c);
                        buf.push(chars.next().unwrap());
                        has_fractional = true;
                    }
                    c if is_digit_char(c) => {
                        buf.push(chars.next().unwrap());
                        has_digit = true;
                    }
                    _ => break,
                }
            }

            if has_exponent || has_fractional {
                Ok(Token::new(TokenKind::Float, buf))
            } else {
                Ok(Token::new(TokenKind::Int, buf))
            }
        }
        '!' => Ok(Token::new(TokenKind::Bang, c.into())),
        '$' => Ok(Token::new(TokenKind::Dollar, c.into())),
        '(' => Ok(Token::new(TokenKind::LParen, c.into())),
        ')' => Ok(Token::new(TokenKind::RParen, c.into())),
        ':' => Ok(Token::new(TokenKind::Colon, c.into())),
        ',' => Ok(Token::new(TokenKind::Comma, c.into())),
        '=' => Ok(Token::new(TokenKind::Eq, c.into())),
        '@' => Ok(Token::new(TokenKind::At, c.into())),
        '[' => Ok(Token::new(TokenKind::LBracket, c.into())),
        ']' => Ok(Token::new(TokenKind::RBracket, c.into())),
        '{' => Ok(Token::new(TokenKind::LCurly, c.into())),
        '|' => Ok(Token::new(TokenKind::Pipe, c.into())),
        '}' => Ok(Token::new(TokenKind::RCurly, c.into())),
        c => format_err!(c, "Unexpected character: {}", c),
    };

    *input = chars.as_str();
    kind
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r')
}

fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z' | '_')
}

fn is_line_terminator(c: char) -> bool {
    matches!(c, '\n' | '\r')
}

fn is_digit_char(c: char) -> bool {
    matches!(c, '0'..='9')
}

// EscapedCharacter
//     "  \  /  b  f  n  r  t
fn is_escaped_char(c: char) -> bool {
    matches!(c, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')
}

// SourceCharacter
//     /[\u0009\u000A\u000D\u0020-\uFFFF]/
fn is_source_char(c: char) -> bool {
    matches!(c, '\t' | '\r' | '\n' | '\u{0020}'..='\u{FFFF}')
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tests() {
        let gql_1 = r#"
{ foo(a: [ü]) }
"#;
        let lexer_1 = Lexer::new(gql_1);
        dbg!(lexer_1.tokens);
    }
}
