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
    finished: bool,
    cursor: Cursor<'a>,
    pub(crate) limit_tracker: LimitTracker,
}

#[derive(Debug)]
enum State {
    Start,
    Ident,
    StringLiteralEscapedUnicode(usize),
    StringLiteral,
    StringLiteralStart,
    BlockStringLiteral,
    BlockStringLiteralBackslash,
    StringLiteralBackslash,
    IntLiteral,
    FloatLiteral,
    ExponentLiteral,
    Whitespace,
    Comment,
    SpreadOperator,
    MinusSign,
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
            cursor: Cursor::new(input),
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

        if self.limit_tracker.check_and_increment() {
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
                }

                Some(Ok(token))
            }
            Err(err) => Some(Err(err)),
        }
    }
}

impl<'a> Cursor<'a> {
    fn advance(&mut self) -> Result<Token<'a>, Error> {
        let mut state = State::Start;
        let mut token = Token {
            kind: TokenKind::Eof,
            data: "",
            index: self.index(),
        };

        loop {
            let Some(c) = self.bump() else {
                return self.eof(state, token);
            };
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
                        '-' => {
                            token.kind = TokenKind::Int;
                            state = State::MinusSign;
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
                        return self.done(token);
                    }
                },
                State::Whitespace => match c {
                    curr if is_whitespace(curr) => {}
                    _ => {
                        token.data = self.prev_str();
                        return self.done(token);
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
                            return self.done(token);
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

                        if self.is_pending() {
                            token.data = self.prev_str();
                        } else {
                            token.data = self.current_str();
                        }
                        return self.done(token);
                    }
                    '\\' => {
                        state = State::StringLiteralBackslash;
                    }
                    _ => {
                        state = State::StringLiteral;

                        continue;
                    }
                },
                State::StringLiteralEscapedUnicode(remaining) => match c {
                    '"' => {
                        self.add_err(Error::new(
                            "incomplete unicode escape sequence",
                            c.to_string(),
                        ));
                        token.data = self.current_str();
                        return self.done(token);
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
                        return self.done(token);
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
                        // If this is \""", we need to eat 3 in total, and then continue parsing.
                        // The lexer does not un-escape escape sequences so it's OK
                        // if we take this path for \"", even if that is technically not an escape
                        // sequence.
                        if self.eatc('"') {
                            self.eatc('"');
                        }

                        state = State::BlockStringLiteral;
                    }
                    _ => {
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
                        return self.done(token);
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
                        return self.done(token);
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
                State::SpreadOperator => {
                    if c == '.' && self.eatc('.') {
                        token.data = self.current_str();
                        return Ok(token);
                    }
                    return self.unterminated_spread_operator(&token);
                }
                State::MinusSign => match c {
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
                        return self.done(token);
                    }
                    _ => {}
                },
            }
        }
    }

    fn eof(&mut self, state: State, mut token: Token<'a>) -> Result<Token<'a>, Error> {
        match state {
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
            State::StringLiteral | State::BlockStringLiteral => {
                let curr = self.drain();

                Err(Error::with_loc(
                    "unterminated string value",
                    curr.to_string(),
                    token.index,
                ))
            }
            State::SpreadOperator => self.unterminated_spread_operator(&token),
            State::MinusSign => Err(Error::new(
                "Unexpected character \"-\"",
                self.current_str().to_string(),
            )),
            State::Ident
            | State::StringLiteralEscapedUnicode(_)
            | State::BlockStringLiteralBackslash
            | State::StringLiteralBackslash
            | State::IntLiteral
            | State::FloatLiteral
            | State::ExponentLiteral
            | State::Whitespace
            | State::Comment => {
                if let Some(mut err) = self.err() {
                    err.set_data(self.current_str().to_string());
                    return Err(err);
                }

                token.data = self.current_str();

                Ok(token)
            }
        }
    }

    fn unterminated_spread_operator(&mut self, token: &Token<'a>) -> Result<Token<'a>, Error> {
        let data = if self.is_pending() {
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

    fn done(&mut self, token: Token<'a>) -> Result<Token<'a>, Error> {
        if let Some(mut err) = self.err() {
            err.set_data(token.data.to_string());
            err.index = token.index;
            self.err = None;
            return Err(err);
        }
        Ok(token)
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
    fn token_limit_exact() {
        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(26);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 26);
        assert!(errors.is_empty());

        let lexer = Lexer::new("type Query { a a a a a a a a a }").with_limit(25);
        let (tokens, errors) = lexer.lex();
        assert_eq!(tokens.len(), 25);
        assert_eq!(
            errors,
            &[Error::limit("token limit reached, aborting lexing", 31)]
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

    #[test]
    fn stream_produces_original_input() {
        let schema = r#"
type Query {
    name: String
    format: String = "Y-m-d\\TH:i:sP"
}
        "#;

        let lexer = Lexer::new(schema);
        let processed_schema = lexer
            .into_iter()
            .fold(String::new(), |acc, token| acc + token.unwrap().data());

        assert_eq!(schema, processed_schema);
    }

    #[test]
    fn quoted_block_comment() {
        let input = r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
        "#;

        let (tokens, errors) = Lexer::new(input).lex();
        assert!(errors.is_empty());
        // The token data should be literally the source text.
        assert_eq!(
            tokens[1].data,
            r#"
"""
Not an escape character:
'/\W/'
Escape character:
\"""
\"""\"""
Not escape characters:
\" \""
Escape character followed by a quote:
\""""
"""
"#
            .trim(),
        );

        let input = r#"
# String contents: """
"""\""""""
# Unclosed block string
"""\"""
        "#;
        let (tokens, errors) = Lexer::new(input).lex();
        assert_eq!(tokens[3].data, r#""""\"""""""#);
        assert_eq!(
            errors,
            &[Error::with_loc(
                "unterminated string value",
                r#""""\"""
        "#
                .to_string(),
                59,
            )]
        );
    }
}
