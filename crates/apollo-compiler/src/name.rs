use crate::diagnostic::CliReport;
use crate::diagnostic::ToCliReport;
use crate::node::ExtensionId;
use crate::parser::FileId;
use crate::parser::LineColumn;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::parser::TaggedFileId;
use crate::symbol::PROBED_MISS;
use crate::symbol::UNINITIALIZED_SYMBOL;
use crate::Node;
use rowan::TextRange;
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::ops::Range;
use std::ptr::NonNull;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::Arc;

/// Create a [`Name`] from a string literal or identifier, checked for validity at compile time.
///
/// A `Name` created this way does not own allocated heap memory or a reference counter,
/// so cloning it is extremely cheap.
///
/// # Examples
///
/// ```
/// use apollo_compiler::name;
///
/// assert_eq!(name!("Query").as_str(), "Query");
/// assert_eq!(name!(Query).as_str(), "Query");
/// ```
///
/// ```compile_fail
/// # use apollo_compiler::name;
/// // error[E0080]: evaluation of constant value failed
/// // assertion failed: ::apollo_compiler::ast::Name::valid_syntax(\"è_é\")
/// let invalid = name!("è_é");
/// ```
#[macro_export]
macro_rules! name {
    ($value: ident) => {
        $crate::name!(stringify!($value))
    };
    ($value: expr) => {{
        const _: () = { assert!($crate::Name::is_valid_syntax($value)) };
        $crate::Name::new_static_unchecked(&$value)
    }};
}

/// A GraphQL [_Name_](https://spec.graphql.org/September2025/#Name) identifier
///
/// Like [`Node`][crate::Node], this string type has cheap `Clone`
/// and carries an optional source location.
///
/// Internally, the string value is either an atomically-reference counted `Arc<str>`
/// or a `&'static str` borrow that lives until the end of the program.
//
// Fields: equivalent to `(UnpackedRepr, Option<SourceSpan>)` but more compact
pub struct Name {
    /// Data pointer of either `Arc<str>::into_raw` (if `tagged_file_id.tag() == TAG_ARC`)
    /// or `&'static str` (if `TAG_STATIC`)
    ptr: NonNull<u8>,
    len: u32,
    start_offset: u32,            // zero if we don’t have a location
    tagged_file_id: TaggedFileId, // `.file_id() == FileId::NONE` means we don’t have a location
    /// The string's global symbol (see `crate::symbol`); 0 = not yet
    /// resolved (names from `const` contexts resolve lazily on first
    /// eq/hash), `PROBED_MISS` = absent from the frozen table.
    symbol: AtomicU32,
    phantom: PhantomData<UnpackedRepr>,
}

#[allow(dead_code)] // only used in PhantomData and static asserts
enum UnpackedRepr {
    Heap(Arc<str>),
    Static(&'static str),
}

/// Tried to create a [`Name`] from a string that is not in valid
/// [GraphQL name](https://spec.graphql.org/September2025/#sec-Names) syntax.
#[derive(Clone, Eq, PartialEq, thiserror::Error)]
#[error("`{name}` is not a valid GraphQL name")]
pub struct InvalidNameError {
    pub name: String,
    pub location: Option<SourceSpan>,
}

const TAG_ARC: bool = true;
const TAG_STATIC: bool = false;

const _: () = {
    // 4 bytes of symbol on top of the former 24-byte layout, padded:
    #[cfg(not(target_family = "wasm"))]
    assert!(size_of::<Name>() == 32);
    #[cfg(target_family = "wasm")]
    assert!(size_of::<Name>() == 24);
    assert!(size_of::<Name>() == size_of::<Option<Name>>());

    // The `unsafe impl`s below are sound since `(tag, ptr, len)` represents `UnpackedRepr`
    const fn assert_send_and_sync<T: Send + Sync>() {}
    assert_send_and_sync::<(UnpackedRepr, u32, TaggedFileId)>();
};

unsafe impl Send for Name {}

unsafe impl Sync for Name {}

impl Name {
    /// Create a new `Name`
    pub fn new(value: &str) -> Result<Self, InvalidNameError> {
        Self::check_valid_syntax(value)?;
        Ok(Self::new_unchecked(value))
    }

    /// Create a new `Name` from a string with static lifetime
    pub fn new_static(value: &'static str) -> Result<Self, InvalidNameError> {
        Self::check_valid_syntax(value)?;
        Ok(Self::new_static_unchecked(value))
    }

