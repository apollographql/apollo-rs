//! Interning tests that freeze the global table.
//!
//! Freezing is one-way and process-wide, so these run in order from a single
//! test in their own binary: checks that need an unfrozen table, then a
//! freeze that races with interning, then checks against the frozen table.

use apollo_compiler::collections::IndexMap;
use apollo_compiler::name;
use apollo_compiler::Name;
use proptest::prelude::*;
use proptest::test_runner::Config;
use proptest::test_runner::TestRunner;
use std::collections::HashMap;
use std::hash::Hash;
use std::hash::Hasher;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[test]
fn interning_before_during_and_after_freeze() {
    let config = Config {
        cases: std::env::var("PROPTEST_CASES")
            .map(|cases| cases.parse().expect("PROPTEST_CASES is a number"))
            .unwrap_or(256),
        failure_persistence: None,
        ..Config::default()
    };

    name_invariants(&config, "[a-c]{1,3}", &[]);

    // Names built before the freeze must keep working with names built after.
    let existing: Vec<Name> = ["a", "b", "ab", "ba", "abc", "frozenTestSchemaName"]
        .into_iter()
        .flat_map(|value| KINDS.map(|kind| build(value, kind)))
        .collect();

    names_interned_during_freeze_stay_interned();
    frozen_name_semantics();

    // Strings containing `d` or `e` were never interned, so this mixes
    // table hits and frozen misses.
    name_invariants(&config, "[a-e]{1,3}", &existing);
}

fn names_interned_during_freeze_stay_interned() {
    let stop = AtomicBool::new(false);
    let created: Vec<Vec<(String, Option<&'static str>)>> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..8)
            .map(|t| {
                let stop = &stop;
                scope.spawn(move || {
                    let mut out = Vec::new();
                    let mut i = 0;
                    while !stop.load(Ordering::Relaxed) || i < 2000 {
                        let s = format!("raceT{t}N{i}");
                        let n = Name::new(&s).unwrap();
                        out.push((s, n.as_static_str()));
                        i += 1;
                    }
                    out
                })
            })
            .collect();
        std::thread::sleep(std::time::Duration::from_millis(2));
        apollo_compiler::freeze_interning();
        stop.store(true, Ordering::Relaxed);
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    for (s, ptr) in created.into_iter().flatten() {
        let Some(ptr) = ptr else { continue };
        let again = Name::new(&s).unwrap();
        assert_eq!(
            again.as_static_str().map(str::as_ptr),
            Some(ptr.as_ptr()),
            "{s} was interned before freeze completed but is missing from the frozen table"
        );
    }
}

/// Expects the table to be frozen with "frozenTestSchemaName" interned.
fn frozen_name_semantics() {
    fn hash_of(value: &impl Hash) -> u64 {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let schema_name = Name::new("frozenTestSchemaName").unwrap();
    assert!(schema_name.as_static_str().is_some());

    // Post-freeze misses (hostile-alias shaped): equality and hashing fall
    // back to strings, consistently between instances.
    let miss_a = Name::new("frozenTestNotInterned").unwrap();
    let miss_b = Name::new("frozenTestNotInterned").unwrap();
    assert!(miss_a.as_static_str().is_none());
    assert_eq!(miss_a, miss_b);
    assert_eq!(hash_of(&miss_a), hash_of(&miss_b));
    assert_ne!(schema_name, miss_a);

    // Maps keyed by both kinds of name work.
    let mut map = IndexMap::default();
    map.insert(schema_name.clone(), 1);
    map.insert(miss_a.clone(), 2);
    assert_eq!(map.get(&miss_b), Some(&2));
    assert_eq!(map.get(&name!("frozenTestSchemaName")), Some(&1));
    assert_eq!(map.get(&name!("frozenTestNotInterned")), Some(&2));
    assert_eq!(map.get(&name!("frozenTestAbsent")), None);

    // Literal names are not interned and compare by string with both kinds.
    assert_eq!(name!("frozenTestSchemaName"), schema_name);
    assert_eq!(name!("frozenTestNotInterned"), miss_a);

    // An `Arc` whose string is already interned is swapped for the table's copy.
    let from_arc = Name::try_from(Arc::<str>::from("frozenTestSchemaName")).unwrap();
    assert_eq!(from_arc.as_static_str(), schema_name.as_static_str());
}

#[derive(Clone, Copy, Debug)]
enum Kind {
    New,
    Arc,
    Static,
}

const KINDS: [Kind; 3] = [Kind::New, Kind::Arc, Kind::Static];

fn build(value: &str, kind: Kind) -> Name {
    match kind {
        Kind::New => Name::new(value).expect("valid name"),
        Kind::Arc => Name::try_from(Arc::<str>::from(value)).expect("valid name"),
        // Leaked like a `name!` literal would be, to get a `TAG_STATIC` name
        // whose string isn't the intern table's copy.
        Kind::Static => Name::new_static(Box::leak(Box::<str>::from(value))).expect("valid name"),
    }
}

/// Checks equality, ordering, hashing and map lookups across every way a
/// `Name` can be built, for random strings drawn from `alphabet`.
fn name_invariants(config: &Config, alphabet: &'static str, existing: &[Name]) {
    let kind = prop_oneof![Just(Kind::New), Just(Kind::Arc), Just(Kind::Static)];
    // Short strings from a small alphabet so that equal strings built
    // different ways are common.
    let entries = prop::collection::vec((alphabet, kind), 1..24);
    TestRunner::new(config.clone())
        .run(&entries, |entries| check_names(&entries, existing))
        .expect("name invariants");
}

fn check_names(entries: &[(String, Kind)], existing: &[Name]) -> Result<(), TestCaseError> {
    let state = ahash::RandomState::new();
    let mut all: Vec<Name> = existing.to_vec();
    all.extend(entries.iter().map(|(value, kind)| build(value, *kind)));

    for a in &all {
        for b in &all {
            prop_assert_eq!(a == b, a.as_str() == b.as_str(), "{:?} vs {:?}", a, b);
            prop_assert_eq!(a.cmp(b), a.as_str().cmp(b.as_str()));
            if a == b {
                prop_assert_eq!(state.hash_one(a), state.hash_one(b), "{:?}", a);
            }
        }
    }

    // A `Name`-keyed map must behave like a `String`-keyed one, whichever
    // kind of `Name` is used to insert and to look up.
    let mut map = IndexMap::default();
    let mut reference = HashMap::new();
    for (i, name) in all.iter().enumerate() {
        map.insert(name.clone(), i);
        reference.insert(name.as_str().to_owned(), i);
    }
    prop_assert_eq!(map.len(), reference.len());
    for (value, _) in entries {
        for kind in KINDS {
            let key = build(value, kind);
            prop_assert_eq!(map.get(&key), reference.get(value), "{:?}", key);
        }
    }
    Ok(())
}
