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
    cursor: Cursor<'a>,
    pub(crate) limit_tracker: LimitTracker,
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
            cursor: Cursor::new(input),
            index: 0,
            finished: false,
            limit_tracker: LimitTracker::new(usize::MAX),
        }
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit_tracker = LimitTracker::new(limit);
        self
    }

    /// Lex the full source text, consuming the lexer.
    pub fn lex(self) -> (Vec<Token<'a>>, Vec<Error>) {
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
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if self.input.is_empty() {
            let mut eof = Token::new(TokenKind::Eof, "EOF");
            eof.index = self.index;

            self.finished = true;
            return Some(Ok(eof));
        }

        self.limit_tracker.consume();
        if self.limit_tracker.limited() {
            self.finished = true;
            return Some(Err(Error::limit(
                "token limit reached, aborting lexing",
                self.cursor.index(),
            )));
        }

        match self.cursor.advance() {
            Ok(token) => {
                if matches!(token.kind(), TokenKind::Eof) {
                    self.finished = true;

                    return Some(Ok(token));
                }

                Some(Ok(token))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Cursor<'a> {
    fn advance(&mut self) -> Result<Token<'a>, Error> {
        #[derive(Debug)]
        enum State {
            Start,
            Done,
            Ident,
            StringLiteralEscapedUnicode(usize),
            StringLiteral,
            StringLiteralStart,
            BlockStringLiteralEscapedUnicode(usize),
            BlockStringLiteral,
            BlockStringLiteralBackslash,
            StringLiteralBackslash,
            IntLiteral,
            FloatLiteral,
            ExponentLiteral,
            Whitespace,
            Comment,
            SpreadOperator,
            PlusMinus,
        }

        let mut state = State::Start;
        let mut token = Token {
            kind: TokenKind::Eof,
            data: "EOF",
            index: self.index(),
        };

        while let Some(c) = self.bump() {
            match state {
                State::Start => {
                    match c {
                        '"' => {
                            token.kind = TokenKind::StringValue;
                            state = State::StringLiteralStart;
                        }
                        '#' => {
                            token.kind = TokenKind::Comment;
                            state = State::Comment;
                        }
                        '.' => {
                            token.kind = TokenKind::Spread;
                            state = State::SpreadOperator;
                        }
                        c if is_whitespace(c) => {
                            token.kind = TokenKind::Whitespace;
                            state = State::Whitespace;
                        }
                        c if is_ident_char(c) => {
                            token.kind = TokenKind::Name;
                            state = State::Ident;
                        }
                        '+' | '-' => {
                            token.kind = TokenKind::Int;
                            state = State::PlusMinus;
                        }
                        c if c.is_ascii_digit() => {
                            token.kind = TokenKind::Int;
                            state = State::IntLiteral;
                        }
                        '!' => {
                            token.kind = TokenKind::Bang;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '$' => {
                            token.kind = TokenKind::Dollar;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '&' => {
                            token.kind = TokenKind::Amp;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '(' => {
                            token.kind = TokenKind::LParen;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        ')' => {
                            token.kind = TokenKind::RParen;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        ':' => {
                            token.kind = TokenKind::Colon;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        ',' => {
                            token.kind = TokenKind::Comma;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '=' => {
                            token.kind = TokenKind::Eq;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '@' => {
                            token.kind = TokenKind::At;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '[' => {
                            token.kind = TokenKind::LBracket;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        ']' => {
                            token.kind = TokenKind::RBracket;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '{' => {
                            token.kind = TokenKind::LCurly;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '|' => {
                            token.kind = TokenKind::Pipe;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        '}' => {
                            token.kind = TokenKind::RCurly;
                            token.data = self.current_str();
                            return Ok(token);
                        }
                        c => {
                            return Err(Error::new(
                                format!("Unexpected character \"{}\"", c),
                                self.current_str().to_string(),
                            ))
                        }
                    };
                }
                State::Ident => match c {
                    curr if is_ident_char(curr) || curr.is_ascii_digit() => {}
                    _ => {
                        token.data = self.prev_str();

                        state = State::Done;
                        break;
                    }
                },
                State::Whitespace => match c {
                    curr if is_whitespace(curr) => {}
                    _ => {
                        token.data = self.prev_str();

                        state = State::Done;
                        break;
                    }
                },
                State::BlockStringLiteral => match c {
                    '\\' => {
                        state = State::BlockStringLiteralBackslash;
                    }
                    '"' => {
                        // Require two additional quotes to complete the triple quote.
                        if self.eatc('"') && self.eatc('"') {
                            token.data = self.current_str();

                            state = State::Done;
                            break;
                        }
                    }
                    _ => {}
                },
                State::StringLiteralStart => match c {
                    '"' => {
                        if self.eatc('"') {
                            state = State::BlockStringLiteral;

                            continue;
                        }

                        if self.pending() {
                            token.data = self.prev_str();
                        } else {
                            token.data = self.current_str();
                        }

                        state = State::Done;
                        break;
                    }
                    '\\' => {
                        state = State::StringLiteralBackslash;
                    }
                    _ => {
                        state = State::StringLiteral;

                        continue;
                    }
                },
                State::BlockStringLiteralEscapedUnicode(remaining) => match c {
                    '"' => {
                        self.add_err(Error::new(
                            "incomplete unicode escape sequence",
                            c.to_string(),
                        ));
                        token.data = self.current_str();
                        state = State::Done;

                        break;
                    }
                    c if !c.is_ascii_hexdigit() => {
                        self.add_err(Error::new("invalid unicode escape sequence", c.to_string()));
                        state = State::BlockStringLiteral;

                        continue;
                    }
                    _ => {
                        if remaining <= 1 {
                            state = State::BlockStringLiteral;

                            continue;
                        }

                        state = State::BlockStringLiteralEscapedUnicode(remaining - 1)
                    }
                },
                State::StringLiteralEscapedUnicode(remaining) => match c {
                    '"' => {
                        self.add_err(Error::new(
                            "incomplete unicode escape sequence",
                            c.to_string(),
                        ));
                        token.data = self.current_str();
                        state = State::Done;

                        break;
                    }
                    c if !c.is_ascii_hexdigit() => {
                        self.add_err(Error::new("invalid unicode escape sequence", c.to_string()));
                        state = State::StringLiteral;

                        continue;
                    }
                    _ => {
                        if remaining <= 1 {
                            state = State::StringLiteral;

                            continue;
                        }

                        state = State::StringLiteralEscapedUnicode(remaining - 1)
                    }
                },
                State::StringLiteral => match c {
                    '"' => {
                        token.data = self.current_str();

                        state = State::Done;
                        break;
                    }
                    curr if is_line_terminator(curr) => {
                        self.add_err(Error::new("unexpected line terminator", "".to_string()));
                    }
                    '\\' => {
                        state = State::StringLiteralBackslash;
                    }
                    _ => {}
                },
                State::BlockStringLiteralBackslash => match c {
                    '"' => {
                        while self.eatc('"') {}

                        state = State::BlockStringLiteral;
                    }
                    curr if is_escaped_char(curr) => {
                        state = State::BlockStringLiteral;
                    }
                    'u' => {
                        state = State::BlockStringLiteralEscapedUnicode(4);
                    }
                    _ => {
                        self.add_err(Error::new("unexpected escaped character", c.to_string()));

                        state = State::BlockStringLiteral;
                    }
                },
                State::StringLiteralBackslash => match c {
                    curr if is_escaped_char(curr) => {
                        state = State::StringLiteral;
                    }
                    'u' => {
                        state = State::StringLiteralEscapedUnicode(4);
                    }
                    _ => {
                        self.add_err(Error::new("unexpected escaped character", c.to_string()));

                        state = State::StringLiteral;
                    }
                },
                State::IntLiteral => match c {
                    curr if curr.is_ascii_digit() => {}
                    '.' => {
                        token.kind = TokenKind::Float;
                        state = State::FloatLiteral;
                    }
                    'e' | 'E' => {
                        token.kind = TokenKind::Float;
                        state = State::ExponentLiteral;
                    }
                    _ => {
                        token.data = self.prev_str();

                        state = State::Done;
                        break;
                    }
                },
                State::FloatLiteral => match c {
                    curr if curr.is_ascii_digit() => {}
                    '.' => {
                        self.add_err(Error::new(
                            format!("Unexpected character `{}`", c),
                            c.to_string(),
                        ));

                        continue;
                    }
                    'e' | 'E' => {
                        state = State::ExponentLiteral;
                    }
                    _ => {
                        token.data = self.prev_str();

                        state = State::Done;
                        break;
                    }
                },
                State::ExponentLiteral => match c {
                    curr if curr.is_ascii_digit() => {
                        state = State::FloatLiteral;
                    }
                    '+' | '-' => {
                        state = State::FloatLiteral;
                    }
                    _ => {
                        let err = self.current_str();
                        return Err(Error::new(
                            format!("Unexpected character `{}`", err),
                            err.to_string(),
                        ));
                    }
                },
                State::SpreadOperator => match c {
                    '.' => {
                        if self.eatc('.') {
                            token.data = self.current_str();
                            return Ok(token);
                        }

                        break;
                    }
                    _ => break,
                },
                State::PlusMinus => match c {
                    curr if curr.is_ascii_digit() => {
                        state = State::IntLiteral;
                    }
                    _ => {
                        let curr = self.current_str();
                        return Err(Error::new(
                            format!("Unexpected character `{}`", curr),
                            curr.to_string(),
                        ));
                    }
                },
                State::Comment => match c {
                    curr if is_line_terminator(curr) => {
                        token.data = self.prev_str();

                        state = State::Done;
                        break;
                    }
                    _ => {}
                },
                State::Done => unreachable!("must finalize loop when State::Done"),
            }
        }

        match state {
            State::Done => {
                if let Some(mut err) = self.err() {
                    err.set_data(token.data.to_string());
                    err.index = token.index;
                    self.err = None;

                    return Err(err);
                }

                Ok(token)
            }
            State::Start => {
                token.index += 1;
                Ok(token)
            }
            State::StringLiteralStart => {
                let curr = self.current_str();

                Err(Error::new(
                    "unexpected end of data while lexing string value",
                    curr.to_string(),
                ))
            }
            State::StringLiteral => {
                let curr = self.drain();

                Err(Error::with_loc(
                    "unterminated string value",
                    curr.to_string(),
                    token.index,
                ))
            }
            State::SpreadOperator => {
                let data = if self.pending() {
                    self.prev_str()
                } else {
                    self.current_str()
                };

                Err(Error::with_loc(
                    "Unterminated spread operator",
                    data.to_string(),
                    token.index,
                ))
            }
            _ => {
                if let Some(mut err) = self.err() {
                    err.set_data(self.current_str().to_string());
                    return Err(err);
                }

                token.data = self.current_str();

                Ok(token)
            }
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

// EscapedCharacter
//     "  \  /  b  f  n  r  t
fn is_escaped_char(c: char) -> bool {
    matches!(c, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't')
}

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
