mod cursor;
mod location;
mod token;
mod token_kind;

use std::str::Chars;

use crate::{create_err, ensure, format_err, Error};

pub use location::Location;
pub use token::Token;
pub use token_kind::TokenKind;

pub(crate) const EOF_CHAR: char = '\0';

pub(crate) struct Cursor<'a> {
    initial_len: usize,
    chars: Chars<'a>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            initial_len: input.len(),
            chars: input.chars(),
        }
    }
}

/// Parses tokens into text.
pub(crate) struct Lexer<'a> {
    tokens: Vec<Token>,
    errors: Vec<Error>,
    input: &'a str,
    chars: Chars<'a>,
    initial_len: usize,
}

impl<'a> Lexer<'a> {
    /// Create a new instance of `Lexer`.
    pub fn new(mut input: &'a str) -> Self {
        Self {
            tokens: Vec::new(),
            errors: Vec::new(),
            input,
            chars: input.chars(),
            initial_len: input.len(),
        }
    }

    pub fn tokenise(&mut self) {
        // while !input.is_empty() {
        //     let token = self.advance();
        //     self.input = self.input[token.len..].to_owned();
        //     self.tokens.push(token);
        // }

        while !self.input.is_empty() {
            let old_input = self.input;

            if old_input.len() == self.input.len() {
                self.chars = self.input.chars();
                self.initial_len = self.input.len();

                let token = self.advance();
                self.input = &self.input[token.len..];
                self.tokens.push(token);
            }
        }
    }

    /// Get a reference to the lexer's tokens.
    pub(crate) fn tokens(&self) -> &[Token] {
        self.tokens.as_slice()
    }

    /// Get a reference to the lexer's tokens.
    pub(crate) fn errors(&self) -> &[Error] {
        self.errors.as_slice()
    }

    fn advance(&mut self) -> Token {
        let first_char = self.bump().unwrap();

        let kind = match first_char {
            '"' => self.string_value(first_char),
            '#' => self.comment(first_char),
            '.' => self.spread_operator(),
            c if is_whitespace(c) => self.whitespace(c),
            c if is_ident_char(c) => self.ident(c),
            c @ '-' | c @ '+' => self.number(c),
            c if is_digit_char(c) => self.number(c),
            '!' => Token::new(TokenKind::Bang, first_char.into(), self.len_consumed()),
            '$' => Token::new(TokenKind::Dollar, first_char.into(), self.len_consumed()),
            '&' => Token::new(TokenKind::Amp, first_char.into(), self.len_consumed()),
            '(' => Token::new(TokenKind::LParen, first_char.into(), self.len_consumed()),
            ')' => Token::new(TokenKind::RParen, first_char.into(), self.len_consumed()),
            ':' => Token::new(TokenKind::Colon, first_char.into(), self.len_consumed()),
            ',' => Token::new(TokenKind::Comma, first_char.into(), self.len_consumed()),
            '=' => Token::new(TokenKind::Eq, first_char.into(), self.len_consumed()),
            '@' => Token::new(TokenKind::At, first_char.into(), self.len_consumed()),
            '[' => Token::new(TokenKind::LBracket, first_char.into(), self.len_consumed()),
            ']' => Token::new(TokenKind::RBracket, first_char.into(), self.len_consumed()),
            '{' => Token::new(TokenKind::LCurly, first_char.into(), self.len_consumed()),
            '|' => Token::new(TokenKind::Pipe, first_char.into(), self.len_consumed()),
            '}' => Token::new(TokenKind::RCurly, first_char.into(), self.len_consumed()),
            _c => todo!(), // create_err!(c, "Unexpected character: {}", c),
        };

        kind
    }

    fn string_value(&mut self, first_char: char) -> Token {
        // TODO @lrlna: consider using a 'terminated' bool to store whether a string
        // character or block character are terminated (rust's lexer does this).
        let mut buf = String::new();
        buf.push(first_char); // the first " we already matched on
        self.bump();

        let c = self.first();
        match c {
            '"' => {
                buf.push(c); // the second " we already matched on
                self.bump();

                // TODO @lrlna: don't clone these chars.
                // The clone is currently in place to account for empty string values, or "".
                // If we encounter "", we need to exit this match statmenet
                // and continue where we left off. Without the clone we miss
                // the next char entirely.
                if let '"' = self.first() {
                    buf.push(self.first());
                    self.bump();

                    while !self.is_eof() {
                        let first = self.first();
                        if first == '"' {
                            buf.push(first);
                            self.bump();
                            match (self.first(), self.second()) {
                                ('"', '"') => {
                                    buf.push(self.first());
                                    buf.push(self.second());
                                    self.bump();
                                    self.bump();
                                    break;
                                }
                                (_a, _b) => {
                                    // let current = format!("{}{}", a, b);
                                    // create_err!(current,
                                    //             "Unterminated block comment, expected `\"\"\"`, found `\"{}`",
                                    //             current,
                                    //         );
                                    break;
                                }
                            }
                        } else if is_source_char(first) {
                            buf.push(first);
                            self.bump();
                        } else {
                            break;
                        }
                    }

                    return Token::new(TokenKind::StringValue, buf, self.len_consumed());
                }

                Token::new(TokenKind::StringValue, buf, self.len_consumed())
            }
            t => {
                buf.push(t);
                self.bump();

                while !self.is_eof() {
                    let first = self.first();
                    if first == '"' {
                        buf.push(first);
                        self.bump();
                        break;
                    } else if is_escaped_char(first)
                        || is_source_char(first)
                            && first != '\\'
                            && first != '"'
                            && !is_line_terminator(first)
                    {
                        buf.push(first);
                        self.bump();
                    // TODO @lrlna: this should error if c == \ or has a line terminator
                    } else {
                        break;
                    }
                }

                Token::new(TokenKind::StringValue, buf, self.len_consumed())
            }
        }
    }

