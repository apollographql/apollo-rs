#![no_main]

use apollo_compiler::ast::Document;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use libfuzzer_sys::fuzz_target;
use log::debug;
use log::trace;
use std::fmt::Debug;

const ENABLE_EXECUTABLE: bool = true;
const ENABLE_SCHEMA: bool = true;

fuzz_target!(|input: &str| {
    let _ = env_logger::try_init();
    debug!("{input}");

    let doc = Document::parse(input, "original.graphql").unwrap_or_else(|invalid| invalid.partial);
    debug!("=> AST:\n{doc:#?}");

    let serialized = doc.to_string();
    debug!("=> AST:\n{serialized}");

    let doc2 = Document::parse(&serialized, "reparsed.graphql").unwrap();
    debug!("=> AST reparsed:\n{doc2:#?}");
    debug!("=> AST reparsed:\n{doc2}");
    if doc != doc2 {
        diff(&doc, "AST", &doc2, "AST reparsed");
        panic!(
            "Serialized and reparsed to a different AST \
             (run with RUST_LOG=debug or trace for details)"
        )
    }
    if ENABLE_SCHEMA || ENABLE_EXECUTABLE {
        let Ok((schema, executable)) = doc.to_mixed_validate() else {
            return;
        };
        let schema_serialized = schema.to_string();
        let executable_serialized = executable.to_string();

        let schema2 =
            Schema::parse_and_validate(&schema_serialized, "schema_reparsed.graphql").unwrap();
        if ENABLE_SCHEMA && schema != schema2 {
            trace!("=> Schema:\n{schema:#?}");
            debug!("=> Schema:\n{schema_serialized}");
            trace!("=> Schema reparsed:\n{schema2:#?}");
            debug!("=> Schema reparsed:\n{schema2}");
            diff(&schema, "Schema", &schema2, "Schema reparsed");
            panic!(
                "Serialized and reparsed to a different schema \
                 (run with RUST_LOG=debug or trace for details)"
            )
        }

        if ENABLE_EXECUTABLE {
            let executable2 = ExecutableDocument::parse_and_validate(
                &schema2,
                &executable_serialized,
                "executable_reparsed.graphql",
            )
            .unwrap();
            if executable != executable2 {
                trace!("=> Executable document:\n{executable:#?}");
                debug!("=> Executable document:\n{executable_serialized}");
                trace!("=> Executable document reparsed:\n{executable2:#?}");
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
        }
    }
});

fn diff(left: impl Debug, left_label: &'static str, right: impl Debug, right_label: &'static str) {
    println!(
        "{}",
        similar_asserts::SimpleDiff::from_str(
            &format!("{left:#?}"),
            &format!("{right:#?}"),
            left_label,
            right_label
        )
    );
}
