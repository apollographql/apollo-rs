mod cursor;
mod token;
mod token_kind;

use crate::{lexer::cursor::Cursor, Error, LimitTracker};

pub use token::Token;
pub use token_kind::TokenKind;

/// Parses GraphQL source text into tokens.
/// ```rust
/// use apollo_parser::Lexer;
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// let (tokens, errors) = Lexer::new(query).lex();
/// assert_eq!(errors.len(), 0);
/// ```
#[derive(Clone, Debug)]
pub struct Lexer<'a> {
    input: &'a str,
    index: usize,
    finished: bool,
    limit: Option<LimitTracker>,
}

impl<'a> Lexer<'a> {
    /// Create a lexer for a GraphQL source text.
    ///
    /// The Lexer is an iterator over tokens and errors:
    /// ```rust
    /// use apollo_parser::Lexer;
    ///
    /// let query = "# --- GraphQL here ---";
    ///
    /// let mut lexer = Lexer::new(query);
    /// let mut tokens = vec![];
    /// for token in lexer {
    ///     match token {
    ///         Ok(token) => tokens.push(token),
    ///         Err(error) => panic!("{:?}", error),
    ///     }
    /// }
    /// ```
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            index: 0,
            finished: false,
            limit: None,
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(LimitTracker::new(limit));
        self
    }

    /// Lex the full source text, consuming the lexer.
    pub fn lex(self) -> (Vec<Token>, Vec<Error>) {
        let mut tokens = vec![];
        let mut errors = vec![];

        for item in self {
            match item {
                Ok(token) => tokens.push(token),
                Err(error) => errors.push(error),
            }
        }

        (tokens, errors)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Result<Token, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        if self.input.is_empty() {
            let mut eof = Token::new(TokenKind::Eof, String::from("EOF"));
            eof.index = self.index;

            self.finished = true;
            return Some(Ok(eof));
        }

        if let Some(limit) = &mut self.limit {
            limit.consume();
            if limit.limited() {
                self.finished = true;
                return Some(Err(Error::limit(
                    "token limit reached, aborting lexing",
                    self.index,
                )));
            }
        }

        let mut c = Cursor::new(self.input);
        let r = c.advance();

        match r {
            Ok(mut token) => {
                token.index = self.index;
                self.index += token.data.len();

                self.input = &self.input[token.data.len()..];
                Some(Ok(token))
            }
            Err(mut err) => {
                err.index = self.index;
                self.index += err.data.len();

                self.input = &self.input[err.data.len()..];
                Some(Err(err))
            }
        }
    }
}

