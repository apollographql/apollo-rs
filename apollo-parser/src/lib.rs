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
        let line = self.loc.line;
        let column = self.loc.column;
        // IDENT@3..6 "foo"

        match &self.kind {
            TokenKind::Node(s) => {
                write!(f, "NODE@{}:{} {:?}", line, column, s)
            }
            TokenKind::Int(n) => {
                write!(f, "INT@{}:{} {:?}", line, column, n)
            }
            TokenKind::Float(n) => {
                write!(f, "FLOAT@{}:{} {:?}", line, column, n)
            }
            TokenKind::Bang => {
                write!(f, "BANG@{}:{}", line, column)
            }
            TokenKind::Dollar => {
                write!(f, "DOLLAR@{}:{}", line, column)
            }
            TokenKind::LParen => {
                write!(f, "L_PAREN@{}:{}", line, column)
            }
            TokenKind::RParen => {
                write!(f, "R_PAREN@{}:{}", line, column)
            }
            TokenKind::Ellipsis => {
                write!(f, "ELLIPSIS@{}:{}", line, column)
            }
            TokenKind::Colon => {
                write!(f, "COLON@{}:{}", line, column)
            }
            TokenKind::Eq => {
                write!(f, "EQ@{}:{}", line, column)
            }
            TokenKind::At => {
                write!(f, "AT@{}:{}", line, column)
            }
            TokenKind::LBracket => {
                write!(f, "L_BRACKET@{}:{}", line, column)
            }
            TokenKind::RBracket => {
                write!(f, "R_BRACKET@{}:{}", line, column)
            }
            TokenKind::LBrace => {
                write!(f, "L_BRACE@{}:{}", line, column)
            }
            TokenKind::Pipe => {
                write!(f, "PIPE@{}:{}", line, column)
            }
            TokenKind::RBrace => {
                write!(f, "R_BRACE@{}:{}", line, column)
            }
            TokenKind::Eof => {
                write!(f, "EOF@{}:{}", line, column)
            }
        }
    }
}

#[derive(Clone)]
pub struct Token {
    kind: TokenKind,
    loc: Location,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Location {
    line: usize,
    column: usize,
}

impl Location {
    fn advance(&mut self, text: &str) {
        match text.rfind('\n') {
            Some(idx) => {
                self.line += text.chars().filter(|&it| it == '\n').count();
                self.column = text[idx + 1..].chars().count();
            }
            None => self.column += text.chars().count(),
        }
    }

    /// Get a reference to the location's line.
    pub fn line(&self) -> &usize {
        &self.line
    }

    /// Get a reference to the location's column.
    pub fn column(&self) -> &usize {
        &self.column
    }
}

pub struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();
        let mut loc = Location::default();

        while !input.is_empty() {
            let old_input = input;
            skip_ws(&mut input);
            skip_comment(&mut input);
            if old_input.len() == input.len() {
                match advance(&mut input) {
                    Ok(kind) => tokens.push(Token { kind, loc }),
                    Err(_) => todo!(),
                }
            }

            let consumed = old_input.len() - input.len();
            loc.advance(&old_input[..consumed]);
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
