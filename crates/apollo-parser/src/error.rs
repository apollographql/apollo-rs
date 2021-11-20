use std::fmt;

/// An `Error` type for operations performed in this crate.
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

    /// Get a reference to the error's data.
    pub fn data(&self) -> &str {
        self.data.as_ref()
    }

    /// Get a reference to the error's index.
    pub fn index(&self) -> usize {
        self.index
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
