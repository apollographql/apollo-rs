#![allow(unstable_name_collisions)] // for `sptr::Strict`

use crate::diagnostic::CliReport;
use crate::diagnostic::ToCliReport;
use crate::execution::GraphQLLocation;
use crate::schema::ComponentName;
use crate::schema::ComponentOrigin;
use crate::NodeLocation;
use crate::SourceMap;
use sptr::Strict;
use std::fmt;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use triomphe::ThinArc;

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
        const _: () = { assert!($crate::Name::valid_syntax($value)) };
        $crate::Name::new_static_unchecked(&$value)
    }};
}

/// A GraphQL identifier
///
/// Smart string type for names and string values in a GraphQL document
///
/// Like [`Node`][crate::Node] it is thread-safe, reference-counted,
/// and carries an optional source location.
/// It is a thin pointer to a single allocation, with a header followed by string data.
pub struct Name {
    /// A type-erased pointer for either `HeapRepr` or `StaticRepr`,
    /// with a tag in the lowest bit: 1 for heap, 0 for static.
    ptr: NonNull<()>,
    phantom: PhantomData<UnpackedRepr>,
}

type Header = Option<NodeLocation>;
type HeapRepr = ThinArc<Header, u8>;
type StaticRepr = &'static &'static str;

#[allow(unused)] // only used in PhantomData
/// What we would use if it didn’t spend an extra 64 bits to store the 1-bit discriminant
enum UnpackedRepr {
    Heap(HeapRepr),
    Static(StaticRepr),
}

#[derive(Clone, Eq, PartialEq, thiserror::Error)]
#[error("`{name}` is not a valid GraphQL name")]
pub struct InvalidNameError {
    name: String,
    location: Option<NodeLocation>,
}

