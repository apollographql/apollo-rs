//! Supporting library for generating GraphQL documents
//! in a [fuzzing target](https://rust-fuzz.github.io/book/introduction.html)
//!
//! This is based on parts of the [`arbitrary`](https://github.com/rust-fuzz/arbitrary) crate
//! (some code copied under the dual MIT or Apache-2.0 license)
//! with some key differences:
//!
//! * No `Arbitrary` trait which generates a value in isolation.
//!   Parts of a GraphQL document often refer to each other so generation needs to be context aware.
//!   (For example: what are the available types a new field definition can have?)
//!
//! * No `size_hint` mechanism.
//!   It seems designed for types that have a finite set of possible values
//!   and does not work out well for tree-like data structures.
//!   Generating a single `&str` consumes up to all available entropy (half of it on average).
//!
//! * Infallible APIs.
//!   `arbitrary` can return an error enum in many places
//!   but each of its variants is only generated in a few places, and not consistently.
//!   For example, `int_in_range` looks like it could return `arbitrary::Error::NotEnoughData`
//!   but it never does.
//!   Instead it defaults to `range.start()` when entropy is exhausted:
//!
//!   <https://github.com/rust-fuzz/arbitrary/issues/139#issuecomment-1385850032>
//!   > We've had empirical results that suggest that this behavior
//!   > results in better fuzzing performance and exploration of the input state space
//!
//!   We apply this choice everywhere and remove the error enum entirely.

use std::ops::RangeInclusive;

/// Uses a byte sequence (typically provided by a fuzzer) as a source of entropy
/// to generate various arbitrary values.
pub struct Entropy<'arbitrary_bytes> {
    bytes: std::slice::Iter<'arbitrary_bytes, u8>,
}

impl<'arbitrary_bytes> Entropy<'arbitrary_bytes> {
    /// Create a new source of entropy from bytes typically
    /// [provided by cargo-fuzz](https://rust-fuzz.github.io/book/cargo-fuzz/tutorial.html).
    ///
    /// Bytes are assumed to have a mostly uniform distribution in the `0..=255` range.
    pub fn new(arbitrary_bytes: &'arbitrary_bytes [u8]) -> Self {
        Self {
            bytes: arbitrary_bytes.iter(),
        }
    }

    /// Returns whether entropy has been exhausted
    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }

    /// Take all remaining entropy. After this, [`Self::is_empty`] return true.
    pub fn take_all(&mut self) {
        self.bytes = Default::default()
    }

    /// Generates an arbitary byte, or zero if entropy is exhausted.
    pub fn u8(&mut self) -> u8 {
        if let Some(&b) = self.bytes.next() {
            b
        } else {
            0
        }
    }

    pub fn u8_array<const N: usize>(&mut self) -> [u8; N] {
        std::array::from_fn(|_| self.u8())
    }

    pub fn i32(&mut self) -> i32 {
        i32::from_le_bytes(self.u8_array())
    }

    pub fn f64(&mut self) -> f64 {
        f64::from_le_bytes(self.u8_array())
    }

    /// Generates an arbitrary boolean, or `false` if entropy is exhausted.
    ///
    /// Generally, code paths that cause more entropy to be consumed
    /// should be taken when this method returns `true`.
    /// If used in a loop break condition for example,
    /// make sure to use this boolean as “keep going?” instead of “break?”
    /// so that the loop stops when entropy is exhausted.
    ///
    /// A loop like `while keep_going() { … }` where `keep_going()` is true with probability `P`:
    ///
    /// * Has probability `(1-P)` to never run
    /// * Has probability `P × (1-P)  ` to run exactly one iteration
    /// * Has probability `P² × (1-P)` to run exactly two iterations
    /// * In general, has probability `P^k × (1-P)` to run exactly `k` iterations
    ///
    /// The expected (average) number of iterations is:
    /// `N = sum[k = 0 to ∞] of k × P^k × (1-P) = P / (1 - P)` (thanks WolframAlpha).
    /// Conversely, to get on average `N` iterations we should pick `P = N / (N + 1)`.
    ///
    /// For example, `while entropy.bool() { … }` has `P = 0.5` and so `N = 1`.
    /// To make a similar loop that runs more than one iteration on average
    /// we need “keep going” boolean conditions with higher probability.
    pub fn bool(&mut self) -> bool {
        (self.u8() & 1) == 1
    }

    /// Generates an arbitrary index in `0..collection_len`, or zero if entropy is exhausted.
    ///
    /// Retuns `None` if `collection_len == 0`.
    ///
    /// The returned index is biased towards lower values:
    ///
    /// * If `choice_count` is not a power of two, or
    /// * If entropy becomes exhausted while generating the index
    pub fn index(&mut self, collection_len: usize) -> Option<usize> {
        let last = collection_len.checked_sub(1)?;
        Some(self.int(0..=last))
    }

    /// Chooses an arbitrary item of the given slice, or the first if entropy is exhausted.
    ///
    /// Retuns `None` if the slice is empty.
    ///
    /// The returned index is biased towards earlier items:
    ///
    /// * If the slice length is not a power of two, or
    /// * If entropy becomes exhausted while generating the index
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> Option<&'a T> {
        let index = self.index(slice.len())?;
        Some(&slice[index])
    }

    /// Generates an arbitrary integer in the given range,
    /// or `range.start()` if entropy is exhausted.
    ///
    /// The returned value is biased towards lower values:
    ///
    /// * If `choice_count` is not a power of two, or
    /// * If entropy becomes exhausted while generating the value
    ///
    /// # Panics
    ///
    /// Panics if `range.start > range.end`.
    /// That is, the given range must be non-empty.
    pub fn int<T: Int>(&mut self, range: RangeInclusive<T>) -> T {
        // Based on `arbitrary::Unstructured::int_in_range`:
        // https://docs.rs/arbitrary/1.3.2/src/arbitrary/unstructured.rs.html#302

        let start = *range.start();
        let end = *range.end();
        assert!(start <= end, "`Entropy::int` requires a non-empty range");

        // When there is only one possible choice,
        // don’t waste any entropy from the underlying data.
        if start == end {
            return start;
        }

        // From here on out we work with the unsigned representation.
        // All of the operations performed below work out just as well
        // whether or not `T` is a signed or unsigned integer.
        let start = start.to_unsigned();
        let end = end.to_unsigned();

        let delta = end.wrapping_sub(start);
        debug_assert_ne!(delta, T::Unsigned::ZERO);

        // Compute an arbitrary integer offset from the start of the range.
        // We do this by consuming up to `size_of(T)` bytes from the input
        // to create an arbitrary integer
        // and then clamping that int into our range bounds with a modulo operation.
        let entropy_bits_wanted = T::BITS - delta.leading_zeros();
        let entropy_bytes_wanted = entropy_bits_wanted.div_ceil(8);
        let mut arbitrary_int = T::Unsigned::ZERO;
        for _ in 0..entropy_bytes_wanted {
            let next = match self.bytes.next() {
                None => break,
                Some(&byte) => T::Unsigned::from(byte),
            };

            // Combine this byte into our arbitrary integer, but avoid
            // overflowing the shift for `u8` and `i8`.
            arbitrary_int = if std::mem::size_of::<T>() == 1 {
                next
            } else {
                (arbitrary_int << 8) | next
            };
        }

        let offset = if delta == T::Unsigned::MAX {
            arbitrary_int
        } else {
            arbitrary_int % (delta.checked_add(T::Unsigned::ONE).unwrap())
        };

        // Finally, we add `start` to our offset from `start` to get the result
        // actual value within the range.
        let result = start.wrapping_add(offset);

        // And convert back to our maybe-signed representation.
        let result = T::from_unsigned(result);
        debug_assert!(*range.start() <= result);
        debug_assert!(result <= *range.end());

        result
    }
}

