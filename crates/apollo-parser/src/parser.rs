use rowan::{GreenNode, GreenNodeBuilder};

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u16)]
pub enum SyntaxKind {
    // Whitespace
    Whitespace = 0,

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

    // Root node
    Root,
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

#[derive(Debug)]
pub struct ParseResult {
    pub green_node: GreenNode,
    pub errors: Vec<String>,
}

#[derive(Debug)]
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
    pub fn new(mut tokens: Vec<(SyntaxKind, String)>) -> Self {
        tokens.reverse();

        Self {
            tokens,
            builder: GreenNodeBuilder::new(),
            errors: vec![],
        }
    }

    pub fn parse(mut self) -> ParseResult {
        self.builder.start_node(SyntaxKind::Root.into());

        // Bang,     // !
        // Dollar,   // $
        // LParen,   // (
        // RParen,   // )
        // Spread,   // ...
        // Colon,    // :
        // Eq,       // =
        // At,       // @
        // LBracket, // [
        // RBracket, // ]
        // LBrace,   // {
        // Pipe,     // |
        // RBrace,   // }

        // Fragment,
        // Directive,
        // Query,
        // On,
        // Node,
        // Int,
        // Float
        loop {
            match self.peek() {
                None => break,
                Some(SyntaxKind::Directive) => {
                    if self.parse_directive().is_err() {
                        panic!("could not parse directive");
                    }
                }
                Some(_) => break,
            }
        }

        self.builder.finish_node();

        ParseResult {
            green_node: self.builder.finish(),
            errors: self.errors,
        }
    }

    fn parse_directive(&mut self) -> Result<(), ()> {
        self.builder.start_node(SyntaxKind::Directive.into());
        self.bump();
        self.parse_whitespace();

        match self.peek() {
            Some(SyntaxKind::At) => self.bump(),
            _ => return Err(()),
        }
        match self.peek() {
            Some(SyntaxKind::Node) => self.bump(),
            _ => return Err(()),
        }

        self.parse_whitespace();
        match self.peek() {
            Some(SyntaxKind::On) => self.bump(),
            _ => return Err(()),
        }

        self.parse_whitespace();
        match self.peek() {
            Some(SyntaxKind::Node) => self.bump(),
            _ => return Err(()),
        }
        self.builder.finish_node();
        Ok(())
    }

    pub fn parse_whitespace(&mut self) {
        let mut text = String::new();
        while let Some(SyntaxKind::Whitespace) = self.peek() {
            let (_, s) = self.tokens.pop().unwrap();
            text.push_str(&s);
        }
        self.builder.token(SyntaxKind::Whitespace.into(), &text);
    }

    pub fn bump(&mut self) {
        let (kind, text) = self.tokens.pop().unwrap();
        self.builder.token(kind.into(), &text);
    }

    pub fn peek(&self) -> Option<SyntaxKind> {
        self.tokens.last().map(|(kind, _)| *kind)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke() {
        // directive @example on FIELD
        let parser = Parser::new(vec![
            (SyntaxKind::Directive, "directive".to_string()),
            (SyntaxKind::Whitespace, " ".to_string()),
            (SyntaxKind::At, "@".to_string()),
            (SyntaxKind::Node, "example".to_string()),
            (SyntaxKind::Whitespace, " ".to_string()),
            (SyntaxKind::On, "on".to_string()),
            (SyntaxKind::Whitespace, " ".to_string()),
            (SyntaxKind::Node, "FIELD".to_string()),
        ]);

        let ParseResult { green_node, .. } = parser.parse();
        let ast = SyntaxNode::new_root(green_node);
        print(0, ast.into());
    }

    type SyntaxNode = rowan::SyntaxNode<super::Language>;
    #[allow(unused)]
    type SyntaxToken = rowan::SyntaxToken<super::Language>;
    #[allow(unused)]
    type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

    fn print(indent: usize, element: SyntaxElement) {
        let kind: SyntaxKind = element.kind().into();
        print!("{:indent$}", "", indent = indent);
        match element {
            rowan::NodeOrToken::Node(node) => {
                println!("- {:?}@{:?}", kind, node.text_range());
                for child in node.children_with_tokens() {
                    print(indent + 2, child);
                }
            }

            rowan::NodeOrToken::Token(token) => {
                println!("- {:?}@{:?} {:?}", kind, token.text_range(), token.text())
            }
        }
    }
}
