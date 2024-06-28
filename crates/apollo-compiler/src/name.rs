use crate::diagnostic::CliReport;
use crate::diagnostic::ToCliReport;
use crate::node::TaggedFileId;
use crate::schema::ComponentName;
use crate::schema::ComponentOrigin;
use crate::FileId;
use crate::LineColumn;
use crate::NodeLocation;
use crate::SourceMap;
use rowan::TextRange;
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::ops::Range;
use std::ptr::NonNull;
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

/// A GraphQL identifier
///
/// Like [`Node`][crate::Node], this string type has cheap `Clone`
/// and carries an optional source location.
///
/// Internally, the string value is either an atomically-reference counted `Arc<str>`
/// or a `&'static str` borrow that lives until the end of the program.
//
// Fields: equivalent to `(UnpackedRepr, Option<NodeLocation>)` but more compact
pub struct Name {
    /// Data pointer of either `Arc<str>::into_raw` (if `tagged_file_id.tag() == TAG_ARC`)
    /// or `&'static str` (if `TAG_STATIC`)
    ptr: NonNull<u8>,
    len: u32,
    start_offset: u32,            // zero if we don’t have a location
    tagged_file_id: TaggedFileId, // `.file_id() == FileId::NONE` means we don’t have a location
    phantom: PhantomData<UnpackedRepr>,
}

#[allow(dead_code)] // only used in PhantomData and static asserts
enum UnpackedRepr {
    Heap(Arc<str>),
    Static(&'static str),
}

/// Tried to create a [`Name`] from a string that is not in valid
/// [GraphQL name](https://spec.graphql.org/draft/#sec-Names) syntax.
#[derive(Clone, Eq, PartialEq, thiserror::Error)]
#[error("`{name}` is not a valid GraphQL name")]
pub struct InvalidNameError {
    pub name: String,
    pub location: Option<NodeLocation>,
}

const TAG_ARC: bool = true;
const TAG_STATIC: bool = false;

const _: () = {
    // 20 "useful" bytes on 32-bit targets like wasm,
    // but still padded to 24 for alignment of u64 file ID:
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

    /// Create a new `Name` without validity checking.
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub fn new_unchecked(value: &str) -> Self {
        Self::from_arc_unchecked(value.into())
    }

    /// Create a new `Name` from an `Arc`, without validity checking.
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub fn from_arc_unchecked(arc: Arc<str>) -> Self {
        let len = Self::new_len(&arc);
        let ptr = Arc::into_raw(arc).cast_mut().cast();
        // SAFETY: Arc always is non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self {
            ptr,
            len,
            start_offset: 0,
            tagged_file_id: TaggedFileId::pack(TAG_ARC, FileId::NONE),
            phantom: PhantomData,
        }
    }

    /// Create a new `Name` from a string with static lifetime, without validity checking.
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
            phantom: PhantomData,
        }
    }

    /// Modifies the given name to add its location in a parsed source file
    pub fn with_location(mut self, location: NodeLocation) -> Self {
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
    pub fn location(&self) -> Option<NodeLocation> {
        let file_id = self.tagged_file_id.file_id();
        if file_id != FileId::NONE {
            Some(NodeLocation {
                file_id,
                text_range: TextRange::at(self.start_offset.into(), self.len.into()),
            })
        } else {
            None
        }
    }

    /// If this string contains a location, convert it to starting line and column numbers
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
    /// Exactly one of this method or [`to_cloned_arc`][Self::to_cloned_arc] returns `Some`.
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
    /// Exactly one of this method or [`as_static_str`][Self::as_static_str] returns `Some`.
    pub fn to_cloned_arc(&self) -> Option<Arc<str>> {
        self.as_arc()
            .map(|manually_drop| Arc::clone(&manually_drop))
    }

    /// Returns whether the given string is a valid GraphQL name.
    ///
    /// <https://spec.graphql.org/October2021/#Name>
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

    /// <https://spec.graphql.org/October2021/#NameStart>
    const fn is_name_start(byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_'
    }

    /// <https://spec.graphql.org/October2021/#NameContinue>
    const fn is_name_continue(byte: u8) -> bool {
        byte.is_ascii_alphanumeric() || byte == b'_'
    }

    pub fn to_component(&self, origin: ComponentOrigin) -> ComponentName {
        ComponentName {
            origin,
            name: self.clone(),
        }
    }
}

impl Clone for Name {
    fn clone(&self) -> Self {
        if let Some(arc) = self.as_arc() {
            let _ptr = Arc::into_raw(Arc::clone(&arc));
            // Conceptually move ownership of this "new" pointer into the new clone
            // However it’s a `*const` and we already have a `NonNull` with the same address in `self`
        }
        Self { ..*self }
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
        self.as_str().hash(state) // location not included
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

impl std::borrow::Borrow<str> for Name {
    fn borrow(&self) -> &str {
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
        self.as_str() == other.as_str() // don’t compare location
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
        impl<'de> serde::de::Visitor<'de> for Visitor {
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
    fn location(&self) -> Option<NodeLocation> {
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
