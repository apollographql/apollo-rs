#![allow(unstable_name_collisions)] // for `sptr::Strict`

use crate::schema::ComponentOrigin;
use crate::schema::ComponentStr;
use crate::NodeLocation;
use sptr::Strict;
use std::marker::PhantomData;
use std::mem::size_of;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use triomphe::ThinArc;

/// Smart string type for names and string values in a GraphQL document
///
/// Like [`Node`][crate::Node] it is thread-safe, reference-counted,
/// and carries an optional source location.
/// It is a thin pointer to a single allocation, with a header followed by string data.
pub struct NodeStr {
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

const _: () = {
    // Both `HeapRepr` and `StaticRepr` are pointers to sufficiently-aligned values,
    // so the lowest address bit is always available to use as a tag
    assert!(std::mem::align_of::<&'static str>() >= 2);
    assert!(std::mem::align_of::<Header>() >= 2);

    // Both pointers are non-null, leaving a niche to represent `None` without extra size
    assert!(size_of::<Option<HeapRepr>>() == size_of::<usize>());
    assert!(size_of::<Option<StaticRepr>>() == size_of::<usize>());
    assert!(size_of::<Option<NodeStr>>() == size_of::<usize>());

    // the `unsafe impl`s below are sound
    const fn _assert_send<T: Send>() {}
    const fn _assert_sync<T: Send>() {}
    _assert_send::<HeapRepr>();
    _assert_sync::<HeapRepr>();
    _assert_send::<StaticRepr>();
    _assert_sync::<StaticRepr>();
};

unsafe impl Send for NodeStr {}

unsafe impl Sync for NodeStr {}

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

impl NodeStr {
    /// Create a new `NodeStr` parsed from the given source location
    #[inline]
    pub fn new_parsed(value: &str, location: NodeLocation) -> Self {
        Self::new_heap(ThinArc::from_header_and_slice(
            Some(location),
            value.as_bytes(),
        ))
    }

    /// Create a new `NodeStr` programatically, not parsed from a source file
    #[inline]
    pub fn new(value: &str) -> Self {
        Self::new_heap(ThinArc::from_header_and_slice(None, value.as_bytes()))
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
    /// let s = apollo_compiler::NodeStr::from_static(&"example");
    /// assert_eq!(s, "example");
    /// ```
    pub const fn from_static(str_ref: &'static &'static str) -> Self {
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
    pub fn to_component(&self, origin: ComponentOrigin) -> ComponentStr {
        ComponentStr {
            origin,
            node: self.clone(),
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
}

impl Clone for NodeStr {
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

impl Drop for NodeStr {
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
        Some(self.cmp(other))
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

impl From<&'_ str> for NodeStr {
    #[inline]
    fn from(value: &'_ str) -> Self {
        Self::new(value)
    }
}

impl From<&'_ String> for NodeStr {
    #[inline]
    fn from(value: &'_ String) -> Self {
        Self::new(value)
    }
}

impl From<String> for NodeStr {
    #[inline]
    fn from(value: String) -> Self {
        Self::new(&value)
    }
}

impl From<&'_ Self> for NodeStr {
    #[inline]
    fn from(value: &'_ Self) -> Self {
        value.clone()
    }
}
