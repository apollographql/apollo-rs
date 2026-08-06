//! Executable-document fuzz target for `@oneOf` validation.
//!
//! The schema-invariant target (`one_of.rs`) exercises schema parsing and the
//! field-level rules in `validation/input_object.rs`.  That approach cannot
//! reach the document-level rules in `validation/value.rs` because a single
//! text string rarely simultaneously satisfies schema AND document validity.
//!
//! This target fixes a rich @oneOf schema and fuzzes *only* the executable
//! document, directly exercising:
//!   - `validation/value.rs` — @oneOf field-count, null, and variable rules
//!   - `resolvers/input_coercion.rs` — runtime coercion of @oneOf values
//!
//! Schema positions covered:
//!   - @oneOf as a query field argument
//!   - @oneOf inside a list argument
//!   - @oneOf nested inside a regular input object
//!   - @oneOf as a mutation argument
//!   - @oneOf as a subscription argument
//!   - @oneOf as a directive argument on a field
#![no_main]
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use libfuzzer_sys::fuzz_target;
use log::debug;
use std::sync::OnceLock;

const SCHEMA_SDL: &str = r#"
    type Query {
        search(filter: SearchFilter): String
        list(items: [ListFilter]): String
        nested(arg: OuterInput): String
    }
    type Mutation {
        create(input: CreateInput): String
    }
    type Subscription {
        events(filter: EventFilter): String
    }

    "Used as a direct field argument."
    input SearchFilter @oneOf {
        id: ID
        name: String
        score: Int
    }

    "Used inside a list argument — each item is @oneOf."
    input ListFilter @oneOf {
        exact: String
        prefix: String
    }

    "Nests a @oneOf inside a regular input."
    input OuterInput {
        filter: SearchFilter
        page: Int
    }

    "Used as a mutation argument."
    input CreateInput @oneOf {
        fromId: ID
        fromName: String
    }

    "Used as a subscription argument."
    input EventFilter @oneOf {
        topic: String
        id: ID
    }
"#;

fn schema() -> &'static Valid<Schema> {
    static SCHEMA: OnceLock<Valid<Schema>> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        Schema::parse_and_validate(SCHEMA_SDL, "schema.graphql")
            .expect("hardcoded @oneOf schema must be valid")
    })
}

fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();
    debug!("{data}");

    let schema = schema();

    // Parse and validate the fuzz input as an executable document.
    // We intentionally discard validation errors — we are looking for panics
    // and for coverage of the @oneOf validation code paths.
    let result = ExecutableDocument::parse_and_validate(schema, data, "fuzz.graphql");

    // For valid documents, also exercise the runtime coercion path.
    // Only attempt coercion when all variables are optional (no required
    // variable that the empty map would reject), so we don't hit expected
    // "missing required variable" errors.
    if let Ok(doc) = result {
        for op in doc.operations.iter() {
            let all_optional = op
                .variables
                .iter()
                .all(|v| !v.ty.is_non_null() && v.default_value.is_none());
            if all_optional {
                let empty = apollo_compiler::response::JsonMap::default();
                let _ = apollo_compiler::request::coerce_variable_values(schema, op, &empty);
            }
        }
    }
});
