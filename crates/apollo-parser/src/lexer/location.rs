/// A location index in the token stream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub(crate) index: usize,
}

impl Location {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
