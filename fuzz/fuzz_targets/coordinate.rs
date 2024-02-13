#![no_main]
use apollo_compiler::coordinate::SchemaCoordinate;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    let coord = data.parse::<SchemaCoordinate>();
    if let Ok(coord) = &coord {
        assert_eq!(
            &coord.to_string().parse::<SchemaCoordinate>().unwrap(),
            coord
        );
    }

    debug!("{:?}", coord);
});