const _: () = {
    // Both `HeapRepr` and `StaticRepr` are pointers to sufficiently-aligned values,
    // so the lowest address bit is always available to use as a tag
    assert!(std::mem::align_of::<&'static str>() >= 2);
    assert!(std::mem::align_of::<Header>() >= 2);

    // Both pointers are non-null, leaving a niche to represent `None` without extra size
    assert!(size_of::<Option<HeapRepr>>() == size_of::<usize>());
    assert!(size_of::<Option<StaticRepr>>() == size_of::<usize>());
    assert!(size_of::<Option<Name>>() == size_of::<usize>());

    // the `unsafe impl`s below are sound
    const fn _assert_send<T: Send>() {}
    const fn _assert_sync<T: Send>() {}
    _assert_send::<HeapRepr>();
    _assert_sync::<HeapRepr>();
    _assert_send::<StaticRepr>();
    _assert_sync::<StaticRepr>();
};

unsafe impl Send for Name {}

unsafe impl Sync for Name {}

const TAG_BITS: usize = 1_usize;

fn address_has_tag(address: usize) -> bool {
    (address & TAG_BITS) != 0
}

fn address_add_tag(address: usize) -> usize {
    address | TAG_BITS
}

fn address_clear_tag(address: usize) -> usize {
    address & !TAG_BITS
}

impl Name {
    /// Create a new `NodeStr` parsed from the given source location
    #[inline]
    pub fn new_parsed(value: &str, location: NodeLocation) -> Result<Self, InvalidNameError> {
        Self::check_valid_syntax(value, Some(location))?;
        Ok(Self::new_heap(ThinArc::from_header_and_slice(
            Some(location),
            value.as_bytes(),
        )))
    }

    /// Create a new `NodeStr` programatically, not parsed from a source file
    #[inline]
    pub fn new(value: &str) -> Result<Self, InvalidNameError> {
        Self::check_valid_syntax(value, None)?;
        Ok(Self::new_heap(ThinArc::from_header_and_slice(
            None,
            value.as_bytes(),
        )))
    }

    #[inline]
    fn new_heap(arc: HeapRepr) -> Self {
        let ptr = ThinArc::into_raw(arc).cast_mut().cast::<()>();
        let tagged_ptr = ptr.map_addr(|address| {
            debug_assert!(!address_has_tag(address)); // checked statically with `align_of` above
            address_add_tag(address)
        });
        Self {
            // Safety: `ThinArc` is always non-null
            ptr: unsafe { NonNull::new_unchecked(tagged_ptr) },
            phantom: PhantomData,
        }
    }

    /// Create a new `NodeStr` from a static string.
    ///
    /// `&str` is a wide pointer (length as pointer metadata stored next to the data pointer),
    /// but we only have space for a thin pointer. So add another `&_` indirection.
    ///
    /// Example:
    ///
    /// ```
    /// let s = apollo_compiler::Name::from_static(&"example").unwrap();
    /// assert_eq!(s, "example");
    /// ```
    pub fn from_static(str_ref: &'static &'static str) -> Result<Self, InvalidNameError> {
        Self::check_valid_syntax(str_ref, None)?;
        Ok(Self::new_static_unchecked(str_ref))
    }

    /// Creates a new `Name` without validity checking.
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub const fn new_static_unchecked(str_ref: &'static &'static str) -> Self {
        let ptr: *const &'static str = str_ref;
        let ptr = ptr.cast_mut().cast();
        // Safety: converted from `&_` which is non-null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };
        Self {
            ptr,
            phantom: PhantomData,
        }
    }

    #[inline]
    fn as_heap(&self) -> Option<*const std::ffi::c_void> {
        let ptr = self.ptr.as_ptr();
        let address = ptr.addr();
        let is_heap = address_has_tag(address);
        is_heap.then(|| {
            ptr.with_addr(address_clear_tag(address))
                .cast_const()
                .cast()
        })
    }

    #[inline]
    fn with_heap<R>(&self, f: impl FnOnce(Option<&HeapRepr>) -> R) -> R {
        if let Some(heap_ptr) = self.as_heap() {
            // Safety:
            //
            // * We’ve checked with the tag that this was created from `Self::new_heap`
            // * This `from_raw` mirrors `into_raw` in `Self::new_heap`
            //
            // `from_raw` normally moves ownership away from the raw pointer,
            // `ManuallyDrop` counteracts that.
            let arc = ManuallyDrop::new(unsafe { ThinArc::from_raw(heap_ptr) });
            f(Some(&arc))
        } else {
            f(None)
        }
    }

    #[inline]
    pub fn location(&self) -> Option<NodeLocation> {
        self.with_heap(|maybe_heap| maybe_heap?.header.header)
    }

    /// If this string contains a location, convert it to line and column numbers
    pub fn line_column(&self, sources: &SourceMap) -> Option<GraphQLLocation> {
        GraphQLLocation::from_node(sources, self.location())
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        self.with_heap(|maybe_heap| {
            if let Some(heap) = maybe_heap {
                // Safety: the bytes in `slice` were copied from an UTF-8 `&str`,
                // and are immutable since.
                let str = unsafe { std::str::from_utf8_unchecked(&heap.slice) };
                // Safety: `heap` is a `&ThinArc` reference
                // whose lifetime is limited to the stack frame of `with_heap`
                // but that points to a `ThinArc` owned by `self`.
                // Since `self` is immutable,
                // the string slice owned by `ThinArc` lives as long as `self`
                // and we can safely extend the lifetime of this borrow:
                let raw: *const str = str;
                unsafe { &*raw }
            } else {
                let ptr: *const &'static str = self.ptr.as_ptr().cast_const().cast();
                // Safety: we just reversed the steps of `Self::_from_static`,
                // which had started from a valid `&'static &'static str`
                unsafe { *ptr }
            }
        })
    }

    /// Returns whether the given string is a valid GraphQL name.
    ///
    /// <https://spec.graphql.org/October2021/#Name>
    pub const fn valid_syntax(value: &str) -> bool {
        let bytes = value.as_bytes();
        let Some(&first) = bytes.first() else {
            return false;
        };
        if !Self::char_is_name_start(first) {
            return false;
        }
        // TODO: iterator when available in const
        let mut i = 1;
        while i < bytes.len() {
            if !Self::char_is_name_continue(bytes[i]) {
                return false;
            }
            i += 1
        }
        true
    }

    fn check_valid_syntax(
        value: &str,
        location: Option<NodeLocation>,
    ) -> Result<(), InvalidNameError> {
        if Self::valid_syntax(value) {
            Ok(())
        } else {
            Err(InvalidNameError {
                name: value.to_owned(),
                location,
            })
        }
    }

    /// <https://spec.graphql.org/October2021/#NameStart>
    const fn char_is_name_start(byte: u8) -> bool {
        byte.is_ascii_alphabetic() || byte == b'_'
    }

    /// <https://spec.graphql.org/October2021/#NameContinue>
    const fn char_is_name_continue(byte: u8) -> bool {
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
        self.with_heap(|maybe_heap| {
            if let Some(heap) = maybe_heap {
                Self::new_heap(ThinArc::clone(heap))
            } else {
                // `&'static &'static str` is `Copy`, just copy the pointer
                Self { ..*self }
            }
        })
    }
}

impl Drop for Name {
    fn drop(&mut self) {
        if let Some(heap_ptr) = self.as_heap() {
            // Safety:
            //
            // * We’ve checked with the tag that this was created from `Self::new_heap`
            // * This `from_raw` mirrors `into_raw` in `Self::new_heap`
            //
            // `from_raw` moves ownership away from the raw pointer, which we want for drop.
            let arc: HeapRepr = unsafe { ThinArc::from_raw(heap_ptr) };
            drop(arc)
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
        report.with_label_opt(self.location, "cannot be parsed as a GraphQL name");
    }
}

impl fmt::Debug for InvalidNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
