#![no_main]
use apollo_compiler::name;
use apollo_compiler::Schema;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();

    let mut input = Schema::new();
    let def = input.schema_definition.make_mut();
    def.description = Some(data.to_owned().into());
    // We can refer to a type that doesn't exist as we won't run validation
    def.query = Some(name!("Dangling").into());
    let doc_generated = input.to_string();

    debug!("INPUT STRING: {:?}", input.schema_definition.description);
    debug!("==== WHOLE DOCUMENT ====");
    debug!("{doc_generated}");
    debug!("========================");

    let reparse = Schema::parse(doc_generated, "").unwrap();
    debug!(
        "REPARSED STRING: {:?}",
        reparse.schema_definition.description
    );

    assert_eq!(
        reparse.schema_definition.description,
        input.schema_definition.description
    );
});
