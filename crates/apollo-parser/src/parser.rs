use crate::lexer::{Token, TokenKind};

impl From<TokenKind> for rowan::SyntaxKind {
    fn from(kind: TokenKind) -> Self {
        Self(kind.into())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Lang {}
impl rowan::Language for Lang {
    type Kind = TokenKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= TokenKind::Root.into());
        // We cannot create a token Kind from a raw u16 with the given Into
        // implementation. A different approach needs to be taken to be able to
        // convert from rowan::SyntaxKind u16 to our TokenKind.
        todo!();
        //unsafe { std::mem::transmute::<u16, TokenKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}
pub struct Parser {
    tokens: Vec<Token>,
}
