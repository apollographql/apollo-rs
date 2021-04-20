#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Node(String),
    Int(i64),
    Float(f64),
    Bang,     // !
    Dollar,   // $
    LParen,   // (
    RParen,   // )
    Ellipsis, // ...
    Colon,    // :
    Eq,       // =
    At,       // @
    LBracket, // [
    RBracket, // ]
    LBrace,   // {
    Pipe,     // |
    RBrace,   // }
    Eof,
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = self.loc.index;
        let end = self.loc.index + self.loc.length;

        match &self.kind {
            TokenKind::Node(s) => {
                write!(f, "NODE@{}:{} {:?}", start, end, s)
            }
            TokenKind::Int(n) => {
                write!(f, "INT@{}:{} {:?}", start, end, n)
            }
            TokenKind::Float(n) => {
                write!(f, "FLOAT@{}:{} {:?}", start, end, n)
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
            TokenKind::Ellipsis => {
                write!(f, "ELLIPSIS@{}:{}", start, end)
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
            TokenKind::Eof => {
                write!(f, "EOF@{}:{}", start, end)
            }
        }
    }
}

#[derive(Clone)]
pub struct Token {
    kind: TokenKind,
    loc: Location,
}

#[derive(Clone, Debug)]
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
                let res = advance(&mut input);
                let consumed = old_input.len() - input.len();
                length += consumed;

                match res {
                    Ok(kind) => {
                        let loc = Location::new(index, length - 1);
                        tokens.push(Token { kind, loc });
                        index += length;
                        length = 0;
                    }
                    Err(_) => todo!(),
                }
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

fn advance(input: &mut &str) -> Result<TokenKind, ()> {
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

            TokenKind::Node(buf)
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
                        if !has_digit {
                            panic!("Unexpected character in exponent");
                        }
                        if has_exponent {
                            panic!("Unexpected character 'e'");
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
                            panic!("unexpected . before a digit");
                        }
                        if has_fractional {
                            panic!("Unexpected .");
                        }
                        if has_exponent {
                            panic!("unexpected e");
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
            (Some('.'), Some('.')) => TokenKind::Ellipsis,
            (Some(a), Some(b)) => {
                panic!("Unterminated ellipsis, expected `...`, found `.{}{}`", a, b)
            }
            (Some(a), None) => panic!("Unterminated ellipsis, expected `...`, found `.{}`", a),
            (_, _) => panic!("Unterminated ellipsis, expected `...`, found `.`"),
        },
        ':' => TokenKind::Colon,
        '=' => TokenKind::Eq,
        '@' => TokenKind::At,
        '[' => TokenKind::LBracket,
        ']' => TokenKind::RBracket,
        '{' => TokenKind::LBrace,
        '|' => TokenKind::Pipe,
        '}' => TokenKind::RBrace,
        c => panic!("Unexpected character: `{}`", c),
    };

    *input = chars.as_str();
    Ok(kind)
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
        let gql = "directive @cacheControl";
        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);

        let gql = "fragment friend Fields on User {
            id name profilePic(size: 5.0)
        }";
        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);
    }
}