    fn comment(&mut self, first_char: char) -> Token {
        let mut buf = String::new();
        buf.push(first_char);
        self.bump();

        while !self.is_eof() {
            let first = self.first();
            if !is_line_terminator(first) {
                buf.push(first);
                self.bump();
            } else {
                break;
            }
        }

        Token::new(TokenKind::Comment, buf, self.len_consumed())
    }

    fn spread_operator(&mut self) -> Token {
        self.bump();
        match (self.first(), self.second()) {
            ('.', '.') => {
                self.bump();
                self.bump();
                Token::new(TokenKind::Spread, "...".to_string(), self.len_consumed())
            }
            (_a, _b) => todo!(),
            // create_err!(
            //     format!("{}{}", a, b),
            //     "Unterminated spread operator, expected `...`, found `.{}{}`",
            //     a,
            //     b,
            // ),
        }
    }

    fn whitespace(&mut self, first_char: char) -> Token {
        let mut buf = String::new();
        buf.push(first_char);
        self.bump();

        while !self.is_eof() {
            let first = self.first();
            if is_whitespace(first) {
                buf.push(first);
                self.bump();
            } else {
                break;
            }
        }

        Token::new(TokenKind::Whitespace, buf, self.len_consumed())
    }

    fn ident(&mut self, first_char: char) -> Token {
        let mut buf = String::new();
        buf.push(first_char);
        self.bump();

        while !self.is_eof() {
            let first = self.first();
            if is_ident_char(first) || is_digit_char(first) {
                buf.push(first);
                self.bump();
            } else {
                break;
            }
        }

        Token::new(TokenKind::Name, buf, self.len_consumed())
    }

    fn number(&mut self, first_digit: char) -> Token {
        let mut buf = String::new();
        buf.push(first_digit);
        self.bump();

        let mut has_exponent = false;
        let mut has_fractional = false;
        let mut has_digit = is_digit_char(first_digit);

        while !self.is_eof() {
            let first = self.first();
            match first {
                'e' | 'E' => {
                    // ensure!(!has_digit, c, "Unexpected character `{}` in exponent", c);
                    // ensure!(!has_exponent, c, "Unexpected character `{}` c", c);
                    buf.push(first);
                    self.bump();
                    has_exponent = true;
                    if matches!(self.first(), '+' | '-') {
                        buf.push(self.first());
                        self.bump();
                    }
                }
                '.' => {
                    // ensure!(has_digit, c, "Unexpected character `{}` before a digit", c);
                    // ensure!(!has_fractional, c, "Unexpected character `{}` a", c);
                    // ensure!(!has_exponent, c, "Unexpected character `{}` b ", c);
                    buf.push(first);
                    self.bump();
                    has_fractional = true;
                }
                first if is_digit_char(first) => {
                    buf.push(first);
                    self.bump();
                    has_digit = true;
                }
                _ => break,
            }
        }

        if has_exponent || has_fractional {
            Token::new(TokenKind::Float, buf, self.len_consumed())
        } else {
            Token::new(TokenKind::Int, buf, self.len_consumed())
        }
    }

    /// Returns nth character relative to the current cursor position.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    fn nth_char(&self, n: usize) -> char {
        self.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    /// Peeks the next symbol from the input stream without consuming it.
    pub(crate) fn first(&self) -> char {
        self.nth_char(0)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    pub(crate) fn second(&self) -> char {
        self.nth_char(1)
    }
    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Returns amount of already consumed symbols.
    pub(crate) fn len_consumed(&self) -> usize {
        self.initial_len - self.chars.as_str().len()
    }
    /// Returns a `Chars` iterator over the remaining characters.
    fn chars(&self) -> Chars<'_> {
        self.chars.clone()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        Some(c)
    }

    // pub(crate) fn push_err(&self, m: String, data: &str) {
    //     let err = Error::new(m.to_string(), data.to_string());
    //     self.errors.push(err)
    // }
}

impl Cursor<'_> {}

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
    // use indoc::indoc;

    #[test]
    fn tests() {
        let gql_1 = "4";
        let lexer_1 = Lexer::new(gql_1);
        dbg!(lexer_1.tokens);
        dbg!(lexer_1.errors);
    }
}
