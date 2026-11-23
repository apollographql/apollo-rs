use crate::Error;

/// Peekable iterator over a char sequence.
#[derive(Debug, Clone)]
pub(crate) struct Cursor<'a> {
    index: usize,
    pub(super) offset: usize,
    pub(super) source: &'a str,
    /// Byte position of the next unread character.
    next: usize,
    pending: Option<char>,
    pub(crate) err: Option<Error>,
}

impl<'a> Cursor<'a> {
    pub(crate) fn new(input: &'a str) -> Cursor<'a> {
        Cursor {
            index: 0,
            offset: 0,
            pending: None,
            source: input,
            next: 0,
            err: None,
        }
    }
}

impl<'a> Cursor<'a> {
    fn next_char(&mut self) -> Option<(usize, char)> {
        let pos = self.next;
        let byte = *self.source.as_bytes().get(pos)?;
        let c = if byte.is_ascii() {
            byte as char
        } else {
            self.source[pos..].chars().next()?
        };
        self.next = pos + c.len_utf8();
        Some((pos, c))
    }

    /// Skips ahead so the next character read is at `pos`. `pos` must be a
    /// char boundary at or after the current read position. Does nothing
    /// while a character is pending, since that character sits before the
    /// read position and would otherwise be dropped.
    fn skip_to(&mut self, pos: usize) {
        if self.pending.is_some() {
            return;
        }
        debug_assert!(pos >= self.next && self.source.is_char_boundary(pos));
        self.next = pos;
    }

    fn skip_while(&mut self, pred: impl Fn(u8) -> bool) {
        let rest = &self.source.as_bytes()[self.next..];
        let len = rest.iter().position(|&b| !pred(b)).unwrap_or(rest.len());
        self.skip_to(self.next + len);
    }

    pub(crate) fn skip_name_continue(&mut self) {
        self.skip_while(|b| b.is_ascii_alphanumeric() || b == b'_');
    }

    /// Only the ASCII part of whitespace is skipped here, a BOM goes through
    /// the regular state machine.
    pub(crate) fn skip_ascii_whitespace(&mut self) {
        self.skip_while(|b| matches!(b, b' ' | b'\t' | b'\n' | b'\r'));
    }

    pub(crate) fn skip_comment_body(&mut self) {
        let rest = &self.source.as_bytes()[self.next..];
        let len = memchr::memchr2(b'\n', b'\r', rest).unwrap_or(rest.len());
        self.skip_to(self.next + len);
    }

    /// Skips string content that needs no special handling, stopping before a
    /// quote, a backslash or a line terminator.
    pub(crate) fn skip_string_body(&mut self) {
        let rest = &self.source.as_bytes()[self.next..];
        let mut len = memchr::memchr3(b'"', b'\\', b'\n', rest).unwrap_or(rest.len());
        // A carriage return is rare in single-line strings, so it gets its own
        // scan over the already narrowed range instead of slowing down the
        // main search.
        if let Some(cr) = memchr::memchr(b'\r', &rest[..len]) {
            len = cr;
        }
        self.skip_to(self.next + len);
    }

    pub(crate) fn skip_block_string_body(&mut self) {
        let rest = &self.source.as_bytes()[self.next..];
        let len = memchr::memchr2(b'"', b'\\', rest).unwrap_or(rest.len());
        self.skip_to(self.next + len);
    }

    /// Current place (index) in the cursor.
    pub(crate) fn index(&self) -> usize {
        self.index
    }

    /// Return true if the current state is pending.
    pub(crate) fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Moves to the next character.
    pub(crate) fn prev_str(&mut self) -> &'a str {
        let slice = &self.source[self.index..self.offset];

        self.index = self.offset;
        self.pending = self
            .source
            .get(self.offset..)
            .and_then(|subslice| subslice.chars().next());

        slice
    }

    /// Moves to the next character.
    pub(crate) fn current_str(&mut self) -> &'a str {
        self.pending = None;

        if let Some((pos, next)) = self.next_char() {
            let current = self.index;

            self.index = pos;
            self.offset = pos;
            self.pending = Some(next);

            self.source.get(current..pos)
        } else {
            let current = self.index;
            self.index = self.source.len() - 1;

            self.source.get(current..)
        }
        .unwrap()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self) -> Option<char> {
        if self.pending.is_some() {
            return self.pending.take();
        }

        if self.offset == self.source.len() {
            return None;
        }

        let (pos, c) = self.next_char()?;
        self.offset = pos;

        Some(c)
    }

    /// Moves to the next character.
    pub(crate) fn eatc(&mut self, c: char) -> bool {
        if self.pending.is_some() {
            panic!("dont call eatc when a character is pending");
        }

        if let Some((pos, c_in)) = self.next_char() {
            self.offset = pos;

            if c_in == c {
                return true;
            }

            self.pending = Some(c_in);
        }

        false
    }

    /// Get current error object in the cursor.
    pub(crate) fn err(&mut self) -> Option<Error> {
        self.err.clone()
    }

    /// Drains the current pending characters.
    pub(crate) fn drain(&mut self) -> &'a str {
        self.pending = None;
        let start = self.index;
        self.index = self.source.len() - 1;

        self.source.get(start..=self.index).unwrap()
    }

    /// Add error object to the cursor.
    pub(crate) fn add_err(&mut self, err: Error) {
        self.err = Some(err)
    }
}
