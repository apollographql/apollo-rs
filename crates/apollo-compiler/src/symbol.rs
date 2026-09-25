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
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::OnceLock;
use std::sync::PoisonError;
use std::sync::RwLock;

/// The interning table. Uses the same hasher as the crate's public
/// collections; it is only probed when a `Name` is constructed or first
/// resolved, never on the eq/hash hot path.
type Table = HashMap<Box<str>, NonZeroU32>;

/// Once frozen, the table only answers lookups; see the module docs.
static FROZEN: AtomicBool = AtomicBool::new(false);

/// Immutable snapshot of the table taken at freeze time: the post-freeze
/// read path uses it without taking any lock.
static SNAPSHOT: OnceLock<Table> = OnceLock::new();

/// Next symbol to allocate. Symbol 0 is `Name`'s "not yet resolved"
/// sentinel and `u32::MAX` is its "probed a frozen table and missed"
/// sentinel, so valid symbols are `1..u32::MAX`.
static NEXT_SYMBOL: AtomicU32 = AtomicU32::new(1);

fn table() -> &'static RwLock<Table> {
    static TABLE: OnceLock<RwLock<Table>> = OnceLock::new();
    TABLE.get_or_init(|| RwLock::new(HashMap::default()))
}

pub(crate) fn frozen() -> bool {
    FROZEN.load(Relaxed)
}

/// See [`crate::freeze_interning`].
pub(crate) fn freeze() {
    FROZEN.store(true, Relaxed);
    let snapshot = table()
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .map(|(k, &v)| (k.clone(), v))
        .collect();
    // First freeze wins; a subsequent freeze() is a no-op.
    let _ = SNAPSHOT.set(snapshot);
}

/// Returns the symbol for `value` if it is interned, without modifying
/// the table.
pub(crate) fn get(value: &str) -> Option<NonZeroU32> {
    if let Some(snapshot) = SNAPSHOT.get() {
        return snapshot.get(value).copied();
    }
    table()
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .get(value)
        .copied()
}

/// Returns the symbol for `value`, interning it if new — unless the table
/// is frozen, in which case misses return `None` and the table is not
/// modified.
pub(crate) fn intern(value: &str) -> Option<NonZeroU32> {
    // Lock-free fast path once frozen.
    if let Some(snapshot) = SNAPSHOT.get() {
        return snapshot.get(value).copied();
    }
    {
        let guard = table().read().unwrap_or_else(PoisonError::into_inner);
        if let Some(&symbol) = guard.get(value) {
            return Some(symbol);
        }
    }
    if frozen() {
        return None;
    }
    let mut guard = table().write().unwrap_or_else(PoisonError::into_inner);
    if let Some(&symbol) = guard.get(value) {
        return Some(symbol);
    }
    let raw = NEXT_SYMBOL.fetch_add(1, Relaxed);
    assert!(raw < u32::MAX, "interned name symbol space exhausted");
    let symbol = NonZeroU32::new(raw).expect("NEXT_SYMBOL starts at 1");
    guard.insert(value.into(), symbol);
    Some(symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_len() -> usize {
        table().read().unwrap_or_else(PoisonError::into_inner).len()
    }

    #[test]
    fn freeze_stops_growth_but_keeps_hits() {
        let interned = intern("freezeTestSchemaName").expect("not frozen yet");
        freeze();
        // Hits still resolve after freezing.
        assert_eq!(intern("freezeTestSchemaName"), Some(interned));
        assert_eq!(get("freezeTestSchemaName"), Some(interned));
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
                    scope.spawn(|| {
                        (0..64)
                            .map(|i| intern(&format!("concurrentInternTest{i}")).unwrap())
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
}
