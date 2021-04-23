use rowan::{GreenNode, GreenNodeBuilder};

#[derive(Debug, Clone, PartialEq)]
#[repr(u16)]
pub enum SyntaxKind {
    Root = 0,

    // Symbols
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

    // Keywords and types
    Fragment,
    Directive,
    Query,
    On,
    Node,
    Int,
    Float,
}

impl From<SyntaxKind> for rowan::SyntaxKind {
    fn from(kind: SyntaxKind) -> Self {
        Self(kind as u16)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Language {}
impl rowan::Language for Language {
    type Kind = SyntaxKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= SyntaxKind::Root as u16);
        unsafe { std::mem::transmute::<u16, SyntaxKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub struct ParseResult {
    pub green_node: GreenNode,
    pub errors: Vec<String>,
}

pub struct Parser {
    /// input tokens, including whitespace,
    /// in *reverse* order.
    tokens: Vec<(SyntaxKind, String)>,
    /// the in-progress tree.
    builder: GreenNodeBuilder<'static>,
    /// the list of syntax errors we've accumulated
    /// so far.
    errors: Vec<String>,
}

impl Parser {
    pub fn new(tokens: Vec<(SyntaxKind, String)>) -> Self {
        Self {
            tokens,
            builder: GreenNodeBuilder::new(),
            errors: vec![],
        }
    }

    pub fn parse(self) -> ParseResult {
        todo!();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        todo!();
    }
}
