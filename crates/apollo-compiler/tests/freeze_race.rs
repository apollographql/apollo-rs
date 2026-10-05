use apollo_compiler::Name;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[test]
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
