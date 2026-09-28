//! Global string→symbol table backing [`Name`] equality and hashing.
//!
//! Every distinct name string maps to a `NonZeroU32` symbol. `Name`
//! equality is an integer comparison and `Name`'s `Hash` impl emits the
//! symbol, so `Name`-keyed map operations hash 4 bytes instead of the
//! string. This removes `Borrow<str>` map lookups (see
//! [`NameKey`][crate::NameKey] for the replacement).
//!
//! # Freezing
//!
//! [`freeze`] (exposed as [`crate::freeze_interning`]) makes the table
//! lookup-only. After freezing, hit-or-miss is a pure function of the
//! string: names whose strings are not interned fall back to string-based
//! equality and hashing, equal strings always agree on which branch they
//! take, and a mixed comparison is unequal by construction. Untrusted
//! input can therefore never grow the table — servers freeze after
//! building their schema(s), making the table's size a function of the
//! schemas alone.
//!
//! The freeze must happen-before parsing untrusted input: an insertion
//! racing the freeze itself could let two equal strings disagree on
//! interned-ness. Freezing at server startup, after schema loading and
//! before serving, satisfies this.
//!
//! Entries are owned by the table and live for the process. Without a
//! freeze (e.g. composition tools), growth is bounded by the distinct
//! names in the schemas and documents processed.
//!
//! [`Name`]: crate::Name

use crate::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;

/// The interning table. Uses the same hasher as the crate's public
/// collections; it is only probed when a `Name` is constructed or first
/// resolved, never on the eq/hash hot path.
type Table = HashMap<&'static str, NonZeroU32>;

/// The table to hold the cache while it is being updated. After the cache is frozen, this will
/// no longer be accessed.
///
/// The value of a name's "symbol" identifier is derived from the size of the table. The symbol
/// "0" is reserved as a sentinel for "not yet resolved" and `u32::MAX` stands in for a cache
/// miss on a frozen table.
///
/// NOTE: A mutex is used over an `RwLock` as the critical sections are very small. With an
/// `RwLock`, interning a new name requires getting two guards (a read and then a write), both
/// of which require cross-core syncing. This is a lot of work for simply checking and possibly
/// inserting something into the cache.
static TABLE: LazyLock<Mutex<Table>> = LazyLock::new(|| Mutex::new(HashMap::default()));

/// Immutable snapshot of the table taken at freeze time. Once the snapshot is taken, the original
/// table cache can not be updated.
static SNAPSHOT: LazyLock<Table> =
    LazyLock::new(|| std::mem::take(&mut *TABLE.lock().unwrap_or_else(PoisonError::into_inner)));

/// Cached-symbol sentinel: The symbol for statically-defined `Name`s can not be cached until
/// runtime. Such names are initialized with this sentital value. When found, the cache (or
/// snapshot) is accessed to update the `Name`'s symbol.
pub(crate) const UNINITIALIZED_SYMBOL: u32 = 0;

/// Cached-symbol sentinel: the string was probed against a frozen table and
/// is not interned. Never a valid symbol (the table asserts `next < MAX`).
pub(crate) const PROBED_MISS: u32 = u32::MAX;

/// See [`crate::freeze_interning`].
pub(crate) fn freeze() {
    let _ = LazyLock::force(&SNAPSHOT);
}

/// Returns the symbol for `value` if it is interned, without modifying
/// the table.
pub(crate) fn get_symbol(value: &str) -> Option<NonZeroU32> {
    if let Some(snapshot) = LazyLock::get(&SNAPSHOT) {
        return snapshot.get(value).copied();
    }
    TABLE
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(value)
        .copied()
}

/// Returns the symbol for `value`, interning it if new — unless the table
/// is frozen, in which case misses return `None` and the table is not
/// modified.
pub(crate) fn intern(value: &str) -> Option<(&'static str, NonZeroU32)> {
    // Lock-free fast path once frozen.
    if let Some(snapshot) = LazyLock::get(&SNAPSHOT) {
        return snapshot.get_key_value(value).map(|(&name, &symbol)| (name, symbol));
    }
    let mut guard = TABLE.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some((&name, &symbol)) = guard.get_key_value(value) {
        return Some((name, symbol));
    }
    let symbol = (guard.len() as u32) + 1;
    assert!(symbol < u32::MAX, "interned name symbol space exhausted");
    let symbol = NonZeroU32::new(symbol).expect("NEXT_SYMBOL starts at 1");
    let name =  Box::leak(Box::from(value));
    guard.insert(name, symbol);
    Some((name, symbol))
}

#[cfg(test)]
mod tests {
    use crate::{name, Name};

    use super::*;

    fn table_len() -> usize {
        TABLE.lock().unwrap_or_else(PoisonError::into_inner).len()
    }

    #[test]
    fn freeze_stops_growth_but_keeps_hits() {
        let interned = intern("freezeTestSchemaName").expect("not frozen yet");
        freeze();
        // Hits still resolve after freezing.
        assert_eq!(intern("freezeTestSchemaName"), Some(interned));
        assert_eq!(get_symbol("freezeTestSchemaName"), Some(interned.1));
        // Misses no longer insert.
        let len_before = table_len();
        for i in 0..100 {
            assert_eq!(intern(&format!("freezeTestHostileAlias{i}")), None);
        }
        assert_eq!(table_len(), len_before);
    }

    #[test]
    fn concurrent_interning_converges() {
        let symbols: Vec<Vec<NonZeroU32>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(move || {
                        (0..64)
                            .map(|i| intern(&format!("concurrentInternTest{i}")).unwrap().1)
                            .collect()
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        // All threads agree on every symbol.
        for other in &symbols[1..] {
            assert_eq!(&symbols[0], other);
        }
    }

    #[test]
    fn cache_consistency_after_freeze() {
        let name = name!("foo");
        let mut index = HashMap::<Name, usize>::default();
        // Hashing silently updates static name's symbol value.
        index.insert(name.clone(), 42);
        assert!(index.contains_key(&name));
        super::freeze();
        let name = name!("foo");
        assert!(index.contains_key(&name));
        let name = Name::new("foo").unwrap();
        assert!(index.contains_key(&name));
    }
}
