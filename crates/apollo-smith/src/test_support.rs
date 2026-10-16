//! Helpers for the property tests in each generator module.

use crate::DocumentBuilder;
use apollo_compiler::ast;
use arbitrary::Result;
use arbitrary::Unstructured;
use proptest::prelude::*;

pub(crate) fn input() -> impl Strategy<Value = Vec<u8>> {
    prop::collection::vec(any::<u8>(), 0..=2048)
}

/// Run `generate` on a builder that starts from `schema`, and return the
/// schema with the generated definition added, or `None` when nothing was
/// generated.
pub(crate) fn generate<D: Into<ast::Definition>>(
    schema: &str,
    data: &[u8],
    generate: impl FnOnce(&mut DocumentBuilder<'_>) -> Result<Option<D>>,
) -> Option<String> {
    let mut u = Unstructured::new(data);
    let doc = apollo_parser::Parser::new(schema)
        .parse()
        .document()
        .try_into()
        .expect("schema converts to a smith document");
    let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
    let definition = generate(&mut builder).ok()??;
    let mut generated = ast::Document::new();
    generated.definitions.push(definition.into());
    Some(format!("{schema}\n{generated}"))
}

/// Fail the current case when `doc` has validation errors.
pub(crate) fn prop_assert_valid(doc: &str) -> std::result::Result<(), TestCaseError> {
    let errors: Vec<String> = match ast::Document::parse(doc, "smith.graphql")
        .expect("smith output must parse")
        .to_mixed_validate()
    {
        Ok(_) => vec![],
        Err(errors) => errors
            .iter()
            .map(|e| e.error.to_string())
            // A test generates one definition, so fragments in its schema
            // are only used when that definition happens to spread them.
            .filter(|m| !m.contains("must be used in an operation"))
            .collect(),
    };
    prop_assert!(errors.is_empty(), "{doc}\n{errors:#?}");
    Ok(())
}
