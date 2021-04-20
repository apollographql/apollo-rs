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

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    kind: TokenKind,
}

pub struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(mut input: &str) -> Self {
        let mut tokens = Vec::new();

        while !input.is_empty() {
            let old_input = input;
            skip_ws(&mut input);
            skip_comment(&mut input);
            if old_input.len() == input.len() {
                match advance(&mut input) {
                    Ok(kind) => tokens.push(Token { kind }),
                    Err(_) => todo!(),
                }
            }
        }

        Self { tokens }
    }

    pub fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token {
            kind: TokenKind::Eof,
        })
    }

    pub fn peek(&mut self) -> Token {
        self.tokens.last().cloned().unwrap_or(Token {
            kind: TokenKind::Eof,
        })
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

            while let Some(c) = chars.clone().next() {
                if is_digit_char(c) {
                    buf.push(chars.next().unwrap());
                } else {
                    break;
                }
            }

            TokenKind::Int(buf.parse().unwrap())
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

        let gql = "fragment friend Fields on User { id name profilePic(size: 50) }";
        let lexer = Lexer::new(gql);
        dbg!(lexer.tokens);
    }
}
