use crate::token_kind::TokenKind;

macro_rules! bail {
    ($data:expr, $($tt:tt)*) => {
        return Err($crate::lexer::Error::new(
            format!($($tt)*),
            $data.to_string(),
        ))
    };
}

macro_rules! ensure {
    ($cond:expr, $data:expr, $($tt:tt)*) => {
        if !$cond {
            return Err($crate::lexer::Error::new(
                format!($($tt)*),
                $data.to_string(),
            ))
        }
    };
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.loc.index;
        let end = self.loc.index + self.data.len();

        match &self.kind {
            TokenKind::Root => {
                write!(f, "ROOT@{}:{}", start, end)
            }
            TokenKind::Bang => {
                write!(f, "BANG@{}:{}", start, end)
            }
            TokenKind::Dollar => {
                write!(f, "DOLLAR@{}:{}", start, end)
            }
            TokenKind::LParen => {
                write!(f, "L_PAREN@{}:{}", start, end)
            }
            TokenKind::RParen => {
                write!(f, "R_PAREN@{}:{}", start, end)
            }
            TokenKind::Spread => {
                write!(f, "SPREAD@{}:{}", start, end)
            }
            TokenKind::Colon => {
                write!(f, "COLON@{}:{}", start, end)
            }
            TokenKind::Comma => {
                write!(f, "COMMA@{}:{}", start, end)
            }
            TokenKind::Eq => {
                write!(f, "EQ@{}:{}", start, end)
            }
            TokenKind::At => {
                write!(f, "AT@{}:{}", start, end)
            }
            TokenKind::LBracket => {
                write!(f, "L_BRACKET@{}:{}", start, end)
            }
            TokenKind::RBracket => {
                write!(f, "R_BRACKET@{}:{}", start, end)
            }
            TokenKind::LBrace => {
                write!(f, "L_BRACE@{}:{}", start, end)
            }
            TokenKind::Pipe => {
                write!(f, "PIPE@{}:{}", start, end)
            }
            TokenKind::RBrace => {
                write!(f, "R_BRACE@{}:{}", start, end)
            }
            TokenKind::Directive => {
                write!(f, "DIRECTIVE@{}:{}", start, end)
            }
            TokenKind::Fragment => {
                write!(f, "FRAGMENT@{}:{}", start, end)
            }
            TokenKind::Query => {
                write!(f, "QUERY@{}:{}", start, end)
            }
            TokenKind::On => {
                write!(f, "ON@{}:{}", start, end)
            }
            TokenKind::Eof => {
                write!(f, "EOF@{}:{}", start, end)
            }

            // composite nodes
            TokenKind::Node => {
                write!(f, "NODE@{}:{} {:?}", start, end, self.data)
            }
            TokenKind::Int => {
                write!(f, "INT@{}:{} {:?}", start, end, self.data)
            }
            TokenKind::Float => {
                write!(f, "FLOAT@{}:{} {:?}", start, end, self.data)
            }
        }
    }
}

#[derive(Clone)]
pub struct Token {
    kind: TokenKind,
    data: String,
    loc: Location,
}

impl Token {
    fn new(kind: TokenKind, data: String) -> Self {
        Self {
            kind,
            data,
            loc: Location::new(0),
        }
    }

    /// Get a reference to the token's kind.
    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    /// Get a reference to the token's data.
    pub fn data(&self) -> &str {
        self.data.as_str()
    }

    /// Get a reference to the token's loc.
    pub fn loc(&self) -> Location {
        self.loc
    }
}

#[derive(Clone)]
pub struct Error {
    message: String,
    data: String,
    loc: Location,
}

impl Error {
    pub fn new(message: String, data: String) -> Self {
        Self {
            message,
            data,
            loc: Location::new(0),
        }
    }

    pub fn with_loc(message: String, data: String, loc: Location) -> Self {
        Self { message, data, loc }
    }
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.loc.index;
        let end = self.loc.index + self.data.len();

        write!(f, "ERROR@{}:{} {:?}", start, end, self.message)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    index: usize,
}

