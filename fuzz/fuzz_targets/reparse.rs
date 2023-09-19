#![no_main]

use apollo_compiler::ast::Document;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use libfuzzer_sys::fuzz_target;
use log::debug;
use log::trace;
use std::fmt::Debug;

fuzz_target!(|input: &str| {
    let _ = env_logger::try_init();
    debug!("{input}");

    let doc = Document::parse(input, "original.graphql");
    debug!("=> AST:\n{doc:#?}");

    let serialized = doc.to_string();
    debug!("=> AST:\n{serialized}");

    let doc2 = Document::parse(&serialized, "reparsed.graphql");
    debug!("=> AST reparsed:\n{doc2:#?}");
    debug!("=> AST reparsed:\n{doc2}");
    if doc != doc2 {
        diff(&doc, "AST", &doc2, "AST reparsed");
        panic!(
            "Serialized and reparsed to a different AST \
             (run with RUST_LOG=debug or trace for details)"
        )
    }

    let (schema, executable) = doc.to_mixed();
    let schema_serialized = schema.to_string();
    let executable_serialized = schema.to_string();

    let schema2 = Schema::parse(&schema_serialized, "schema_reparsed.graphql");
    // if schema != schema2 {
    //     trace!("=> Schema:\n{schema:#?}");
    //     debug!("=> Schema:\n{schema_serialized}");
    //     trace!("=> Schema reparsed:\n{schema2:#?}");
    //     debug!("=> Schema reparsed:\n{schema2}");
    //     diff(&schema, "Schema", &schema2, "Schema reparsed");
    //     panic!(
    //         "Serialized and reparsed to a different schema \
    //          (run with RUST_LOG=debug or trace for details)"
    //     )
    // }

    let executable2 = ExecutableDocument::parse(
        &schema2,
        &executable.to_string(),
        "executable_reparsed.graphql",
    );
    if executable != executable2 {
        trace!("=> Executable document:\n{executable:?}");
        debug!("=> Executable document:\n{executable_serialized}");
        trace!("=> Executable document reparsed:\n{executable2:?}");
        debug!("=> Executable document reparsed:\n{executable2}");
        diff(
            &executable,
            "Executable",
            &executable2,
            "Executable reparsed",
        );
        panic!(
            "Serialized and reparsed to a different executable document \
             (run with RUST_LOG=debug or trace for details)"
        )
    }
});

fn diff(left: impl Debug, left_label: &'static str, right: impl Debug, right_label: &'static str) {
    println!(
        "{}",
        similar_asserts::SimpleDiff::from_str(
            &format!("{:#?}", left),
            &format!("{:#?}", right),
            left_label,
            right_label
        )
    );
}