    /// Create a new `Name` without [validity checking][Self::is_valid_syntax].
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub fn new_unchecked(value: &str) -> Self {
        match crate::symbol::intern(value) {
            Some((name, symbol)) => {
                let mut digest = Self::new_static_unchecked(name);
                digest.symbol = AtomicU32::new(symbol.get());
                digest
            }
            // table frozen and this string isn't in it
            None => {
                let arc: Arc<str> = Arc::from(value);
                let len = Self::new_len(&arc);
                let ptr = Arc::into_raw(arc).cast_mut().cast();
                // SAFETY: Arc always is non-null
                let ptr = unsafe { NonNull::new_unchecked(ptr) };
                Self {
                    ptr,
                    len,
                    start_offset: 0,
                    tagged_file_id: TaggedFileId::pack(TAG_ARC, FileId::NONE),
                    symbol: AtomicU32::new(PROBED_MISS),
                    phantom: PhantomData,
                }
            }
        }
    }

    /// Create a new `Name` from an `Arc`, without [validity checking][Self::is_valid_syntax].
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub fn from_arc_unchecked(arc: Arc<str>) -> Self {
        let symbol = AtomicU32::new(match crate::symbol::intern(&arc) {
            Some(symbol) => symbol.1.get(),
            None => PROBED_MISS, // table frozen and this string isn't in it
        });
        let len = Self::new_len(&arc);
        let ptr = Arc::into_raw(arc).cast_mut().cast();
        // SAFETY: Arc always is non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self {
            ptr,
            len,
            start_offset: 0,
            tagged_file_id: TaggedFileId::pack(TAG_ARC, FileId::NONE),
            symbol,
            phantom: PhantomData,
        }
    }

