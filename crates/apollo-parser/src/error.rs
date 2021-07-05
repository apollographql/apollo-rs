use std::fmt;

use crate::Location;

/// An `Error` type for operations performed in this crate.
#[derive(Clone)]
pub struct Error {
    pub(crate) message: String,
    pub(crate) data: String,
    pub(crate) loc: Location,
}

impl Error {
    /// Create a new instance of `Error`.
    pub fn new(message: String, data: String) -> Self {
        Self {
            message,
            data,
            loc: Location::new(0),
        }
    }

    /// Create a new instance of `Error` with a `Location`.
    pub fn with_loc(message: String, data: String, loc: Location) -> Self {
        Self { message, data, loc }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start = self.loc.index;
        let end = self.loc.index + self.data.len();

        write!(f, "ERROR@{}:{} {:?}", start, end, self.message)
    }
}