mod sealed {
    pub trait Sealed {}
}

// Based on https://docs.rs/arbitrary/1.3.2/src/arbitrary/unstructured.rs.html#748

/// A trait that is implemented for all of the primitive integers:
///
/// * `u8`
/// * `u16`
/// * `u32`
/// * `u64`
/// * `u128`
/// * `usize`
/// * `i8`
/// * `i16`
/// * `i32`
/// * `i64`
/// * `i128`
/// * `isize`
///
/// Intended solely for methods of [`Entropy`].
/// The exact bounds and associated items may change.
pub trait Int:
    sealed::Sealed
    + Copy
    + Ord
    + std::ops::BitOr<Output = Self>
    + std::ops::Rem<Output = Self>
    + std::ops::Shl<u32, Output = Self>
    + std::ops::Shr<Output = Self>
    + std::fmt::Debug
{
    #[doc(hidden)]
    type Unsigned: Int + From<u8>;

    #[doc(hidden)]
    const ZERO: Self;

    #[doc(hidden)]
    const ONE: Self;

    #[doc(hidden)]
    const MAX: Self;

    #[doc(hidden)]
    const BITS: u32;

    #[doc(hidden)]
    fn leading_zeros(self) -> u32;

    #[doc(hidden)]
    fn checked_add(self, rhs: Self) -> Option<Self>;

    #[doc(hidden)]
    fn wrapping_add(self, rhs: Self) -> Self;

    #[doc(hidden)]
    fn wrapping_sub(self, rhs: Self) -> Self;

    #[doc(hidden)]
    fn to_unsigned(self) -> Self::Unsigned;

    #[doc(hidden)]
    fn from_unsigned(unsigned: Self::Unsigned) -> Self;
}

macro_rules! impl_int {
    ( $( $ty:ty : $unsigned_ty: ty ; )* ) => {
        $(
            impl sealed::Sealed for $ty {}

            impl Int for $ty {
                type Unsigned = $unsigned_ty;

                const ZERO: Self = 0;

                const ONE: Self = 1;

                const MAX: Self = Self::MAX;

                const BITS: u32 = Self::BITS;

                fn leading_zeros(self) -> u32 {
                    <$ty>::leading_zeros(self)
                }

                fn checked_add(self, rhs: Self) -> Option<Self> {
                    <$ty>::checked_add(self, rhs)
                }

                fn wrapping_add(self, rhs: Self) -> Self {
                    <$ty>::wrapping_add(self, rhs)
                }

                fn wrapping_sub(self, rhs: Self) -> Self {
                    <$ty>::wrapping_sub(self, rhs)
                }

                fn to_unsigned(self) -> Self::Unsigned {
                    self as $unsigned_ty
                }

                fn from_unsigned(unsigned: $unsigned_ty) -> Self {
                    unsigned as Self
                }
            }
        )*
    }
}

impl_int! {
    u8: u8;
    u16: u16;
    u32: u32;
    u64: u64;
    u128: u128;
    usize: usize;
    i8: u8;
    i16: u16;
    i32: u32;
    i64: u64;
    i128: u128;
    isize: usize;
}
