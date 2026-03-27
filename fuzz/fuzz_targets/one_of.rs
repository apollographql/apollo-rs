//! Structure-aware fuzz target for `@oneOf` input object invariants.
//!
//! Uses `apollo-smith` to generate *valid* GraphQL documents (schema +
//! executable) from raw bytes, then asserts that every `@oneOf` input type
//! in the resulting schema satisfies the spec invariants:
//!
//!   1. All fields are nullable.
//!   2. No field has a default value.
//!   3. `InputObjectType::is_one_of()` agrees with directive presence.
//!
//! Because the input is always a structurally valid document, the fuzzer
//! spends its budget exploring interesting schema shapes rather than
//! discarding malformed SDL — giving much better coverage of the validation
//! logic than a plain-text target would.
#![no_main]
use apollo_compiler::Schema;
use apollo_rs_fuzz::generate_valid_document;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &[u8]| {
    let _ = env_logger::try_init();

    let Ok(doc) = generate_valid_document(data) else {
        return;
    };
    debug!("{doc}");

    let Ok(schema) = Schema::parse_and_validate(&doc, "fuzz.graphql") else {
        return;
    };

    for (type_name, ty) in &schema.types {
        let apollo_compiler::schema::ExtendedType::InputObject(input_obj) = ty else {
            continue;
        };

        let has_directive = input_obj.directives.get("oneOf").is_some();
        let is_one_of = input_obj.is_one_of();

        // Invariant: is_one_of() must agree with directive presence.
        assert_eq!(
            has_directive, is_one_of,
            "is_one_of() disagrees with @oneOf directive presence for type `{type_name}`"
        );

        if is_one_of {
            for (field_name, field) in &input_obj.fields {
                // Invariant 1: all fields of a @oneOf type must be nullable.
                assert!(
                    !field.ty.is_non_null(),
                    "@oneOf type `{type_name}` field `{field_name}` must be nullable \
                     but has type `{}`",
                    field.ty
                );

                // Invariant 2: no field may carry a default value.
                assert!(
                    field.default_value.is_none(),
                    "@oneOf type `{type_name}` field `{field_name}` must not have a default value"
                );
            }
        }
    }
});
