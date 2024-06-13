use hashbrown::raw::RawTable;
use parking_lot::RwLock;
use parking_lot::RwLockUpgradableReadGuard;
use std::hash::BuildHasher;
use std::hash::RandomState;
use std::sync::OnceLock;

#[derive(Default)]
struct Interner {
    locked_table: RwLock<RawTable<&'static str>>,
    hasher: RandomState,
}

fn interner() -> &'static Interner {
    static INTERNER: OnceLock<Interner> = OnceLock::new();
    INTERNER.get_or_init(Default::default)
}

/// Return a string equal to `value` with `'static` lifetime valid until the end of the program.
///
/// Strings are deduplicated in a global hash set
/// and their heap allocations leaked with [`String::leak`] so that they are never deallocated.
/// This may be appropriate for short-lived programs or for a bounded set of string values, but
/// be careful about exposing this to user/client input in a long-lived program such as a server.
pub fn intern_and_leak(value: &str) -> &'static str {
    // Using `RawTable` instead of `HashSet` allows computing the string hash only once
    // and reusing it in both `get` and `insert`.
    // `HashMap::entry` would require an exclusive lock on the table from the start,
    // whereas we first take a read-only lock and upgrade it only if necessary.
    let Interner {
        locked_table,
        hasher,
    } = interner();
    let hash = hasher.hash_one::<&str>(value);
    let table = locked_table.upgradable_read();
    if let Some(interned) = table.get(hash, |interned| value == *interned) {
        interned
    } else {
        let leaked = value.to_owned().leak();
        let mut table = RwLockUpgradableReadGuard::upgrade(table);
        table.insert_entry(hash, leaked, |reallocated_entry| {
            // When the table grows, existing entries need to be rehashed
            hasher.hash_one::<&str>(reallocated_entry)
        })
    }
}

/// If a string equal to `value` has already been interned, return its `'static` reference.
pub fn get(value: &str) -> Option<&'static str> {
    let interner = interner();
    let hash = interner.hasher.hash_one::<&str>(value);
    interner
        .locked_table
        .read()
        .get(hash, |interned| value == *interned)
        .copied()
}
