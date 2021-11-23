use std::fmt;

/// An `Error` type for operations performed in the lexer and the parser.
///
/// Errors get returned alongside the resulting AST if either the lexer or the
/// parser encouter lexical or syntactical errors respectively.
///
/// We encourage you to check for the AST's errors before proceeding to iterate
/// over the AST's nodes:
///
/// ## Example
/// ```rust
/// use apollo_parser::Parser;
///
/// let input = "union SearchResult = Photo | Person | Cat | Dog";
/// let parser = Parser::new(input);
/// let ast = parser.parse();
///
/// assert!(ast.errors().is_empty());
///
/// let doc = ast.document();
/// ```
///
/// `Error` struct does not at the moment implement `Display`. We encourage you
/// to take a look at the [miette] and [annotate_snippets] integration examples
/// if you'd like to display Errors in a terminal-like environment. The
/// `Error`'s API allows for you to configure other ways to render your errors
/// as well if a terminal-like environment is not where your final output goes.
///
/// [miette]: https://github.com/apollographql/apollo-rs/blob/main/crates/apollo-parser/examples/miette.rs
/// [annotate_snippets]: https://github.com/apollographql/apollo-rs/blob/main/crates/apollo-parser/examples/annotate_snippet.rs
#[derive(PartialEq, Eq, Clone)]
pub struct Error {
    pub(crate) message: String,
    pub(crate) data: String,
    pub(crate) index: usize,
}

impl Error {
    /// Create a new instance of `Error`.
    pub fn new<S: Into<String>>(message: S, data: String) -> Self {
        Self {
            message: message.into(),
            data,
            index: 0,
        }
    }

    /// Create a new instance of `Error` with a `Location`.
    pub fn with_loc<S: Into<String>>(message: S, data: String, index: usize) -> Self {
        Self {
            message: message.into(),
            data,
            index,
        }
    }

    /// Get a reference to the error's data. This is usually the token that
    /// `apollo-parser` has found to be lexically or syntactically incorrect.
    pub fn data(&self) -> &str {
        self.data.as_ref()
    }

    /// Get a reference to the error's index. This is where the error begins in
    /// a given input.
    pub fn index(&self) -> usize {
        self.index
    }

    /// Get a reference to the error's message.
    pub fn message(&self) -> &str {
        self.message.as_ref()
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.index;
        let end = self.index + self.data.len();

        if &self.data == "EOF" {
            write!(
                f,
                "ERROR@{}:{} {:?} {}",
                start, start, self.message, self.data
            )
        } else {
            write!(
                f,
                "ERROR@{}:{} {:?} {}",
                start, end, self.message, self.data
            )
        }
    }
}
