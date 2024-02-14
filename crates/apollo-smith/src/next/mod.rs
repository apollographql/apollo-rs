use std::any::type_name;
use std::path::PathBuf;

use arbitrary::Unstructured;

use apollo_compiler::ast::Document;
use apollo_compiler::validation::WithErrors;
use apollo_compiler::Schema;

/// macro for accessing fields
macro_rules! field_access {
    () => {
        fn random_field(
            &self,
            u: &mut Unstructured,
        ) -> arbitrary::Result<&Node<apollo_compiler::ast::FieldDefinition>> {
            Ok(u.choose(&self.target().fields).map_err(|e| {
                if let arbitrary::Error::EmptyChoose = e {
                    panic!("no existing fields")
                } else {
                    e
                }
            })?)
        }

        fn random_field_mut(
            &mut self,
            u: &mut Unstructured,
        ) -> arbitrary::Result<&mut Node<apollo_compiler::ast::FieldDefinition>> {
            let idx = u.choose_index(self.target().fields.len()).map_err(|e| {
                if let arbitrary::Error::EmptyChoose = e {
                    panic!("no existing fields")
                } else {
                    e
                }
            })?;
            Ok(&mut self.target_mut().fields[idx])
        }

        fn sample_fields(
            &self,
            u: &mut Unstructured,
        ) -> arbitrary::Result<Vec<&Node<apollo_compiler::ast::FieldDefinition>>> {
            let existing = self
                .target()
                .fields
                .iter()
                .filter(|_| u.arbitrary().unwrap_or(false))
                .collect::<Vec<_>>();

            Ok(existing)
        }
    };
}
mod ast;
mod common;
mod mutations;
mod schema;
mod unstructured;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("arbitrary error")]
    Arbitrary(#[from] arbitrary::Error),

    #[error("schema validation")]
    Validation(WithErrors<Schema>),

    #[error("schema validation passed, but should have failed")]
    ExpectedValidationFail(Document),

    #[error("parse error")]
    Parse(WithErrors<Document>),
}

pub fn generate_schema_document(input: &[u8]) -> Result<Document, Error> {
    let mut u = Unstructured::new(input);
    println!("starting");
    let mut doc = Document::parse(
        "type Query { me: String }".to_string(),
        PathBuf::from("synthetic"),
    )
    .map_err(Error::Parse)?; // Start with a minimal schema
    println!("parsed initial");
    let mutations = mutations::all_mutations();
    let mut schema = doc.to_schema().expect("initial schema must be valid");
    for n in 0..1000 {
        println!("iteration: {}", n);
        let mut mutation = u.choose(&mutations)?;
        println!("applying mutation: {} ", mutation.type_name());
        // First let's modify the document. We use the schema because it has all the built-in definitions in it.
        mutation.apply(&mut u, &mut doc, &schema)?;

        // Let's reparse the document to check that it can be parsed
        Document::parse(doc.to_string(), PathBuf::from("synthetic")).map_err(Error::Parse)?;
        // Now let's validate that the schema says it's OK

        println!("{}", doc.to_string());

        match (mutation.is_valid(), doc.to_schema_validate()) {
            (true, Ok(new_schema)) => {
                schema = new_schema.into_inner();
                continue;
            }
            (true, Err(e)) => {
                return Err(Error::Validation(e));
            }
            (false, Ok(_)) => {
                return Err(Error::ExpectedValidationFail(doc));
            }
            (false, Err(_)) => {
                // Validation was expected to fail, we can't continue
                return Ok(doc);
            }
        }
    }

    Ok(doc)
}

#[cfg(test)]
mod test {
    use crate::next::{generate_schema_document, Error};
    use apollo_compiler::ast::Document;
    use apollo_compiler::Schema;

    #[test]
    fn test_schema() {
        let f = Schema::builder().add_ast(&Document::new()).build().unwrap();
        println!("{:?}", f.types.len());
    }

    #[test]
    fn test() {
        let input = b"293ur928jff029jf0293f";
        match generate_schema_document(input) {
            Ok(_) => {}
            Err(e) => {
                panic!("error: {:?}", e)
            }
        }
    }
}