    /// Create a new `Name` from a string with static lifetime,
    /// without [validity checking][Self::is_valid_syntax].
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub const fn new_static_unchecked(value: &'static str) -> Self {
        let ptr = value.as_ptr().cast_mut();
        // SAFETY: `&'static str` is always non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self {
            ptr,
            len: Self::new_len(value),
            start_offset: 0,
            tagged_file_id: TaggedFileId::pack(TAG_STATIC, FileId::NONE),
            symbol: AtomicU32::new(0),
            phantom: PhantomData,
        }
    }

    /// The name's global symbol, resolving lazily for names created in
    /// `const` contexts. Returns [`PROBED_MISS`] for names whose string is
    /// not in the frozen table; such names use string-based equality and
    /// hashing. With a frozen table, hit-or-miss is a pure function of the
    /// string, so equal strings always agree.
    #[inline]
    pub(crate) fn symbol(&self) -> u32 {
        #[deny(non_snake_case)]
        match self.symbol.load(Relaxed) {
            UNINITIALIZED_SYMBOL => self.symbol_slow(),
            symbol => symbol,
        }
    }

    #[cold]
    fn symbol_slow(&self) -> u32 {
        let symbol = match crate::symbol::intern(self.as_str()) {
            Some(symbol) => symbol.1.get(),
            None => PROBED_MISS,
        };
        self.symbol.store(symbol, Relaxed);
        symbol
    }

    /// Modifies the given name to add its location in a parsed source file
    pub fn with_location(mut self, location: SourceSpan) -> Self {
        debug_assert_eq!(location.text_range.len(), self.len.into());
        self.start_offset = location.text_range.start().into();
        self.tagged_file_id = TaggedFileId::pack(self.tagged_file_id.tag(), location.file_id);
        self
    }

    const fn new_len(value: &str) -> u32 {
        let len = value.len();
        if len >= (u32::MAX as usize) {
            panic!("Name length overflows 4 GiB")
        }
        len as _
    }

    /// If this node was parsed from a source file, returns the file ID and source span
    /// (start and end byte offsets) within that file.
    pub fn location(&self) -> Option<SourceSpan> {
        let file_id = self.tagged_file_id.file_id();
        if file_id != FileId::NONE {
            Some(SourceSpan {
                file_id,
                text_range: TextRange::at(self.start_offset.into(), self.len.into()),
            })
        } else {
            None
        }
    }

    /// If this string contains a location, convert it to line and column numbers
    pub fn line_column_range(&self, sources: &SourceMap) -> Option<Range<LineColumn>> {
        self.location()?.line_column_range(sources)
    }

    #[allow(clippy::len_without_is_empty)] // GraphQL Name is never empty
    #[inline]
    pub fn len(&self) -> usize {
        self.len as _
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        let slice = NonNull::slice_from_raw_parts(self.ptr, self.len());
        // SAFETY: all constructors set `self.ptr` and `self.len` from valid UTF-8,
        // and we return a lifetime tied to `self`.
        unsafe { std::str::from_utf8_unchecked(slice.as_ref()) }
    }

    /// If this `Name` was created with [`new_static`][Self::new_static]
    /// or the [`name!`][crate::name!] macro, return the string with `'static` lifetime.
    ///
    /// Returns `Some` if and only if [`to_cloned_arc`][Self::to_cloned_arc] returns `None`.
    pub fn as_static_str(&self) -> Option<&'static str> {
        if self.tagged_file_id.tag() == TAG_STATIC {
            let raw_slice = NonNull::slice_from_raw_parts(self.ptr, self.len());
            // SAFETY: the tag indicates `self.ptr` came from `Self::ptr_and_tag_from_static`,
            // so it has the static lifetime and points to valid UTF-8 of the correct length.
            Some(unsafe { std::str::from_utf8_unchecked(raw_slice.as_ref()) })
        } else {
            None
        }
    }

    fn as_arc(&self) -> Option<ManuallyDrop<Arc<str>>> {
        if self.tagged_file_id.tag() == TAG_ARC {
            let raw_slice = NonNull::slice_from_raw_parts(self.ptr, self.len())
                .as_ptr()
                .cast_const();

            // SAFETY:
            //
            // * The tag indicates `self.ptr` came from `Arc::into_raw` in `ptr_and_tag_with_arc`
            // * `Arc::from_raw` normally moves ownership away from the raw pointer,
            //   `ManuallyDrop` counteracts that
            Some(ManuallyDrop::new(unsafe {
                Arc::from_raw(raw_slice as *const str)
            }))
        } else {
            None
        }
    }

    /// If this `Name` contains an `Arc<str>`, return a clone of it (reference count increment)
    ///
    /// Returns `Some` if and only if [`as_static_str`][Self::as_static_str] returns `None`.
    pub fn to_cloned_arc(&self) -> Option<Arc<str>> {
        self.as_arc()
            .map(|manually_drop| Arc::clone(&manually_drop))
    }

    /// Returns whether the given string is a valid
    /// GraphQL [_Name_](https://spec.graphql.org/September2025/#Name).
    pub const fn is_valid_syntax(value: &str) -> bool {
        let bytes = value.as_bytes();
        let Some(&first) = bytes.first() else {
            return false;
        };
        if !Self::is_name_start(first) {
            return false;
        }
        // TODO: iterator when available in const
        let mut i = 1;
        while i < bytes.len() {
            if !Self::is_name_continue(bytes[i]) {
                return false;
            }
            i += 1
        }
        true
    }

    fn check_valid_syntax(value: &str) -> Result<(), InvalidNameError> {
        if Self::is_valid_syntax(value) {
            Ok(())
        } else {
            Err(InvalidNameError {
                name: value.to_owned(),
                location: None,
            })
        }
    }

    /// <https://spec.graphql.org/September2025/#NameStart>
    const fn is_name_start(byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_'
    }

    /// <https://spec.graphql.org/September2025/#NameContinue>
    const fn is_name_continue(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_'
    }

    /// Converts to a [`Node<Name>`] with the given extension ID,
    /// keeping the source location of this name.
    pub fn to_node(&self, extension_id: Option<ExtensionId>) -> Node<Name> {
        let mut node = Node::new_opt_location(self.clone(), self.location());
        if let Some(id) = extension_id {
            node.set_extension_id(id);
        }
        node
    }
}

impl Clone for Name {
    fn clone(&self) -> Self {
        if let Some(arc) = self.as_arc() {
            let _ptr = Arc::into_raw(Arc::clone(&arc));
            // Conceptually move ownership of this "new" pointer into the new clone
            // However it’s a `*const` and we already have a `NonNull` with the same address in `self`
        }
        Self {
            ptr: self.ptr,
            len: self.len,
            start_offset: self.start_offset,
            tagged_file_id: self.tagged_file_id,
            symbol: AtomicU32::new(self.symbol.load(Relaxed)),
            phantom: PhantomData,
        }
    }
}

impl Drop for Name {
    fn drop(&mut self) {
        if let Some(arc) = &mut self.as_arc() {
            // SAFETY: neither the dropped `ManuallyDrop` nor `self.ptr` is used again
            unsafe { ManuallyDrop::drop(arc) }
        }
    }
}

