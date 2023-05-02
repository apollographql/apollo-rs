#![no_main]
use apollo_encoder::StringValue;
use apollo_parser::{ast, Parser};
use apollo_rs_fuzz::log_gql_doc;
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};
use log::debug;
use std::panic;

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let string = String::arbitrary(&mut u).unwrap();
    let doc_generated = format!(
        "{}\nscalar DummyDefinition",
        StringValue::Top {
            source: string.clone()
        }
    );

    let parser = Parser::new(&doc_generated);
    let tree = parser.parse();

    debug!("======= DOCUMENT =======");
    debug!("{}", doc_generated);
    debug!("========================");

    let mut should_panic = false;

    if tree.errors().len() > 0 {
        should_panic = true;
        let errors = tree
            .errors()
            .map(|err| err.message())
            .collect::<Vec<&str>>()
            .join("\n");
        debug!("Parser errors ========== \n{:?}", errors);
        debug!("========================");
        log_gql_doc(&doc_generated, &errors);
    }

    let scalar_def = tree.document().definitions().next().unwrap();
    let ast::Definition::ScalarTypeDefinition(scalar_def) = scalar_def else {
        panic!("parser produced wrong node type");
    };
    let description = scalar_def.description().unwrap();
    let reparsed_string = String::from(description.string_value().unwrap());

    assert_eq!(reparsed_string, string);

    if should_panic {
        panic!("error detected");
    }
});
