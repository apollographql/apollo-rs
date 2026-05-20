//! A type that declares `implements X` must declare every field `X`
//! declares, and any interface `X` transitively implements. This test
//! pins two failure modes the validator emits when smith hasn't
//! reconciled interface inheritance:
//!
//! - "does not satisfy interface: missing field"
//! - "declares that it implements X, but to do so it must also implement Y"
//!
//! Covariance (the implementer happens to roll a field whose name
//! matches a parent's but with a different type) is out of scope for
//! this branch — leaving it to other generation-time changes.

use apollo_compiler::ast::Document as AstDocument;
use apollo_smith::DocumentBuilder;
use arbitrary::Unstructured;

fn assert_implementations_satisfied(seed: &[u8]) {
    let mut u = Unstructured::new(seed);
    let doc: String = DocumentBuilder::new(&mut u).unwrap().finish().into();
    let ast = match AstDocument::parse(&doc, "smith.graphql") {
        Ok(d) => d,
        Err(e) => panic!("smith output must parse: {e}\n\n=== DOC ===\n{doc}\n=== END ==="),
    };

    if let Err(errors) = ast.to_mixed_validate() {
        let errors_str = errors.to_string();
        for diag in [
            "does not satisfy interface",
            "but to do so it must also implement",
        ] {
            assert!(
                !errors_str.contains(diag),
                "seed={seed:?}: smith produced `{diag}`\n\nErrors:\n{errors_str}\n\n=== DOC ===\n{doc}\n=== END ==="
            );
        }
    }
}

#[test]
fn empty_seed() {
    assert_implementations_satisfied(&[]);
}

#[test]
fn many_random_seeds() {
    use rand::rngs::StdRng;
    use rand::Rng as _;
    use rand::SeedableRng;
    for seed in 0u64..2000 {
        let mut rng = StdRng::seed_from_u64(seed);
        let mut buf = vec![0u8; 4096];
        rng.fill_bytes(&mut buf);
        assert_implementations_satisfied(&buf);
    }
}

/// Random byte streams rarely exercise corners like extension-extension
/// field-name collisions, multi-interface cycles, or conflicting parent
/// signatures. The libFuzzer-evolved corpus reaches those paths reliably.
#[test]
fn fuzz_corpus_inputs() {
    let corpus =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fuzz/corpus/validate");
    let Ok(entries) = std::fs::read_dir(&corpus) else {
        eprintln!("skipping: corpus {} not present", corpus.display());
        return;
    };
    for entry in entries.flatten() {
        let Ok(bytes) = std::fs::read(entry.path()) else {
            continue;
        };
        assert_implementations_satisfied(&bytes);
    }
}