impl std::hash::Hash for Name {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash the 4-byte symbol, not the string. Consequence:
        // `Borrow<str>` lookups are impossible (str hashes bytes).
        // Names missing from a frozen table hash their string; equal
        // strings always take the same branch (see `Self::symbol`).
        // Location not included in either branch.
        #[deny(non_snake_case)]
        match self.symbol() {
            symbol @ 0..u32::MAX => state.write_u32(symbol),
            PROBED_MISS => self.as_str().hash(state),
        }
    }
}

impl std::ops::Deref for Name {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Name {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Debug for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Display for Name {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Eq for Name {}

impl PartialEq for Name {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // don’t compare location
        let (a, b) = (self.symbol(), other.symbol());
        if a != b {
            // Covers the mixed case too: a hit's string is in the frozen
            // table, a miss's is not, so they can't be equal.
            return false;
        }
        // Equal symbols mean equal strings, except that two names missing
        // from the frozen table must fall back to their strings.
        a != PROBED_MISS || self.as_str() == other.as_str()
    }
}

impl Ord for Name {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl PartialOrd for Name {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::borrow::Borrow<str> for Node<Name> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Node<Name> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<T: AsRef<str>> PartialEq<T> for Node<Name> {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<str> for Name {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialOrd<str> for Name {
    #[inline]
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialEq<&'_ str> for Name {
    #[inline]
    fn eq(&self, other: &&'_ str) -> bool {
        self.as_str() == *other
    }
}

impl PartialOrd<&'_ str> for Name {
    #[inline]
    fn partial_cmp(&self, other: &&'_ str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl From<&'_ Self> for Name {
    #[inline]
    fn from(value: &'_ Self) -> Self {
        value.clone()
    }
}

impl From<Name> for Arc<str> {
    fn from(value: Name) -> Self {
        match value.to_cloned_arc() {
            Some(arc) => arc,
            None => value.as_str().into(),
        }
    }
}

impl TryFrom<Arc<str>> for Name {
    type Error = InvalidNameError;

    fn try_from(value: Arc<str>) -> Result<Self, Self::Error> {
        Self::check_valid_syntax(&value)?;
        Ok(Self::from_arc_unchecked(value))
    }
}

impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> serde::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const EXPECTING: &str = "a string in GraphQL Name syntax";
        struct Visitor;
        impl serde::de::Visitor<'_> for Visitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(EXPECTING)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Name::new(v)
                    .map_err(|_| E::invalid_value(serde::de::Unexpected::Str(v), &EXPECTING))
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl TryFrom<&str> for Name {
    type Error = InvalidNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Name {
    type Error = InvalidNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(&value)
    }
}

impl TryFrom<&'_ String> for Name {
    type Error = InvalidNameError;

    fn try_from(value: &'_ String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl AsRef<Name> for Name {
    fn as_ref(&self) -> &Name {
        self
    }
}

impl ToCliReport for InvalidNameError {
    fn location(&self) -> Option<SourceSpan> {
        self.location
    }
    fn report(&self, report: &mut CliReport) {
        report.with_label_opt(self.location, "cannot be parsed as a GraphQL Name");
    }
}

impl fmt::Debug for InvalidNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

/// A borrowed key for looking up entries of [`Name`]-keyed
/// [`IndexMap`s][crate::collections::IndexMap] by string.
///
/// `Name` compares and hashes by its interned symbol, so such maps cannot
/// be queried with a plain `&str` (which hashes its bytes). `NameKey`
/// probes the symbol table read-only — no allocation, no insertion — and
/// hashes accordingly, making it the drop-in replacement for the former
/// `Borrow<str>`-based lookups:
///
/// ```
/// use apollo_compiler::NameKey;
/// use apollo_compiler::Schema;
///
/// let schema = Schema::parse_and_validate("type Query { x: Int }", "s.graphql").unwrap();
/// let ty = schema.types.get(&NameKey("Query")).unwrap();
/// assert!(ty.is_object());
/// ```
pub struct NameKey<'a>(pub &'a str);

impl std::hash::Hash for NameKey<'_> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match crate::symbol::get_symbol(self.0) {
            Some(symbol) => state.write_u32(symbol.get()),
            None => self.0.hash(state),
        }
    }
}

impl indexmap::Equivalent<Name> for NameKey<'_> {
    #[inline]
    fn equivalent(&self, key: &Name) -> bool {
        self.0 == key.as_str()
    }
}

impl indexmap::Equivalent<crate::Node<Name>> for NameKey<'_> {
    #[inline]
    fn equivalent(&self, key: &crate::Node<Name>) -> bool {
        self.0 == key.as_str()
    }
}
