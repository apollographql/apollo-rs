use std::fmt;

use rowan::GreenNodeBuilder;

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

pub struct SyntaxTree {
    ast: rowan::SyntaxNode<Language>,
    errors: Vec<String>,
}

impl SyntaxTree {
    /// Get a reference to the syntax tree's errors.
    pub fn errors(&self) -> &Vec<String> {
        &self.errors
    }
}

impl fmt::Debug for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        type SyntaxNode = rowan::SyntaxNode<Language>;
        #[allow(unused)]
        type SyntaxToken = rowan::SyntaxToken<Language>;
        #[allow(unused)]
        type SyntaxElement = rowan::NodeOrToken<SyntaxNode, SyntaxToken>;

        fn print(f: &mut fmt::Formatter<'_>, indent: usize, element: SyntaxElement) -> fmt::Result {
            let kind: SyntaxKind = element.kind().into();
            print!("{:indent$}", "", indent = indent);
            match element {
                rowan::NodeOrToken::Node(node) => {
                    writeln!(f, "- {:?}@{:?}", kind, node.text_range())?;
                    for child in node.children_with_tokens() {
                        print(f, indent + 2, child)?;
                    }
                    Ok(())
                }

                rowan::NodeOrToken::Token(token) => {
                    writeln!(
                        f,
                        "- {:?}@{:?} {:?}",
                        kind,
                        token.text_range(),
                        token.text()
                    )
                }
            }
        }

        print(f, 0, self.ast.clone().into())
    }
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

    pub fn parse(mut self) -> SyntaxTree {
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
                Some(SyntaxKind::Fragment) => {
                    if self.parse_fragment().is_err() {
                        panic!("could not parse fragment");
                    }
                }
                Some(SyntaxKind::Directive) => {
                    if self.parse_directive().is_err() {
                        panic!("could not parse directive");
                    }
                }
                Some(_) => break,
            }
        }

        self.builder.finish_node();

        SyntaxTree {
            ast: rowan::SyntaxNode::new_root(self.builder.finish()),
            errors: self.errors,
        }
    }

    // See: https://spec.graphql.org/June2018/#sec-Language.Fragments
    // 
    // ```txt
    // FragmentDefinition
    //     fragment FragmentName TypeCondition Directives(opt) SelectionSet
    // ```
    fn parse_fragment(&mut self) -> Result<(), ()> {
        self.builder.start_node(SyntaxKind::Fragment.into());
        self.bump();
        self.parse_whitespace();
        self.parse_fragment_name()?;

        // TODO(lrlna): parse TypeCondition, Directives, SelectionSet

        Ok(())
    }

    // See: https://spec.graphql.org/June2018/#FragmentName
    //
    // ```txt
    // FragmentName
    //     Name *but not* on
    // ```
    fn parse_fragment_name(&mut self) -> Result<(), ()> {
        match self.peek() {
            Some(SyntaxKind::Node) => {
                if self.peek_data().unwrap() == "on" {
                    return Err(());
                }
                self.bump();
                Ok(())
            },
            _ => return Err(()),
        }
    }

    // See: https://spec.graphql.org/June2018/#DirectiveDefinition
    // 
    // ```txt
    // DirectiveDefinition
    //     Description(opt) directive @ Name ArgumentsDefinition(opt) on DirectiveLocations
    // ```
    fn parse_directive(&mut self) -> Result<(), ()> {
        self.builder.start_node(SyntaxKind::Directive.into());
        // TODO(lrlna): parse Description
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
    
    pub fn peek_data(&self) -> Option<&String> {
        self.tokens.last().map(|(_, s)| s)
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

        println!("{:?}", parser.parse());
    }
}

