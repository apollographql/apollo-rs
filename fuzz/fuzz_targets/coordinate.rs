#![no_main]
use apollo_compiler::coordinate::SchemaCoordinate;
use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &[u8]| {
    let s = String::arbitrary(&mut Unstructured::new(data)).unwrap();

    let coord = s.parse::<SchemaCoordinate>();
    if let Ok(coord) = &coord {
        assert_eq!(
            &coord.to_string().parse::<SchemaCoordinate>().unwrap(),
            coord
        );
    }

    debug!("{:?}", coord);
});
