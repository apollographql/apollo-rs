mod generated;
mod language;
mod syntax_tree;
mod token_text;

pub(crate) mod grammar;

use std::{cell::RefCell, rc::Rc};

use crate::{lexer::Lexer, Error, LimitTracker, Token, TokenKind};

pub use generated::syntax_kind::SyntaxKind;
pub use language::{SyntaxElement, SyntaxNode, SyntaxNodeChildren, SyntaxNodePtr, SyntaxToken};
pub use syntax_tree::SyntaxTree;

// pub(crate) use language::GraphQLLanguage;
pub(crate) use syntax_tree::SyntaxTreeBuilder;
pub(crate) use token_text::TokenText;

/// Parse GraphQL schemas or queries into a typed CST.
///
/// ## Example
///
/// The API to parse a query or a schema is the same, as the parser currently
/// accepts a `&str`. Here is an example of parsing a query:
/// ```rust
/// use apollo_parser::Parser;
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// // Create a new instance of a parser given a query above.
/// let parser = Parser::new(query);
/// // Parse the query, and return a SyntaxTree.
/// let cst = parser.parse();
/// // Check that are no errors. These are not part of the CST.
/// assert_eq!(0, cst.errors().len());
///
/// // Get the document root node
/// let doc = cst.document();
/// // ... continue
/// ```
///
/// Here is how you'd parse a schema:
/// ```rust
/// use apollo_parser::Parser;
/// let core_schema = r#"
/// schema @core(feature: "https://specs.apollo.dev/join/v0.1") {
///   query: Query
///   mutation: Mutation
/// }
///
/// enum join__Graph {
///   ACCOUNTS @join__graph(name: "accounts")
/// }
/// "#;
/// let parser = Parser::new(core_schema);
/// let cst = parser.parse();
///
/// assert_eq!(0, cst.errors().len());
///
/// let document = cst.document();
/// ```
#[derive(Debug)]
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    /// Store one lookahead token so we don't need to reparse things as much.
    current_token: Option<Token<'a>>,
    /// The in-progress tree.
    builder: Rc<RefCell<SyntaxTreeBuilder>>,
    /// Ignored tokens that should be added to the tree.
    ignored: Vec<Token<'a>>,
    /// The list of syntax errors we've accumulated so far.
    errors: Vec<crate::Error>,
    /// The limit to apply to parsing.
    recursion_limit: LimitTracker,
    /// Accept parsing errors?
    accept_errors: bool,
}

impl<'a> Parser<'a> {
    /// Create a new instance of a parser given an input string.
    pub fn new(input: &'a str) -> Self {
        let lexer = Lexer::new(input);

        Self {
            lexer,
            current_token: None,
            builder: Rc::new(RefCell::new(SyntaxTreeBuilder::new())),
            ignored: vec![],
            errors: Vec::new(),
            recursion_limit: Default::default(),
            accept_errors: true,
        }
    }

    /// Configure the recursion limit to use while parsing.
    pub fn recursion_limit(mut self, recursion_limit: usize) -> Self {
        self.recursion_limit = LimitTracker::new(recursion_limit);
        self
    }

    /// Configure the limit on the number of tokens to parse. If an input document
    /// is too big, parsing will be aborted.
    ///
    /// By default, there is no limit.
    pub fn token_limit(mut self, token_limit: usize) -> Self {
        self.lexer = self.lexer.with_limit(token_limit);
        self
    }

    /// Parse the current tokens.
    pub fn parse(mut self) -> SyntaxTree {
        grammar::document::document(&mut self);

        let builder = Rc::try_unwrap(self.builder)
            .expect("More than one reference to builder left")
            .into_inner();
        builder.finish(self.errors, self.recursion_limit, self.lexer.limit_tracker)
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

    /// Consume a token and add it to the syntax tree. Queue any ignored tokens that follow.
    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        self.eat(kind);
        self.skip_ignored();
    }

    /// Consume and skip ignored tokens from the lexer.
    pub(crate) fn skip_ignored(&mut self) {
        while let Some(TokenKind::Comment | TokenKind::Whitespace | TokenKind::Comma) = self.peek()
        {
            let token = self.pop();
            self.ignored.push(token);
        }
    }

