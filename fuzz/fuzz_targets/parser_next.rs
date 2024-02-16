#![no_main]

use std::panic;

use libfuzzer_sys::{arbitrary, fuzz_target};

use apollo_smith::next::Error;

fuzz_target!(|data: &[u8]| {
    let _ = env_logger::try_init();

    if let Err(e) = apollo_rs_fuzz::generate_schema_document(data) {
        match &e {
            Error::Arbitrary(arbitrary::Error::NotEnoughData) => {
                return;
            }
            Error::Arbitrary(e) => {
                println!("arbitrary error: {}", e);
            }
            Error::Parse(doc) => {
                println!("{}\ndoc:\n{}\nerrors:\n{}", e, doc.to_string(), doc.errors);
            }
            Error::ExpectedValidationFail { doc, mutation } => {
                println!("{}\nmutation:\n{}\ndoc:\n{}", e, mutation, doc.to_string());
            }
            Error::SerializationInconsistency { original, new } => {
                println!(
                    "{}\noriginal:\n{}\nnew:\n{}",
                    e,
                    original.to_string(),
                    new.to_string()
                );
            }
            Error::SchemaDocumentValidation { doc, errors } => {
                println!(
                    "{}\ndoc:\n{}\nerrors:\n{}",
                    e,
                    doc.to_string(),
                    errors.errors
                );
            }
            Error::Reparse { doc, errors } => {
                println!(
                    "{}\ndoc:\n{}\nerrors:\n{}",
                    e,
                    doc.to_string(),
                    errors.errors
                );
            }
            Error::ExecutableDocumentValidation {
                doc,
                schema,
                errors,
            } => {
                println!(
                    "{}\nschena\n{}\ndoc:\n{}\nerrors:\n{}",
                    e,
                    schema.to_string(),
                    doc.to_string(),
                    errors.errors
                );
            }
        }
        panic!("error detected: {}", e);
    }
});
