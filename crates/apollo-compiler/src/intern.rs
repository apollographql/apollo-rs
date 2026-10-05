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
use std::sync::MutexGuard;
use std::sync::OnceLock;
use std::sync::PoisonError;

type Table = HashSet<&'static str>;

/// The table while it can still grow, or `None` once it has moved to `FROZEN`.
static GROWING: LazyLock<Mutex<Option<Table>>> =
    LazyLock::new(|| Mutex::new(Some(Table::default())));

/// The table after freezing, read without locking.
static FROZEN: OnceLock<Table> = OnceLock::new();

fn lock_growing() -> MutexGuard<'static, Option<Table>> {
    GROWING.lock().unwrap_or_else(PoisonError::into_inner)
}

/// See [`crate::freeze_interning`].
pub(crate) fn freeze() {
    let mut growing = lock_growing();
    if let Some(mut table) = growing.take() {
        table.shrink_to_fit();
        // Publishing before the lock is released means a thread that finds
        // `GROWING` empty can always find `FROZEN`.
        let _ = FROZEN.set(table);
    }
}

/// Returns the table's copy of `value`, interning it if new. Once the
/// table is frozen, misses return `None` and the table is not modified.
pub(crate) fn intern(value: &str) -> Option<&'static str> {
    if let Some(frozen) = FROZEN.get() {
        return frozen.get(value).copied();
    }
    let mut growing = lock_growing();
    let Some(table) = growing.as_mut() else {
        // Frozen while we waited for the lock.
        return FROZEN.get()?.get(value).copied();
    };
    if let Some(&name) = table.get(value) {
        return Some(name);
    }
    let name: &'static str = Box::leak(Box::from(value));
    table.insert(name);
    Some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Freezing is tested in `tests/freeze.rs`, since it would affect every
    // other test sharing this process.
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