    /// Push skipped ignored tokens to the current node.
    pub(crate) fn push_ignored(&mut self) {
        let tokens = std::mem::take(&mut self.ignored);
        for token in tokens {
            let syntax_kind = match token.kind {
                TokenKind::Comment => SyntaxKind::COMMENT,
                TokenKind::Whitespace => SyntaxKind::WHITESPACE,
                TokenKind::Comma => SyntaxKind::COMMA,
                _ => unreachable!(),
            };
            self.push_token(syntax_kind, token);
        }
    }

    /// Get current token's data.
    pub(crate) fn current(&mut self) -> Option<&Token> {
        self.peek_token()
    }

    /// Consume a token from the lexer and add it to the syntax tree.
    fn eat(&mut self, kind: SyntaxKind) {
        self.push_ignored();
        if self.current().is_none() {
            return;
        }

        let token = self.pop();
        self.push_token(kind, token);
    }

    /// Create a parser limit error and push it into the error vector.
    ///
    /// Note: After a limit error is pushed, any further errors pushed
    /// are silently discarded.
    pub(crate) fn limit_err<S: Into<String>>(&mut self, message: S) {
        let current = if let Some(current) = self.current() {
            current
        } else {
            return;
        };
        // this needs to be the computed location
        let err = Error::limit(message, current.index());
        self.push_err(err);
        self.accept_errors = false;
    }

    /// Create a parser error at a given location and push it into the error vector.
    pub(crate) fn err_at_token(&mut self, current: &Token, message: &str) {
        let err = if current.kind == TokenKind::Eof {
            Error::eof(message, current.index())
        } else {
            // this needs to be the computed location
            Error::with_loc(message, current.data().to_string(), current.index())
        };
        self.push_err(err);
    }

    /// Create a parser error at the current location and push it into the error vector.
    pub(crate) fn err(&mut self, message: &str) {
        let current = if let Some(current) = self.current() {
            current
        } else {
            return;
        };
        let err = if current.kind == TokenKind::Eof {
            Error::eof(message, current.index())
        } else {
            // this needs to be the computed location
            Error::with_loc(message, current.data().to_string(), current.index())
        };
        self.push_err(err);
    }

    /// Create a parser error at the current location and eat the responsible token.
    pub(crate) fn err_and_pop(&mut self, message: &str) {
        self.push_ignored();
        if self.current().is_none() {
            return;
        }

        let current = self.pop();
        let err = if current.kind == TokenKind::Eof {
            Error::eof(message, current.index())
        } else {
            // this needs to be the computed location
            Error::with_loc(message, current.data().to_string(), current.index())
        };

        // Keep the error in the parse tree for position information
        self.push_token(SyntaxKind::ERROR, current);
        self.push_err(err);

        // we usually skip ignored tokens after we pop each token, so make sure we also do
        // this when we create an error and pop.
        self.skip_ignored();
    }

    /// Consume the next token if it is `kind` or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, token: TokenKind, kind: SyntaxKind) {
        let current = if let Some(current) = self.current() {
            current
        } else {
            return;
        };
        let is_eof = current.kind == TokenKind::Eof;
        // TODO(@goto-bus-stop) this allocation is only required if we have an
        // error, but has to be done eagerly here as the &str reference gets
        // invalidated by `self.at()`. Can we avoid that?
        let data = current.data().to_string();
        let index = current.index();

        if self.at(token) {
            self.bump(kind);
            return;
        }

        let err = if is_eof {
            let message = format!("expected {kind:?}, got EOF");
            Error::eof(message, index)
        } else {
            let message = format!("expected {kind:?}, got {data}");
            Error::with_loc(message, data, index)
        };

