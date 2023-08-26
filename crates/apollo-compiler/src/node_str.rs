use crate::schema::ComponentOrigin;
use crate::schema::ComponentStr;
use crate::NodeLocation;
use triomphe::ThinArc;

/// Smart string type for names and string values in a GraphQL document
///
/// Like [`Node`][crate::Node] it is thread-safe, reference-counted,
/// and carries an optional source location.
/// It is a thin pointer to a single allocation, with a header followed by string data.
#[derive(Clone)]
pub struct NodeStr(ThinArc<Header, u8>);

type Header = Option<NodeLocation>;

impl NodeStr {
    /// Create a new `NodeStr` parsed from the given source location
    #[inline]
    pub fn new_parsed(value: &str, location: NodeLocation) -> Self {
        Self(ThinArc::from_header_and_slice(
            Some(location),
            value.as_bytes(),
        ))
    }

    /// Create a new `NodeStr` programatically, not parsed from a source file
    #[inline]
    pub fn new_synthetic(value: &str) -> Self {
        Self(ThinArc::from_header_and_slice(None, value.as_bytes()))
    }

    #[inline]
    pub fn to_component(&self, origin: ComponentOrigin) -> ComponentStr {
        ComponentStr {
            origin,
            node: self.clone(),
        }
    }

    #[inline]
    pub fn location(&self) -> Option<&NodeLocation> {
        self.0.header.header.as_ref()
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0.slice) }
    }

    /// Returns wether two `NodeStr`s point to the same allocation
    #[inline]
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.0.as_ptr() == other.0.as_ptr()
    }
}

impl std::hash::Hash for NodeStr {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state) // location not included
    }
}

impl std::ops::Deref for NodeStr {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for NodeStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for NodeStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Debug for NodeStr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for NodeStr {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Eq for NodeStr {}

impl PartialEq for NodeStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_ref() // don’t compare location
    }
}

impl Ord for NodeStr {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for NodeStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other.as_str())
    }
}

impl PartialEq<str> for NodeStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialOrd<str> for NodeStr {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialEq<&'_ str> for NodeStr {
    #[inline]
    fn eq(&self, other: &&'_ str) -> bool {
        self.as_str() == *other
    }
}

impl PartialOrd<&'_ str> for NodeStr {
    #[inline]
    fn partial_cmp(&self, other: &&'_ str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl From<&'_ Self> for NodeStr {
    #[inline]
    fn from(value: &'_ Self) -> Self {
        value.clone()
    }
}
