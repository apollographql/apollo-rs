mod location;
mod token;
mod token_kind;

use crate::{create_err, Error};

pub use location::Location;
pub use token::Token;
pub use token_kind::TokenKind;

/// Parses tokens into text.
pub(crate) struct Lexer {
    tokens: Vec<Token>,
    errors: Vec<Error>,
    index: usize,
}

impl Lexer {
    /// Create a new instance of `Lexer`.
    pub fn new(input: &str) -> Self {
        let tokens = Vec::new();
        let errors = Vec::new();
        let index = 0;

        let mut lexer = Self {
            tokens,
            errors,
            index,
        };

        lexer.parse(input);
        lexer
    }

    fn parse(&mut self, mut input: &str) {
        while !input.is_empty() {
            let old_input = input;

            if old_input.len() == input.len() {
                self.advance(&mut input);
            }
        }

        let mut eof = Token::new(TokenKind::Eof, String::from("EOF"));
        eof.loc = Location::new(self.index);
        self.tokens.push(eof);
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

    fn push_err(&mut self, mut err: Error) {
        let loc = Location::new(self.index);
        err.loc = loc;
        self.index += err.data.len();
        self.errors.push(err);
    }

    fn push_token(&mut self, mut token: Token) {
        let loc = Location::new(self.index);
        token.loc = loc;
        self.index += token.data.len();
        self.tokens.push(token);
    }

    fn advance(&mut self, input: &mut &str) {
        let mut chars = input.chars();
        let c = chars.next().unwrap();

        match c {
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
                                            self.push_err(create_err!(current,
                                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                                current,
                                            ));
                                            break;
                                        }
                                        (Some(a), None) => {
                                            buf.push(a);
                                            self.push_err(create_err!(a,
                                                "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                                a
                                            ));
                                            break;
                                        }
                                        (_, _) => {
                                            buf.push(chars.next().unwrap());
                                            self.push_err(create_err!(
                                                "",
                                                "Unterminated block comment, expected `\"\"\"`, found `\"`"
                                            ));
                                            break;
                                        }
                                    }
                                } else if is_source_char(c) {
                                    buf.push(chars.next().unwrap());
                                } else {
                                    break;
                                }
                            }
                        }

                        self.push_token(Token::new(TokenKind::StringValue, buf));
                    }
                    t => {
                        buf.push(t);

                        while let Some(c) = chars.clone().next() {
                            if c == '"' {
                                buf.push(chars.next().unwrap());
                                break;
                            } else if is_escaped_char(c)
                                || is_source_char(c)
                                    && c != '\\'
                                    && c != '"'
                                    && !is_line_terminator(c)
                            {
                                buf.push(chars.next().unwrap());
                            // TODO @lrlna: this should error if c == \ or has a line terminator
                            } else {
                                break;
                            }
                        }

                        self.push_token(Token::new(TokenKind::StringValue, buf));
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

                self.push_token(Token::new(TokenKind::Comment, buf));
            }
            '.' => match (chars.next(), chars.next()) {
                (Some('.'), Some('.')) => {
                    self.push_token(Token::new(TokenKind::Spread, "...".to_string()))
                }
                (Some(a), Some(b)) => self.push_err(create_err!(
                    format!("{}{}", a, b),
                    "Unterminated spread operator, expected `...`, found `.{}{}`",
                    a,
                    b,
                )),
                (Some(a), None) => {
                    self.push_err(create_err!(
                        a,
                        "Unterminated spread, expected `...`, found `.{}`",
                        a
                    ));
                }
                (_, _) => self.push_err(create_err!(
                    "",
                    "Unterminated spread operator, expected `...`, found `.`"
                )),
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

                self.push_token(Token::new(TokenKind::Whitespace, buf));
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

                self.push_token(Token::new(TokenKind::Name, buf));
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
                            if has_digit {
                                self.push_err(create_err!(
                                    c,
                                    "Unexpected character `{}` in exponent",
                                    c
                                ));
                            }
                            if has_exponent {
                                self.push_err(create_err!(c, "Unexpected character `{}`", c));
                            }
                            buf.push(chars.next().unwrap());
                            has_exponent = true;
                            if let Some(c) = chars.clone().next() {
                                if matches!(c, '+' | '-') {
                                    buf.push(chars.next().unwrap());
                                }
                            }
                        }
                        '.' => {
                            if !has_digit {
                                self.push_err(create_err!(
                                    c,
                                    "Unexpected character `{}` before a digit",
                                    c
                                ));
                            }
                            if has_fractional {
                                self.push_err(create_err!(c, "Unexpected character `{}`", c));
                            }
                            if has_exponent {
                                self.push_err(create_err!(c, "Unexpected character `{}`", c));
                            }
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
                    self.push_token(Token::new(TokenKind::Float, buf))
                } else {
                    self.push_token(Token::new(TokenKind::Int, buf))
                }
            }
            '!' => self.push_token(Token::new(TokenKind::Bang, c.into())),
            '$' => self.push_token(Token::new(TokenKind::Dollar, c.into())),
            '&' => self.push_token(Token::new(TokenKind::Amp, c.into())),
            '(' => self.push_token(Token::new(TokenKind::LParen, c.into())),
            ')' => self.push_token(Token::new(TokenKind::RParen, c.into())),
            ':' => self.push_token(Token::new(TokenKind::Colon, c.into())),
            ',' => self.push_token(Token::new(TokenKind::Comma, c.into())),
            '=' => self.push_token(Token::new(TokenKind::Eq, c.into())),
            '@' => self.push_token(Token::new(TokenKind::At, c.into())),
            '[' => self.push_token(Token::new(TokenKind::LBracket, c.into())),
            ']' => self.push_token(Token::new(TokenKind::RBracket, c.into())),
            '{' => self.push_token(Token::new(TokenKind::LCurly, c.into())),
            '|' => self.push_token(Token::new(TokenKind::Pipe, c.into())),
            '}' => self.push_token(Token::new(TokenKind::RCurly, c.into())),
            c => self.push_err(create_err!(c, "Unexpected character: {}", c)),
        };

        *input = chars.as_str();
    }
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
