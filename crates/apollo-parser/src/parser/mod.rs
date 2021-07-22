use std::cell::RefCell;
use std::rc::Rc;

use crate::lexer;
use crate::lexer::Lexer;
use crate::lexer::Location;
use crate::TokenKind;

pub use generated::syntax_kind::SyntaxKind;
pub use language::{
    SyntaxElement, SyntaxElementChildren, SyntaxNode, SyntaxNodeChildren, SyntaxToken,
};
pub use syntax_tree::SyntaxTree;

pub(crate) use language::GraphQLLanguage;
pub(crate) use parse_directive::parse_directive;
pub(crate) use parse_directive_locations::parse_directive_locations;
pub(crate) use parse_fragment::parse_fragment;
pub(crate) use parse_fragment_name::parse_fragment_name;
pub(crate) use parse_input_value_definitions::parse_input_value_definitions;
pub(crate) use parse_name::parse_name;
pub(crate) use syntax_tree::SyntaxTreeBuilder;

mod generated;
mod language;
mod parse_directive;
mod parse_directive_locations;
mod parse_fragment;
mod parse_fragment_name;
mod parse_input_value_definitions;
mod parse_name;
mod syntax_tree;

/// Parse text into an AST.
#[derive(Debug)]
pub struct Parser {
    /// input tokens, including whitespace,
    /// in *reverse* order.
    tokens: Vec<lexer::Token>,
    /// the in-progress tree.
    builder: Rc<RefCell<SyntaxTreeBuilder>>,
    /// the list of syntax errors we've accumulated
    /// so far.
    errors: Vec<crate::Error>,
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
            builder: Rc::new(RefCell::new(SyntaxTreeBuilder::new())),
            errors,
        }
    }

    pub fn parse(mut self) -> SyntaxTree {
        let guard = self.start_node(SyntaxKind::DOCUMENT);

        loop {
            match self.peek() {
                None => break,
                Some(TokenKind::Fragment) => {
                    if parse_fragment(&mut self).is_err() {
                        panic!("could not parse fragment")
                        // self.errors.push(Error::with_loc("could not parse fragment".into(), self.peek_data().unwrap(), self.peek_loc().unwrap()));
                    }
                }
                Some(TokenKind::Directive) => {
                    if parse_directive(&mut self).is_err() {
                        panic!("could not parse directive");
                    }
                }
                Some(_) => break,
            }
        }

        guard.finish_node();

        let builder = Rc::try_unwrap(self.builder)
            .expect("More than one reference to builder left")
            .into_inner();
        builder.finish(self.errors)
    }

    pub fn bump(&mut self, kind: SyntaxKind) {
        let token = self.tokens.pop().unwrap();
        self.builder.borrow_mut().token(kind, token.data());
    }

    pub fn start_node(&mut self, kind: SyntaxKind) -> NodeGuard {
        self.builder.borrow_mut().start_node(kind);
        NodeGuard::new(self.builder.clone())
    }

    pub fn peek(&self) -> Option<TokenKind> {
        self.tokens.last().map(|token| token.kind().into())
    }

    pub fn peek_data(&self) -> Option<String> {
        self.tokens.last().map(|token| token.data().to_string())
    }

    pub fn peek_loc(&self) -> Option<Location> {
        self.tokens.last().map(|token| token.loc())
    }
}

#[must_use]
pub struct NodeGuard {
    builder: Rc<RefCell<SyntaxTreeBuilder>>,
}

impl NodeGuard {
    fn new(builder: Rc<RefCell<SyntaxTreeBuilder>>) -> Self {
        Self { builder }
    }

    pub(crate) fn finish_node(self) {
        drop(self);
    }
}

impl Drop for NodeGuard {
    fn drop(&mut self) {
        self.builder.borrow_mut().finish_node();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn smoke_fragment() {
        let input = "fragment friendFields on User {
            id name profilePic(size: 5.0)
        }";
        let parser = Parser::new(input);
        println!("{:?}", parser.parse());
    }

    #[test]
    fn smoke_directive() {
        let input = "directive @example(isTreat: Boolean, treatKind: String) on FIELD | MUTATION";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }

    #[test]
    fn smoke_directive_with_errors() {
        let input =
            "directive ø @example(isTreat: ø Boolean, treatKind: String) on FIELD | MUTATION";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }
}
