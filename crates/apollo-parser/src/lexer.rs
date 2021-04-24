macro_rules! format_err {
    ($data:expr, $($tt:tt)*) => {
        $crate::lexer::TokenKind::Error {
            message: format!($($tt)*),
            data: $data.to_string(),
        }
    };
}

macro_rules! ensure {
    ($cond:expr, $data:expr, $($tt:tt)*) => {
        if !$cond {
            return $crate::lexer::TokenKind::Error {
                message: format!($($tt)*),
                data: $data.to_string(),
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Root,
    Bang,     // !
    Dollar,   // $
    LParen,   // (
    RParen,   // )
    Spread,   // ...
    Colon,    // :
    Eq,       // =
    At,       // @
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    Pipe,     // |
    RBrace,   // }
    Fragment,
    Directive,
    Query,
    On,
    Eof,

    // composite nodes
    Node(String),
    Int(i64),
    Float(f64),
    Error {
        /// The raw data that's part of the error range.
        data: String,
        /// The corresponding error message.
        message: String,
    },
}

impl Into<u16> for TokenKind {
    fn into(self) -> u16 {
        match self {
            TokenKind::Root => 0,
            TokenKind::Bang => 1,
            TokenKind::Dollar => 2,
            TokenKind::LParen => 3,
            TokenKind::RParen => 4,
            TokenKind::Spread => 5,
            TokenKind::Colon => 6,
            TokenKind::Eq => 7,
            TokenKind::At => 8,
            TokenKind::LBracket => 9,
            TokenKind::RBracket => 10,
            TokenKind::LBrace => 11,
            TokenKind::Pipe => 12,
            TokenKind::RBrace => 13,
            TokenKind::Fragment => 14,
            TokenKind::Directive => 15,
            TokenKind::Query => 16,
            TokenKind::On => 17,
            TokenKind::Eof => 18,

            // composite nodes
            TokenKind::Node(_) => 19,
            TokenKind::Int(_) => 20,
            TokenKind::Float(_) => 21,
            TokenKind::Error { .. } => 22,
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.loc.index;
        let end = self.loc.index + self.loc.length;

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
            TokenKind::Node(s) => {
                write!(f, "NODE@{}:{} {:?}", start, end, s)
            }
            TokenKind::Int(n) => {
                write!(f, "INT@{}:{} {:?}", start, end, n)
            }
            TokenKind::Float(n) => {
                write!(f, "FLOAT@{}:{} {:?}", start, end, n)
            }
            TokenKind::Error { message, .. } => {
                write!(f, "ERROR@{}:{} {:?}", start, end, message)
            }
        }
    }
}

#[derive(Clone)]
pub struct Token {
    kind: TokenKind,
    loc: Location,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Location {
    index: usize,
    length: usize,
}

impl Location {
    pub fn new(index: usize, length: usize) -> Self {
        Self { index, length }
    }

    /// Get a reference to the location's index.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get a reference to the location's length.
    pub fn length(&self) -> usize {
        self.length
    }
}

pub struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();

        let mut index = 0;
        let mut length = 0;

        while !input.is_empty() {
            let old_input = input;
            skip_ws(&mut input);
            skip_comment(&mut input);

            if old_input.len() == input.len() {
                let kind = advance(&mut input);
                let consumed = old_input.len() - input.len();
                length += consumed;

                let loc = Location::new(index, length - 1);
                tokens.push(Token { kind, loc });
                index += length;
                length = 0;
            }
        }

        Self { tokens }
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop().expect("Unexpected EOF")
    }

    pub fn peek(&mut self) -> Option<Token> {
        self.tokens.last().cloned()
    }
}

fn advance(input: &mut &str) -> TokenKind {
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
                "on" => TokenKind::On,
                "directive" => TokenKind::Directive,
                "fragment" => TokenKind::Fragment,
                "query" => TokenKind::Query,
                _ => TokenKind::Node(buf),
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
                TokenKind::Float(buf.parse().unwrap())
            } else {
                TokenKind::Int(buf.parse().unwrap())
            }
        }
        '!' => TokenKind::Bang,
        '$' => TokenKind::Dollar,
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        '.' => match (chars.next(), chars.next()) {
            (Some('.'), Some('.')) => TokenKind::Spread,
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
        ':' => TokenKind::Colon,
        '=' => TokenKind::Eq,
        '@' => TokenKind::At,
        '[' => TokenKind::LBracket,
        ']' => TokenKind::RBracket,
        '{' => TokenKind::LBrace,
        '|' => TokenKind::Pipe,
        '}' => TokenKind::RBrace,
        c => format_err!(c, "Unexpected character: {}", c),
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
        let gql = "directive @example on FIELD";
        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);

        let gql = "fragment friendFields on User {
            id name profilePic(size: 5.0)
        }";
        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);

        let gql = "query withFragments {
  user(id: 4) {
    friends(first: 10) {
      ...friendFields
    }
    mutualFriends(first: 10)æ {
      ...friendFields
    }
  }
}";

        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);
    }
}
