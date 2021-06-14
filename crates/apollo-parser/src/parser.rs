use std::fmt;

use rowan::GreenNodeBuilder;

use crate::lexer;
use crate::lexer::Lexer;
use crate::token_kind::TokenKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Language {}
impl rowan::Language for Language {
    type Kind = TokenKind;
    fn kind_from_raw(raw: rowan::SyntaxKind) -> Self::Kind {
        assert!(raw.0 <= TokenKind::Root as u16);
        unsafe { std::mem::transmute::<u16, TokenKind>(raw.0) }
    }
    fn kind_to_raw(kind: Self::Kind) -> rowan::SyntaxKind {
        kind.into()
    }
}

pub struct SyntaxTree {
    ast: rowan::SyntaxNode<Language>,
    errors: Vec<lexer::Error>,
}

impl SyntaxTree {
    /// Get a reference to the syntax tree's errors.
    pub fn errors(&self) -> &Vec<lexer::Error> {
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
            let kind: TokenKind = element.kind().into();
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
    tokens: Vec<lexer::Token>,
    /// the in-progress tree.
    builder: GreenNodeBuilder<'static>,
    /// the list of syntax errors we've accumulated
    /// so far.
    errors: Vec<lexer::Error>,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let lexer = Lexer::new(&input);

        let mut tokens = Vec::new();
        let mut errors = Vec::new();
        
        for s in lexer.tokens().to_owned() {
            match s {
                Ok(t) => tokens.push(t),
                Err(e) => errors.push(e),
            }
        }

        tokens.reverse();
        errors.reverse();

        Self {
            tokens,
            builder: GreenNodeBuilder::new(),
            errors,
        }
    }

    pub fn parse(mut self) -> SyntaxTree {
        self.builder.start_node(TokenKind::Root.into());

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
                Some(TokenKind::Fragment) => {
                    if self.parse_fragment().is_err() {
                        panic!("could not parse fragment");
                    }
                }
                Some(TokenKind::Directive) => {
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
        self.builder.start_node(TokenKind::Fragment.into());
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
            Some(TokenKind::Node) => {
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
        self.builder.start_node(TokenKind::Directive.into());
        // TODO(lrlna): parse Description
        self.bump();
        self.parse_whitespace();

        match self.peek() {
            Some(TokenKind::At) => self.bump(),
            _ => return Err(()),
        }
        match self.peek() {
            Some(TokenKind::Node) => self.bump(),
            _ => return Err(()),
        }

        self.parse_whitespace();
        match self.peek() {
            Some(TokenKind::On) => self.bump(),
            _ => return Err(()),
        }

        self.parse_whitespace();
        match self.peek() {
            Some(TokenKind::Node) => self.bump(),
            _ => return Err(()),
        }
        self.builder.finish_node();
        Ok(())
    }

    pub fn parse_whitespace(&mut self) {
        let mut text = String::new();
        while let Some(TokenKind::Whitespace) = self.peek() {
            let token = self.tokens.pop().unwrap();
            text.push_str(token.data());
        }
        self.builder.token(TokenKind::Whitespace.into(), &text);
    }

    pub fn bump(&mut self) {
        let token = self.tokens.pop().unwrap();
        self.builder.token(token.kind().into(), token.data());
    }

    pub fn peek(&self) -> Option<TokenKind> {
        self.tokens.last().map(|token| token.kind().into())
    }
    
    pub fn peek_data(&self) -> Option<String> {
        self.tokens.last().map(|token| token.data().to_string())
    }

}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    // fn smoke() {
    //     // directive @example on FIELD
    //     let parser = Parser::new(vec![
    //         (TokenKind::Directive, "directive".to_string()),
    //         (TokenKind::Whitespace, " ".to_string()),
    //         (TokenKind::At, "@".to_string()),
    //         (TokenKind::Node, "example".to_string()),
    //         (TokenKind::Whitespace, " ".to_string()),
    //         (TokenKind::On, "on".to_string()),
    //         (TokenKind::Whitespace, " ".to_string()),
    //         (TokenKind::Node, "FIELD".to_string()),
    //     ]);

    //     println!("{:?}", parser.parse());
    // }

    #[test]
    fn smoke_2() {
        // directive @example on FIELD
        let input = "fragment friendFields on User {
            id name profilePic(size: 5.0)
        }";
        // let input = "directive @example on FIELD";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }
}

