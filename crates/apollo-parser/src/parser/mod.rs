use rowan::GreenNodeBuilder;

use crate::lexer;
use crate::lexer::Lexer;
use crate::lexer::Location;
use crate::TokenKind;

use language::Language;
pub use syntax_tree::SyntaxTree;

mod language;
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
                    if self.parse_fragment().is_err() {
                        panic!("could not parse fragment")
                        // self.errors.push(Error::with_loc("could not parse fragment".into(), self.peek_data().unwrap(), self.peek_loc().unwrap()));
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
        // self.parse_whitespace();
        self.parse_fragment_name()?;

        // TODO(lrlna): parse TypeCondition, Directives, SelectionSet
        self.builder.finish_node();
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
                    // fragment name cannot have "on" as part of its definition
                    return Err(());
                }
                self.bump();
                Ok(())
            }
            // missing fragment name
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
        // self.parse_whitespace();

        match self.peek() {
            Some(TokenKind::At) => self.bump(),
            // missing directive name
            _ => return Err(()),
        }
        match self.peek() {
            Some(TokenKind::Node) => self.bump(),
            // missing directive name
            _ => return Err(()),
        }

        match self.peek() {
            Some(TokenKind::LParen) => {
                self.bump();
                self.parse_input_value_definitions(false)?;
                match self.peek() {
                    Some(TokenKind::RParen) => self.bump(),
                    // missing a closing RParen
                    _ => return Err(()),
                }

                match self.peek() {
                    Some(TokenKind::On) => self.bump(),
                    // missing directive locations in directive definition
                    _ => return Err(()),
                }
            }
            Some(TokenKind::On) => self.bump(),
            // missing directive locations in directive definition
            _ => return Err(()),
        }

        self.parse_directive_locations(false)?;
        self.builder.finish_node();
        Ok(())
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
    // See: https://spec.graphql.org/June2018/#InputValueDefinition
    //
    // ```txt
    // InputValueDefinition
    //     Description(opt) Name : Type DefaultValue(opt) Directives(const/opt)
    // ```
    fn parse_input_value_definitions(&mut self, is_input: bool) -> Result<(), ()> {
        // TODO: parse description
        // TODO: parse default value
        // TODO: parse directives
        match self.peek() {
            // Name
            Some(TokenKind::Node) => {
                self.bump();
                match self.peek() {
                    // Colon
                    Some(TokenKind::Colon) => {
                        self.bump();
                        match self.peek() {
                            // Type
                            Some(TokenKind::Node) => {
                                self.bump();
                                match self.peek() {
                                    Some(_) => self.parse_input_value_definitions(true),
                                    _ => Ok(()),
                                }
                            }
                            _ => return Err(()),
                        }
                    }
                    _ => return Err(()),
                }
            }
            Some(TokenKind::Comma) => {
                self.bump();
                self.parse_input_value_definitions(is_input)
            }
            _ => {
                // if we already have an input, can proceed without returning an error
                if is_input {
                    Ok(())
                } else {
                    // if there is no input, and a LPAREN was supplied, send an error
                    return Err(());
                }
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
