/// A location in the stream.
// TODO lrlna: Am I needed?
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Location {
    pub(crate) index: usize,
}

impl Location {
    pub fn new(index: usize) -> Self {
        Self { index }
    }
}
