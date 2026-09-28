//! Global table of interned name strings backing [`Name`] equality.
//!
//! Every distinct name string is leaked once into the table, and interned
//! `Name`s point at that single copy. Two interned names are therefore
//! equal exactly when their pointers are equal, and they hash the pointer
//! instead of the string bytes. This rules out `Borrow<str>` map lookups,
//! so `Name`-keyed maps are queried with a `Name`.
//!
//! # Freezing
//!
//! [`freeze`] (exposed as [`crate::freeze_interning`]) makes the table
//! lookup-only. After freezing, strings missing from the table produce
//! `Arc`-backed names that compare and hash by string. Whether a string is
//! in the frozen table does not depend on which `Name` holds it, so equal
//! names always agree on how to hash. Untrusted input can therefore
//! never grow the table. Servers freeze after building their schema(s),
//! making the table's size a function of the schemas alone.
//!
//! Entries are owned by the table and live for the process. Without a
//! freeze (e.g. composition tools), growth is bounded by the distinct
//! names in the schemas and documents processed.
//!
//! [`Name`]: crate::Name

use crate::collections::HashSet;
use std::sync::LazyLock;
use std::sync::Mutex;
use std::sync::PoisonError;

type Table = HashSet<&'static str>;

/// The table while it can still grow. After freezing it is left empty and
/// never accessed again.
///
/// A mutex is used over an `RwLock` because the critical sections are
/// small, and an `RwLock` would need both a read and a write guard to
/// insert a new name.
static TABLE: LazyLock<Mutex<Table>> = LazyLock::new(|| Mutex::new(HashSet::default()));

/// Immutable snapshot of the table taken at freeze time, read without locking.
static SNAPSHOT: LazyLock<Table> = LazyLock::new(|| {
    let mut guard = TABLE.lock().unwrap_or_else(PoisonError::into_inner);
    guard.shrink_to_fit();
    std::mem::take(&mut *guard)
});

/// See [`crate::freeze_interning`].
pub(crate) fn freeze() {
    let _ = LazyLock::force(&SNAPSHOT);
}

/// Returns the table's copy of `value`, interning it if new. Once the
/// table is frozen, misses return `None` and the table is not modified.
pub(crate) fn intern(value: &str) -> Option<&'static str> {
    if let Some(snapshot) = LazyLock::get(&SNAPSHOT) {
        return snapshot.get(value).copied();
    }
    let mut guard = TABLE.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(&name) = guard.get(value) {
        return Some(name);
    }
    let name: &'static str = Box::leak(Box::from(value));
    guard.insert(name);
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table_len() -> usize {
        TABLE.lock().unwrap_or_else(PoisonError::into_inner).len()
    }

    #[test]
    fn freeze_stops_growth_but_keeps_hits() {
        let interned = intern("freezeTestSchemaName").expect("not frozen yet");
        freeze();
        let hit = intern("freezeTestSchemaName").expect("interned before freeze");
        assert!(std::ptr::eq(hit, interned));
        let len_before = table_len();
        for i in 0..100 {
            assert_eq!(intern(&format!("freezeTestHostileAlias{i}")), None);
        }
        assert_eq!(table_len(), len_before);
    }

    #[test]
    fn concurrent_interning_converges() {
        let ptrs: Vec<Vec<usize>> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..8)
                .map(|_| {
                    scope.spawn(move || {
                        (0..64)
                            .map(|i| {
                                let name = intern(&format!("concurrentInternTest{i}")).unwrap();
                                name.as_ptr() as usize
                            })
                            .collect()
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });
        for other in &ptrs[1..] {
            assert_eq!(&ptrs[0], other);
        }
    }
}
