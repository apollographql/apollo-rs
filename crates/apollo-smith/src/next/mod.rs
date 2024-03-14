use std::path::PathBuf;

use apollo_compiler::ast::Document;
use apollo_compiler::validation::{Valid, WithErrors};
use apollo_compiler::{ExecutableDocument, Schema};

pub use crate::next::unstructured::Unstructured;

mod ast;
mod mutations;
mod schema;
mod unstructured;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("arbitrary error")]
    Arbitrary(#[from] arbitrary::Error),

    #[error("schema document validation failed")]
    SchemaDocumentValidation {
        doc: Document,
        errors: WithErrors<Schema>,
    },

    #[error("executable document validation failed")]
    ExecutableDocumentValidation {
        doc: Document,
        schema: Valid<Schema>,
        errors: WithErrors<ExecutableDocument>,
    },

    #[error("validation passed, but should have failed")]
    SchemaExpectedValidationFail { doc: Document, mutation: String },

    #[error("the serialized AST did not round trip to an identical AST")]
    SerializationInconsistency { original: Document, new: Document },

    #[error("parse error")]
    Parse(WithErrors<Document>),

    #[error("validation passed, but should have failed")]
    ExecutableExpectedValidationFail {
        schema: Valid<Schema>,
        doc: Document,
        mutation: String
    },

    #[error("reparse error")]
    SchemaReparse {
        doc: Document,
        errors: WithErrors<Document>,
    },

    #[error("reparse error")]
    ExecutableReparse {
        schema: Valid<Schema>,
        doc: Document,
        errors: WithErrors<Document>,
    },
}

pub fn generate_schema_document(u: &mut Unstructured) -> Result<Document, Error> {
    let mut doc = Document::parse(
        "type Query { me: String }".to_string(),
        PathBuf::from("synthetic"),
    )
    .map_err(Error::Parse)?; // Start with a minimal schema
    let mutations = mutations::schema_mutations();
    let mut schema = doc.to_schema().expect("initial schema must be valid");
    for _ in 0..1000 {
        if u.len() == 0 {
            // We ran out of data abort. This is not an error
            return Err(Error::Arbitrary(arbitrary::Error::NotEnoughData))?;
        }

        let mutation = u.choose(&mutations)?;
        let mut new_doc = doc.clone();
        // First let's modify the document. We use the schema because it has all the built-in definitions in it.
        if !mutation.apply(u, &mut new_doc, &schema)? {
            // The mutation didn't apply, let's try another one
            continue;
        }


        // Now let's validate that the schema says it's OK
        match (mutation.is_valid(), new_doc.to_schema_validate()) {
            (true, Ok(new_schema)) => {
                // Let's reparse the document to check that it can be parsed
                let reparsed = Document::parse(new_doc.to_string(), PathBuf::from("synthetic"))
                    .map_err(|e| Error::SchemaReparse {
                        doc: new_doc.clone(),
                        errors: e,
                    })?;

                // The reparsed document should be the same as the original document
                if reparsed != new_doc {
                    return Err(Error::SerializationInconsistency {
                        original: new_doc,
                        new: reparsed,
                    });
                }

                // Let's try and create an executable document from the schema
                generate_executable_document(u, &new_schema)?;
                schema = new_schema.into_inner();
                doc = new_doc;
                continue;
            }
            (true, Err(e)) => {
                return Err(Error::SchemaDocumentValidation {
                    doc: new_doc,
                    errors: e,
                });
            }
            (false, Ok(_)) => {
                return Err(Error::SchemaExpectedValidationFail {
                    doc: new_doc,
                    mutation: mutation.type_name().to_string(),
                });
            }
            (false, Err(_)) => {
                // Validation was expected to fail, we can continue using the old doc and schema
                continue;
            }
        }
    }
    Ok(doc)
}

pub(crate) fn generate_executable_document(
    u: &mut Unstructured,
    schema: &Valid<Schema>,
) -> Result<Document, Error> {
    let mut doc = Document::new();
    let mut executable_document = doc.to_executable(schema).expect("initial document must be valid");
    let mutations = mutations::executable_document_mutations();
    for _ in 0..1000 {
        if u.len() == 0 {
            // We ran out of data abort. This is not an error
            return Err(Error::Arbitrary(arbitrary::Error::NotEnoughData))?;
        }
        let mutation = u.choose(&mutations)?;
        let mut new_doc = doc.clone();
        // First let's modify the document.
        if !mutation.apply(u, &mut new_doc, &schema, &executable_document)? {
            // The mutation didn't apply, let's try another one
            continue;
        }

        // Now let's validate that the schema says it's OK
        match (mutation.is_valid(), new_doc.to_executable_validate(schema)) {
            (true, Ok(new_executable_document)) => {
                // Let's reparse the document to check that it can be parsed
                let reparsed = Document::parse(new_doc.to_string(), PathBuf::from("synthetic"))
                    .map_err(|e| Error::ExecutableReparse {
                        schema: schema.clone(),
                        doc: new_doc.clone(),
                        errors: e,
                    })?;

                // The reparsed document should be the same as the original document
                if reparsed != new_doc {
                    return Err(Error::SerializationInconsistency {
                        original: new_doc,
                        new: reparsed,
                    });
                }

                doc = new_doc;
                executable_document = new_executable_document.into_inner();
                continue;
            }
            (true, Err(e)) => {
                return Err(Error::ExecutableDocumentValidation {
                    doc: new_doc,
                    schema: schema.clone(),
                    errors: e,
                });
            }
            (false, Ok(_)) => {
                return Err(Error::SchemaExpectedValidationFail {
                    doc: new_doc,
                    mutation: mutation.type_name().to_string(),
                });
            }
            (false, Err(_)) => {
                // Validation was expected to fail, we can continue using the old doc and schema
                continue;
            }
        }
    }

    Ok(doc)
}
