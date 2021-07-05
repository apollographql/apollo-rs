use rowan::GreenNodeBuilder;

use crate::lexer;
use crate::lexer::Lexer;
use crate::lexer::Location;
use crate::TokenKind;

pub use generated::syntax_kind::SyntaxKind;
pub use syntax_tree::SyntaxTree;

use language::Language;
pub(crate) use parse_directive::parse_directive;
pub(crate) use parse_fragment::parse_fragment;
pub(crate) use parse_fragment_name::parse_fragment_name;
pub(crate) use parse_input_value_definitions::parse_input_value_definitions;

mod generated;
mod language;
mod parse_directive;
mod parse_fragment;
mod parse_fragment_name;
mod parse_input_value_definitions;
mod syntax_tree;

/// Parse text into an AST.
#[derive(Debug)]
pub struct Parser {
    /// input tokens, including whitespace,
    /// in *reverse* order.
    tokens: Vec<lexer::Token>,
    /// the in-progress tree.
    builder: GreenNodeBuilder<'static>,
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
            builder: GreenNodeBuilder::new(),
            errors,
        }
    }

    pub fn parse(mut self) -> SyntaxTree {
        self.builder.start_node(TokenKind::Root.into());

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

        self.builder.finish_node();

        SyntaxTree {
            ast: rowan::SyntaxNode::new_root(self.builder.finish()),
            errors: self.errors,
        }
    }

    fn parse_directive_locations(&mut self, is_location: bool) -> Result<(), ()> {
        match self.peek() {
            Some(TokenKind::Pipe) => {
                self.bump();
                self.parse_directive_locations(is_location)
            }
            Some(TokenKind::Node) => {
                self.bump();
                match self.peek_data() {
                    Some(_) => return self.parse_directive_locations(true),
                    _ => return Ok(()),
                }
            }
            _ => {
                if !is_location {
                    // missing directive locations in directive definition
                    return Err(());
                }
                Ok(())
            }
        }
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

    pub fn peek_loc(&self) -> Option<Location> {
        self.tokens.last().map(|token| token.loc())
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
}
