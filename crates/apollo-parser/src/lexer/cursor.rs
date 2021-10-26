use std::str::Chars;

use crate::Error;
/// Peekable iterator over a char sequence.
pub(crate) struct Cursor<'a> {
    chars: Chars<'a>,
    pub(crate) err: Option<Error>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            chars: input.chars(),
            err: None,
        }
    }
}

pub(crate) const EOF_CHAR: char = '\0';

impl<'a> Cursor<'a> {
    /// Returns nth character relative to the current cursor position.
    fn nth_char(&self, n: usize) -> char {
        self.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    /// Peeks the next char in input without consuming.
    pub(crate) fn first(&self) -> char {
        self.nth_char(0)
    }

    /// Peeks the second char in input without consuming.
    pub(crate) fn second(&self) -> char {
        self.nth_char(1)
    }

    /// Checks if there are chars to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.chars.as_str().is_empty()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;

        Some(c)
    }

    /// Get current error object in the cursor.
    pub(crate) fn err(&mut self) -> Option<Error> {
        self.err.clone()
    }

    /// Add error object to the cursor.
    pub(crate) fn add_err(&mut self, err: Error) {
        self.err = Some(err)
    }

    /// Returns a `Chars` iterator over the remaining characters.
    fn chars(&self) -> Chars<'_> {
        self.chars.clone()
    }
}
