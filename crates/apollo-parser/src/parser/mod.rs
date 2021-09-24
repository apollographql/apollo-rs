use std::cell::RefCell;
use std::rc::Rc;

use crate::create_err;
use crate::lexer::Lexer;
use crate::TokenKind;
use crate::{lexer, Token};

pub use generated::syntax_kind::SyntaxKind;
pub use language::{SyntaxElement, SyntaxElementChildren, SyntaxNodeChildren, SyntaxToken};
pub use syntax_tree::SyntaxTree;

pub(crate) use language::{GraphQLLanguage, SyntaxNode};
pub(crate) use syntax_tree::SyntaxTreeBuilder;
pub(crate) use token_text::TokenText;

mod generated;
pub(crate) mod grammar;
mod language;
mod syntax_tree;
mod token_text;
pub(crate) mod utils;
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
        let lexer = Lexer::new(input);

        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        for s in lexer.tokens().to_owned() {
            tokens.push(s);
        }

        for e in lexer.errors().to_owned() {
            errors.push(e);
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
        grammar::document::document(&mut self);

        let builder = Rc::try_unwrap(self.builder)
            .expect("More than one reference to builder left")
            .into_inner();
        builder.finish(self.errors)
    }

    /// Check if the current token is `kind`.
    pub(crate) fn at(&mut self, token: TokenKind) -> bool {
        if let Some(t) = self.peek() {
            if t == token {
                return true;
            }
            return false;
        }

        false
    }

    /// Consume a token from the lexer, and any ignored tokens that follow it
    /// and add them to the AST.
    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        self.eat(kind);
        self.bump_ignored();
    }

    /// Consume ignored tokens and add them to the AST.
    pub(crate) fn bump_ignored(&mut self) {
        while let Some(TokenKind::Comment | TokenKind::Whitespace) = self.peek() {
            if let Some(TokenKind::Comment) = self.peek() {
                self.bump(SyntaxKind::COMMENT);
            }
            if let Some(TokenKind::Whitespace) = self.peek() {
                self.bump(SyntaxKind::WHITESPACE);
            }
        }
    }

    /// Get current token's data.
    pub(crate) fn current(&mut self) -> String {
        self.peek_data().unwrap_or_else(|| "EOF".to_string())
    }

    /// Consume a token from the lexer and add it to the AST.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) {
        let token = self.tokens.pop().unwrap();
        self.builder.borrow_mut().token(kind, token.data());
    }

    /// Create a parser error and push it into the error vector.
    pub(crate) fn err(&mut self, message: &str) {
        let current = self.current();
        let err = crate::Error::new(message.into(), current);
        self.push_err(err)
    }

    /// Consume the next token if it is `kind` or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, token: TokenKind, kind: SyntaxKind) {
        let current_t = self.current();

        if self.at(token) {
            self.eat(kind);
            return;
        }

        self.push_err(create_err!(
            current_t,
            "expected {:?}, got {}",
            kind,
            current_t,
        ));
    }

    pub(crate) fn push_err(&mut self, err: crate::error::Error) {
        self.errors.push(err);
    }

    /// Consume a token from the lexer.
    pub(crate) fn pop(&mut self) -> Token {
        self.tokens.pop().unwrap()
    }

    /// Insert a token into the AST.
    pub(crate) fn push_ast(&mut self, kind: SyntaxKind, token: Token) {
        self.builder.borrow_mut().token(kind, token.data())
    }

    pub(crate) fn start_node(&mut self, kind: SyntaxKind) -> NodeGuard {
        self.builder.borrow_mut().start_node(kind);
        let guard = NodeGuard::new(self.builder.clone());
        self.bump_ignored();

        guard
    }

    pub(crate) fn peek(&self) -> Option<TokenKind> {
        self.tokens.last().map(|token| token.kind())
    }

    pub(crate) fn peek_n(&self, n: usize) -> Option<TokenKind> {
        let tok = self
            .tokens
            .clone()
            .into_iter()
            .filter(|token| !matches!(token.kind(), TokenKind::Whitespace | TokenKind::Comment))
            .collect::<Vec<Token>>();
        tok.get(tok.len() - n).map(|token| token.kind())
    }

    pub(crate) fn peek_data(&self) -> Option<String> {
        self.tokens.last().map(|token| token.data().to_string())
    }

    pub(crate) fn peek_data_n(&self, n: usize) -> Option<String> {
        let tok = self
            .tokens
            .clone()
            .into_iter()
            .filter(|token| !matches!(token.kind(), TokenKind::Whitespace | TokenKind::Comment))
            .collect::<Vec<Token>>();
        tok.get(tok.len() - n).map(|token| token.data().to_string())
    }
}

#[must_use]
pub(crate) struct NodeGuard {
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
    use indoc::indoc;

    #[test]
    fn smoke_subgraph_test() {
        let input = indoc! { r#"
            schema {
              query: Query
            }

            type Query {
              translate(
                fromLanguage: Language
                toLanguage: Language
                text: String
              ): String
            }

            enum Language {
              EN
              FR
              CH
            }
        "#};
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }
}
