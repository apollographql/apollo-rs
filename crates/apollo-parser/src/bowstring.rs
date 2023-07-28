use triomphe::ThinArc;

/// A string for archery.
///
/// This is similar to `Arc<str>` except:
///
/// * With thin pointers (the length is stored in the heap allocation)
/// * Without support for weak references (one less counter stored)
///
/// Based on [`triomphe`](https://crates.io/crates/triomphe).
#[derive(Clone)]
pub struct BowString {
    // Invariant: must be well-formed UTF-8
    bytes: ThinArc<(), u8>,
}

impl BowString {
    #[inline]
    pub fn new(value: &str) -> Self {
        Self {
            bytes: ThinArc::from_header_and_slice((), value.as_bytes()),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.bytes.slice) }
    }
}

impl std::ops::Deref for BowString {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for BowString {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for BowString {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Debug for BowString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for BowString {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Default for BowString {
    #[inline]
    fn default() -> Self {
        Self::new("")
    }
}

impl Eq for BowString {}

impl<Other: AsRef<str>> PartialEq<Other> for BowString {
    #[inline]
    fn eq(&self, other: &Other) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl Ord for BowString {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl<Other: AsRef<str>> PartialOrd<Other> for BowString {
    #[inline]
    fn partial_cmp(&self, other: &Other) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(&other.as_ref())
    }
}

impl std::hash::Hash for BowString {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl std::str::FromStr for BowString {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl From<&'_ str> for BowString {
    #[inline]
    fn from(value: &'_ str) -> Self {
        Self::new(value)
    }
}

impl From<&'_ String> for BowString {
    #[inline]
    fn from(value: &'_ String) -> Self {
        Self::new(value)
    }
}

impl From<String> for BowString {
    #[inline]
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}
