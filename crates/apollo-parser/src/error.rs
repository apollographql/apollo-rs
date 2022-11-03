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
/// assert_eq!(0, ast.errors().len());
///
/// let doc = ast.document();
/// ```
///
/// ### Diagnostics
///
/// Using something like [miette crate] along with apollo-parser lets you have
/// more visual diagnostics. [miette] and [annotate_snippets] examples guide you
/// through integrating them with apollo-parser. These are useful if you are
/// displaying Errors in a terminal-like environment.
///
/// <img src="https://raw.githubusercontent.com/apollographql/apollo-rs/main/crates/apollo-parser/screenshots/apollo_parser_error.png" alt="A screenshot of an error example produced by using apollo-parser and miette. The ascii display shows a graphql code snippet with line numbers to the left. Under the code sample there is a line pointing to where a value is missing in graphql code">
///
/// [miette]: https://github.com/apollographql/apollo-rs/blob/a7f616454a53dcb8496725ceac6c63eacddefb2c/crates/apollo-parser/examples/miette.rs
/// [annotate_snippets]: https://github.com/apollographql/apollo-rs/blob/a7f616454a53dcb8496725ceac6c63eacddefb2c/crates/apollo-parser/examples/annotate_snippet.rs
/// [miette crate]: https://docs.rs/miette/3.2.0/miette/index.html

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum ErrorData {
    Eof,
    LimitExceeded,
    Text(String),
}

impl ErrorData {
    pub fn len(&self) -> usize {
        match self {
            Self::Eof | Self::LimitExceeded => 0,
            Self::Text(text) => text.len(),
        }
    }
}

impl fmt::Display for ErrorData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eof => write!(f, "EOF"),
            Self::LimitExceeded => Ok(()),
            Self::Text(text) => write!(f, "{}", text),
        }
    }
}

#[derive(PartialEq, Eq, Clone, thiserror::Error)]
#[error("ERROR@{index}:{} {message:?} {data}", .index + .data.len())]
pub struct Error {
    pub(crate) message: String,
    pub(crate) data: ErrorData,
    pub(crate) index: usize,
}

impl Error {
    /// Create a new instance of `Error`.
    pub fn new<S: Into<String>>(message: S, data: String) -> Self {
        Self {
            message: message.into(),
            data: ErrorData::Text(data),
            index: 0,
        }
    }

    /// Create a new instance of `Error` with a `Location`.
    pub fn with_loc<S: Into<String>>(message: S, data: String, index: usize) -> Self {
        Self {
            message: message.into(),
            data: ErrorData::Text(data),
            index,
        }
    }

    pub fn limit<S: Into<String>>(message: S, index: usize) -> Self {
        Self {
            message: message.into(),
            data: ErrorData::LimitExceeded,
            index,
        }
    }

    pub fn eof<S: Into<String>>(message: S, index: usize) -> Self {
        Self {
            message: message.into(),
            data: ErrorData::Eof,
            index,
        }
    }

    /// Get a reference to the error's data. This is usually the token that
    /// `apollo-parser` has found to be lexically or syntactically incorrect.
    pub fn data(&self) -> &str {
        match &self.data {
            ErrorData::Text(text) => text,
            _ => "",
        }
    }

    pub fn is_limit(&self) -> bool {
        matches!(&self.data, ErrorData::LimitExceeded)
    }

    pub fn is_eof(&self) -> bool {
        matches!(&self.data, ErrorData::Eof)
    }

    pub(crate) fn set_data(&mut self, data: String) {
        self.data = ErrorData::Text(data);
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

        write!(
            f,
            "ERROR@{}:{} {:?} {}",
            start, end, self.message, self.data
        )
    }
}