        self.push_err(err);
    }

    /// Push an error to parser's error Vec.
    pub(crate) fn push_err(&mut self, err: crate::error::Error) {
        // If the parser has reached a limit, self.accept_errors will
        // be set to false so that we do not push any more errors.
        //
        // This is because the limit activation will result
        // in an early termination which will cause the parser to
        // report "errors" which aren't really errors and thus
        // must be ignored.
        if self.accept_errors {
            self.errors.push(err);
        }
    }

    /// Gets the next token from the lexer.
    fn next_token(&mut self) -> Option<Token<'a>> {
        for res in &mut self.lexer {
            match res {
                Err(err) => {
                    if err.is_limit() {
                        self.accept_errors = false;
                    }
                    self.errors.push(err);
                }
                Ok(token) => {
                    return Some(token);
                }
            }
        }

        None
    }

    /// Consume a token from the lexer.
    pub(crate) fn pop(&mut self) -> Token<'a> {
        if let Some(token) = self.current_token.take() {
            return token;
        }

        self.next_token()
            .expect("Could not pop a token from the lexer")
    }

    /// Insert a token into the syntax tree.
    pub(crate) fn push_token(&mut self, kind: SyntaxKind, token: Token) {
        self.builder.borrow_mut().token(kind, token.data())
    }

    /// Start a node and make it current.
    ///
    /// This also creates a NodeGuard under the hood that will automatically
    /// close the node(via Drop) when the guard goes out of scope.
    /// This allows for us to not have to always close nodes when we are parsing
    /// tokens.
    pub(crate) fn start_node(&mut self, kind: SyntaxKind) -> NodeGuard {
        self.push_ignored();

        self.builder.borrow_mut().start_node(kind);
        let guard = NodeGuard::new(self.builder.clone());
        self.skip_ignored();

        guard
    }

    /// Set a checkpoint for *maybe* wrapping the following parse tree in some
    /// other node.
    pub(crate) fn checkpoint_node(&mut self) -> Checkpoint {
        // We may start a new node here in the future, so let's process
        // our preceding whitespace first
        self.push_ignored();

        let checkpoint = self.builder.borrow().checkpoint();
        Checkpoint::new(self.builder.clone(), checkpoint)
    }

    /// Peek the next Token and return its TokenKind.
    pub(crate) fn peek(&mut self) -> Option<TokenKind> {
        self.peek_token().map(|token| token.kind())
    }

    /// Peek the next Token and return it.
    pub(crate) fn peek_token(&mut self) -> Option<&Token> {
        if self.current_token.is_none() {
            self.current_token = self.next_token();
        }
        self.current_token.as_ref()
    }

    /// Peek Token `n` and return it.
    pub(crate) fn peek_token_n(&self, n: usize) -> Option<Token> {
        self.peek_n_inner(n)
    }

    /// Peek Token `n` and return its TokenKind.
    pub(crate) fn peek_n(&self, n: usize) -> Option<TokenKind> {
        self.peek_n_inner(n).map(|token| token.kind())
    }

    fn peek_n_inner(&self, n: usize) -> Option<Token> {
        self.current_token
            .iter()
            .cloned()
            .map(Result::Ok)
            .chain(self.lexer.clone())
            .filter_map(Result::ok)
            .filter(|token| !matches!(token.kind(), TokenKind::Whitespace | TokenKind::Comment))
            .nth(n - 1)
    }

    /// Peek next Token's `data` property.
    pub(crate) fn peek_data(&mut self) -> Option<String> {
        self.peek_token().map(|token| token.data().to_string())
    }

    /// Peek `n` Token's `data` property.
    pub(crate) fn peek_data_n(&self, n: usize) -> Option<String> {
        self.peek_token_n(n).map(|token| token.data().to_string())
    }
}

/// A wrapper around the SyntaxTreeBuilder used to self-close nodes.
///
/// When the NodeGuard goes out of scope, it automatically runs `finish_node()`
/// on the SyntaxTreeBuilder. This ensures that nodes are not forgotten to be
/// closed.
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

/// A rowan Checkpoint that can self-close the new wrapper node if required.
pub(crate) struct Checkpoint {
    builder: Rc<RefCell<SyntaxTreeBuilder>>,
    checkpoint: rowan::Checkpoint,
}

impl Checkpoint {
    fn new(builder: Rc<RefCell<SyntaxTreeBuilder>>, checkpoint: rowan::Checkpoint) -> Self {
        Self {
            builder,
            checkpoint,
        }
    }

    /// Wrap the nodes that were parsed since setting this checkpoint in a new parent node of kind
    /// `kind`. Returns a NodeGuard that when dropped, finishes this new parent node. More children
    /// can be added to this new node in the mean time.
    pub(crate) fn wrap_node(self, kind: SyntaxKind) -> NodeGuard {
        self.builder.borrow_mut().wrap_node(self.checkpoint, kind);
        NodeGuard::new(self.builder)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Error, Parser};
    use expect_test::expect;

