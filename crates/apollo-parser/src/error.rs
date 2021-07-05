use std::fmt;

use crate::Location;

#[derive(Clone)]
pub struct Error {
    message: String,
    pub(crate) data: String,
    pub(crate) loc: Location,
}

impl Error {
    pub fn new(message: String, data: String) -> Self {
        Self {
            message,
            data,
            loc: Location::new(0),
        }
    }

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