impl Cursor<'_> {
    fn advance(&mut self) -> Result<Token, Error> {
        let first_char = self.bump().unwrap();

        match first_char {
            '"' => self.string_value(first_char),
            '#' => self.comment(first_char),
            '.' => self.spread_operator(first_char),
            c if is_whitespace(c) => self.whitespace(c),
            c if is_ident_char(c) => self.ident(c),
            c @ '-' | c @ '+' => self.number(c),
            c if is_digit_char(c) => self.number(c),
            '!' => Ok(Token::new(TokenKind::Bang, first_char.into())),
            '$' => Ok(Token::new(TokenKind::Dollar, first_char.into())),
            '&' => Ok(Token::new(TokenKind::Amp, first_char.into())),
            '(' => Ok(Token::new(TokenKind::LParen, first_char.into())),
            ')' => Ok(Token::new(TokenKind::RParen, first_char.into())),
            ':' => Ok(Token::new(TokenKind::Colon, first_char.into())),
            ',' => Ok(Token::new(TokenKind::Comma, first_char.into())),
            '=' => Ok(Token::new(TokenKind::Eq, first_char.into())),
            '@' => Ok(Token::new(TokenKind::At, first_char.into())),
            '[' => Ok(Token::new(TokenKind::LBracket, first_char.into())),
            ']' => Ok(Token::new(TokenKind::RBracket, first_char.into())),
            '{' => Ok(Token::new(TokenKind::LCurly, first_char.into())),
            '|' => Ok(Token::new(TokenKind::Pipe, first_char.into())),
            '}' => Ok(Token::new(TokenKind::RCurly, first_char.into())),
            c => Err(Error::new("Unexpected character", c.to_string())),
        }
    }

    fn string_value(&mut self, first_char: char) -> Result<Token, Error> {
        // TODO @lrlna: consider using a 'terminated' bool to store whether a string
        // character or block character are terminated (rust's lexer does this).
        let mut buf = String::new();
        buf.push(first_char); // the first " we already matched on

        let c = match self.bump() {
            None => {
                return Err(Error::new(
                    "unexpected end of data while lexing string value",
                    "\"".to_string(),
                ));
            }
            Some(c) => c,
        };

        match c {
            '"' => self.block_string_value(buf, c),
            t => {
                buf.push(t);
                let mut was_backslash = t == '\\';

                while !self.is_eof() {
                    let c = self.bump().unwrap();

                    if was_backslash && !is_escaped_char(c) && c != 'u' {
                        self.add_err(Error::new("unexpected escaped character", c.to_string()));
                    }

                    buf.push(c);
                    if c == '"' {
                        if !was_backslash {
                            break;
                        }
                    } else if is_line_terminator(c) {
                        self.add_err(Error::new("unexpected line terminator", c.to_string()));
                    }
                    was_backslash = c == '\\' && !was_backslash;
                }

                if !buf.ends_with('"') {
                    // If it's an unclosed string then take all remaining tokens into this string value
                    while !self.is_eof() {
                        buf.push(self.bump().unwrap());
                    }
                    self.add_err(Error::new("unterminated string value", buf.clone()));
                }

                if let Some(mut err) = self.err() {
                    err.set_data(buf);
                    return Err(err);
                }

                Ok(Token::new(TokenKind::StringValue, buf))
            }
        }
    }

    fn block_string_value(&mut self, mut buf: String, char: char) -> Result<Token, Error> {
        buf.push(char); // the second " we already matched on

        let c = match self.bump() {
            None => {
                return Ok(Token::new(TokenKind::StringValue, buf));
            }
            Some(c) => c,
        };

        if let first_char @ '"' = c {
            buf.push(first_char);

            while !self.is_eof() {
                let c = self.bump().unwrap();
                let was_backslash = c == '\\';

                if was_backslash && !is_escaped_char(c) && c != 'u' {
                    self.add_err(Error::new("unexpected escaped character", c.to_string()));
                }

                buf.push(c);

                if was_backslash {
                    while self.first() == '"' {
                        buf.push(self.first());
                        self.bump();
                    }
                } else if c == '"' && ('"', '"') == (self.first(), self.second()) {
                    buf.push(self.first());
                    buf.push(self.second());
                    self.bump();
                    self.bump();
                    break;
                }
            }
        }

        Ok(Token::new(TokenKind::StringValue, buf))
    }

    fn comment(&mut self, first_char: char) -> Result<Token, Error> {
        let mut buf = String::new();
        buf.push(first_char);

        while !self.is_eof() {
            let first = self.bump().unwrap();
            if !is_line_terminator(first) {
                buf.push(first);
            } else {
                break;
            }
        }

        Ok(Token::new(TokenKind::Comment, buf))
    }

    fn spread_operator(&mut self, first_char: char) -> Result<Token, Error> {
        let mut buf = String::new();
        buf.push(first_char);

        match (self.first(), self.second()) {
            ('.', '.') => {
                buf.push('.');
                buf.push('.');
                self.bump();
                self.bump();
            }
            ('.', b) => {
                self.bump();
                buf.push('.');

                self.add_err(Error::new(
                    "Unterminated spread operator",
                    format!("..{}", b),
                ));
            }
            (a, b) => self.add_err(Error::new(
                "Unterminated spread operator",
                format!(".{}{}", a, b),
            )),
        }

        if let Some(mut err) = self.err() {
            err.set_data(buf);
            return Err(err);
        }

        Ok(Token::new(TokenKind::Spread, buf))
    }

    fn whitespace(&mut self, first_char: char) -> Result<Token, Error> {
        let mut buf = String::new();
        buf.push(first_char);

        while !self.is_eof() {
            let first = self.bump().unwrap();
            if is_whitespace(first) {
                buf.push(first);
            } else {
                break;
            }
        }

        Ok(Token::new(TokenKind::Whitespace, buf))
    }

    fn ident(&mut self, first_char: char) -> Result<Token, Error> {
        let mut buf = String::new();
        buf.push(first_char);

        while !self.is_eof() {
            let first = self.first();
            if is_ident_char(first) || is_digit_char(first) {
                buf.push(first);
                self.bump();
            } else {
                break;
            }
        }

        Ok(Token::new(TokenKind::Name, buf))
    }

    fn number(&mut self, first_digit: char) -> Result<Token, Error> {
        let mut buf = String::new();
        buf.push(first_digit);

        let mut has_exponent = false;
        let mut has_fractional = false;
        let mut has_digit = is_digit_char(first_digit);

        while !self.is_eof() {
            let first = self.first();
            match first {
                'e' | 'E' => {
                    buf.push(first);
                    self.bump();
                    if !has_digit {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}` in exponent", first),
                            first.to_string(),
                        ));
                    }
                    if has_exponent {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}`", first),
                            first.to_string(),
                        ));
                    }
                    has_exponent = true;
                    if matches!(self.first(), '+' | '-') {
                        buf.push(self.first());
                        self.bump();
                    }
                }
                '.' => {
                    buf.push(first);
                    self.bump();

                    if !has_digit {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}` before a digit", first),
                            first.to_string(),
                        ));
                    }

                    if has_fractional {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}`", first),
                            first.to_string(),
                        ));
                    }

                    if has_exponent {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}`", first),
                            first.to_string(),
                        ));
                    }

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

        if let Some(mut err) = self.err() {
            err.set_data(buf);
            return Err(err);
        }

        if has_exponent || has_fractional {
            Ok(Token::new(TokenKind::Float, buf))
        } else {
            Ok(Token::new(TokenKind::Int, buf))
        }
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
// fn is_source_char(c: char) -> bool {
//     matches!(c, '\t' | '\r' | '\n' | '\u{0020}'..='\u{FFFF}')
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn unterminated_string() {
        let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP" 
}
        "#;
        let (tokens, errors) = Lexer::new(schema).lex();
        dbg!(tokens);
        dbg!(errors);
    }

    #[test]
    fn token_limit() {
        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(10);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 10);
        assert_eq!(
            errors,
            &[Error::limit("token limit reached, aborting lexing", 17)]
        );
    }

    #[test]
    fn errors_and_token_limit() {
        let lexer = Lexer::new("type Query { ..a a a a a a a a a }").with_limit(10);
        let (tokens, errors) = lexer.lex();
        // Errors contribute to the token limit
        assert_eq!(tokens.len(), 9);
        assert_eq!(
            errors,
            &[
                Error::with_loc("Unterminated spread operator", "..".to_string(), 13),
                Error::limit("token limit reached, aborting lexing", 18),
            ],
        );
    }
}
