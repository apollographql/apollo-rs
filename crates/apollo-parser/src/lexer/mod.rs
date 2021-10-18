mod location;
mod token;
mod token_kind;

use crate::{create_err, Error, ensure, format_err};

pub use location::Location;
pub use token::Token;
pub use token_kind::TokenKind;

/// Parse text into tokens.
pub struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<Error>,
}

impl Lexer {
    /// Create a new instance of `Lexer`.
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

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
                        tokens.push(t);
                    }
                    Err(mut e) => {
                        e.loc = loc;
                        index += e.data.len();
                        errors.push(e);
                    }
                };
            }
        }

        let mut eof = Token::new(TokenKind::Eof, String::from("EOF"));
        eof.loc = Location::new(index);
        tokens.push(eof);

        Self { tokens, errors }
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
    pub(crate) fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    /// Get a reference to the lexer's tokens.
    pub(crate) fn errors(&self) -> &[Error] {
        self.errors.as_slice()
    }

    //pub(crate) fn push_err(&self, m: String, data: &str) {
    //    let err = Error::new(m.to_string(), data.to_string());
    //    self.errors.push(err)
    //}
}

fn advance(input: &mut &str) -> Result<Token, Error> {
    let mut chars = input.chars();
    let c = chars.next().unwrap();

    let kind = match c {
        '"' => {
            // TODO @lrlna: consider using a 'terminated' bool to store whether a string
            // character or block character are terminated (rust's lexer does this).
            let mut buf = String::new();
            buf.push(c); // the first " we already matched on

            let c = chars.next().unwrap();
            match c {
                '"' => {
                    buf.push(c); // the second " we already matched on

                    // TODO @lrlna: don't clone these chars.
                    // The clone is currently in place to account for empty string values, or "".
                    // If we encounter "", we need to exit this match statmenet
                    // and continue where we left off. Without the clone we miss
                    // the next char entirely.
                    if let '"' = chars.clone().next().unwrap() {
                        buf.push(chars.next().unwrap());

                        while let Some(c) = chars.clone().next() {
                            if c == '"' {
                                buf.push(chars.next().unwrap());
                                let n1 = chars.next();
                                let n2 = chars.next();
                                match (n1, n2) {
                                    (Some('"'), Some('"')) => {
                                        buf.push(n1.unwrap());
                                        buf.push(n2.unwrap());
                                        break;
                                    }
                                    (Some(a), Some(b)) => {
                                        buf.push(a);
                                        buf.push(b);
                                        let current = format!("{}{}", a, b);
                                        create_err!(current,
                                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                                current,
                                            );
                                        break;
                                    }
                                    (Some(a), None) => {
                                        buf.push(a);
                                        create_err!(a,
                                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                                a
                                            );
                                        break;
                                    }
                                    (_, _) => {
                                        buf.push(chars.next().unwrap());
                                        create_err!(
                                                "",
                                                "Unterminated block comment, expected `\"\"\"`, found `\"`"
                                            );
                                        break;
                                    }
                                }
                            } else if is_source_char(c) {
                                buf.push(chars.next().unwrap());
                            } else {
                                break;
                            }
                        }

                        return Ok(Token::new(TokenKind::StringValue, buf));
                    }

                    Ok(Token::new(TokenKind::StringValue, buf))
                }
                t => {
                    buf.push(t);

                    while let Some(c) = chars.clone().next() {
                        if c == '"' {
                            buf.push(chars.next().unwrap());
                            break;
                        } else if is_escaped_char(c)
                            || is_source_char(c) && c != '\\' && c != '"' && !is_line_terminator(c)
                        {
                            buf.push(chars.next().unwrap());
                        // TODO @lrlna: this should error if c == \ or has a line terminator
                        } else {
                            break;
                        }
                    }

                    Ok(Token::new(TokenKind::StringValue, buf))
                }
            }
        }
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
                if is_ident_char(c) || is_digit_char(c) {
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
        '&' => Ok(Token::new(TokenKind::Amp, c.into())),
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
    // from rust's lexer:
    matches!(
        c,
        // ASCII
        '\u{0009}'   // \t
        | '\u{000A}' // \n
        | '\u{000B}' // vertical tab
        | '\u{000C}' // form feed
        | '\u{000D}' // \r
        | '\u{0020}' // space

        // Unicode BOM (Byte Order Mark)
        | '\u{FEFF}'

        // NEXT LINE from latin1
        | '\u{0085}'

        // Bidi markers
        | '\u{200E}' // LEFT-TO-RIGHT MARK
        | '\u{200F}' // RIGHT-TO-LEFT MARK

        // Dedicated whitespace characters from Unicode
        | '\u{2028}' // LINE SEPARATOR
        | '\u{2029}' // PARAGRAPH SEPARATOR
    )
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
    use indoc::indoc;

    #[test]
    fn tests() {
        let gql_1 = indoc! { r#"
enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts" url: "" )
}"#};
        let lexer_1 = Lexer::new(gql_1);
        dbg!(lexer_1.tokens);
        dbg!(lexer_1.errors);
    }
}