impl Location {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    /// Get a reference to the location's index.
    pub fn index(&self) -> usize {
        self.index
    }
}

pub struct Lexer {
    tokens: Vec<Result<Token, Error>>,
}

impl Lexer {
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();

        let mut index = 0;

        while !input.is_empty() {
            let old_input = input;
            // TODO: do not skip comment
            // TODO: add comment to token kinds
            // TODO: add parsing of comment to parser.rs
            skip_ws(&mut input);
            skip_comment(&mut input);

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

        Self { tokens }
    }

    pub fn next(&mut self) -> Result<Token, Error> {
        self.tokens.pop().expect("Unexpected EOF")
    }

    pub fn peek(&mut self) -> Option<Result<Token, Error>> {
        self.tokens.last().cloned()
    }

    /// Get a reference to the lexer's tokens.
    pub fn tokens(&self) -> &[Result<Token, Error>] {
        self.tokens.as_slice()
    }
}

fn advance(input: &mut &str) -> Result<Token, Error> {
    let mut chars = input.chars();
    let c = chars.next().unwrap();

    let kind = match c {
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

            match buf.as_str() {
                "on" => Ok(Token::new(TokenKind::On, buf)),
                "directive" => Ok(Token::new(TokenKind::Directive, buf)),
                "fragment" => Ok(Token::new(TokenKind::Fragment, buf)),
                "query" => Ok(Token::new(TokenKind::Query, buf)),
                _ => Ok(Token::new(TokenKind::Node, buf)),
            }
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
        '.' => match (chars.next(), chars.next()) {
            (Some('.'), Some('.')) => Ok(Token::new(TokenKind::Spread, "...".to_string())),
            (Some(a), Some(b)) => bail!(
                format!("{}{}", a, b),
                "Unterminated spread operator, expected `...`, found `.{}{}`",
                a,
                b,
            ),
            (Some(a), None) => {
                bail!(a, "Unterminated spread, expected `...`, found `.{}`", a)
            }
            (_, _) => bail!(
                "",
                "Unterminated spread operator, expected `...`, found `.`"
            ),
        },
        ':' => Ok(Token::new(TokenKind::Colon, c.into())),
        ',' => Ok(Token::new(TokenKind::Comma, c.into())),
        '=' => Ok(Token::new(TokenKind::Eq, c.into())),
        '@' => Ok(Token::new(TokenKind::At, c.into())),
        '[' => Ok(Token::new(TokenKind::LBracket, c.into())),
        ']' => Ok(Token::new(TokenKind::RBracket, c.into())),
        '{' => Ok(Token::new(TokenKind::LBrace, c.into())),
        '|' => Ok(Token::new(TokenKind::Pipe, c.into())),
        '}' => Ok(Token::new(TokenKind::RBrace, c.into())),
        c => bail!(c, "Unexpected character: {}", c),
    };

    *input = chars.as_str();
    kind
}

fn skip_ws(input: &mut &str) {
    *input = input.trim_start_matches(is_whitespace)
}

fn skip_comment(input: &mut &str) {
    if input.starts_with("#") {
        let idx = input.find('\n').map_or(input.len(), |it| it + 1);
        *input = &input[idx..]
    }
}

fn is_whitespace(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n')
}

fn is_ident_char(c: char) -> bool {
    matches!(c, 'a'..='z' | 'A'..='Z')
}

fn is_digit_char(c: char) -> bool {
    matches!(c, '0'..='9')
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn tests() {
        let gql_1 = "directive @example on FIELD";
        let lexer_1 = Lexer::new(gql_1);
        dbg!(lexer_1.tokens);

        let gql_2 = "fragment friendFields on User {
            id name profilePic(size: 5.0)
        }";
        let lexer_2 = Lexer::new(gql_2);
        dbg!(lexer_2.tokens);

        let gql_3 = "query withFragments {
  user(id: 4) {
    friends(first: 10) {
      ...friendFields
    }
    mutualFriends(first: 10)æ {
      ...friendFields
    }
  }
}";

        let lexer_3 = Lexer::new(gql_3);
        dbg!(lexer_3.tokens);
    }
}