    #[test]
    fn limited_mid_node() {
        let source = r#"
            type Query {
                field(arg1: Int, arg2: Int, arg3: Int, arg4: Int, arg5: Int, arg6: Int): Int
            }
        "#;
        let parser = Parser::new(source)
            // Make it stop inside the arguments list
            .token_limit(18);
        let tree = parser.parse();
        let mut errors = tree.errors();
        assert_eq!(
            errors.next(),
            Some(&Error::limit("token limit reached, aborting lexing", 65))
        );
        assert_eq!(errors.next(), None);
    }

    #[test]
    fn multiple_limits() {
        let source = r#"
            query {
                a {
                    a {
                        a {
                            a
                        }
                    }
                }
            }
        "#;

        let parser = Parser::new(source).recursion_limit(10).token_limit(22);
        let cst = parser.parse();
        let errors = cst.errors().collect::<Vec<_>>();
        assert_eq!(
            errors,
            &[&Error::limit("token limit reached, aborting lexing", 170),]
        );

        let parser = Parser::new(source).recursion_limit(3).token_limit(200);
        let cst = parser.parse();
        let errors = cst.errors().collect::<Vec<_>>();
        assert_eq!(errors, &[&Error::limit("parser limit(3) reached", 121),]);
    }

    #[test]
    fn syntax_errors_and_limits() {
        // Syntax errors before and after the limit
        let source = r#"
            type Query {
                field(arg1: Int, missing_arg): Int
                # limit reached here
                field2: !String
            } and then some garbage
        "#;
        let parser = Parser::new(source).token_limit(22);
        let cst = parser.parse();
        let mut errors = cst.errors();
        assert_eq!(
            errors.next(),
            Some(&Error::with_loc("expected a Name", ")".to_string(), 70))
        );
        // index 113 is immediately after the comment, before the newline
        assert_eq!(
            errors.next(),
            Some(&Error::limit("token limit reached, aborting lexing", 113))
        );
        assert_eq!(errors.next(), None);

        let tree = expect![[r##"
            DOCUMENT@0..113
              WHITESPACE@0..13 "\n            "
              OBJECT_TYPE_DEFINITION@13..76
                type_KW@13..17 "type"
                WHITESPACE@17..18 " "
                NAME@18..23
                  IDENT@18..23 "Query"
                WHITESPACE@23..24 " "
                FIELDS_DEFINITION@24..76
                  L_CURLY@24..25 "{"
                  WHITESPACE@25..42 "\n                "
                  FIELD_DEFINITION@42..76
                    NAME@42..47
                      IDENT@42..47 "field"
                    ARGUMENTS_DEFINITION@47..71
                      L_PAREN@47..48 "("
                      INPUT_VALUE_DEFINITION@48..57
                        NAME@48..52
                          IDENT@48..52 "arg1"
                        COLON@52..53 ":"
                        WHITESPACE@53..54 " "
                        NAMED_TYPE@54..57
                          NAME@54..57
                            IDENT@54..57 "Int"
                      COMMA@57..58 ","
                      WHITESPACE@58..59 " "
                      INPUT_VALUE_DEFINITION@59..70
                        NAME@59..70
                          IDENT@59..70 "missing_arg"
                      R_PAREN@70..71 ")"
                    COLON@71..72 ":"
                    WHITESPACE@72..73 " "
                    NAMED_TYPE@73..76
                      NAME@73..76
                        IDENT@73..76 "Int"
              WHITESPACE@76..93 "\n                "
              COMMENT@93..113 "# limit reached here"
        "##]];
        tree.assert_eq(&format!("{:#?}", cst.document().syntax));
    }

    #[test]
    fn tree_with_syntax_errors() {
        use crate::cst::Definition;

        // Some arbitrary token spam in incorrect places--this test uses
        // valid tokens only
        let source = r#"
            garbage type Query implements X {
                field(arg: Int): Int
            } garbage :,, (|) interface X {}
        "#;
        let cst = Parser::new(source).parse();

        let mut definitions = cst.document().definitions();
        let query_def = definitions.next().unwrap();
        let interface_def = definitions.next().unwrap();
        assert_eq!(definitions.next(), None);
        assert!(matches!(query_def, Definition::ObjectTypeDefinition(_)));
        assert!(matches!(
            interface_def,
            Definition::InterfaceTypeDefinition(_)
        ));
    }

    #[test]
    fn token_limit() {
        let cst = Parser::new("type Query { a a a a a a a a a }")
            .token_limit(100)
            .parse();
        // token count includes EOF token.
        assert_eq!(cst.token_limit().high, 26);
    }
}
