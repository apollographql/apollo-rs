//! Semantic-invariant fuzz target for `@oneOf` input objects.
//!
//! Exercises two layers of invariants from a single fuzz input:
//!
//! ## Schema invariants (spec §3.10.1)
//! If `Schema::parse_and_validate` succeeds, every `@oneOf` input type must:
//!   1. Have all fields nullable.
//!   2. Have no field with a default value.
//!   3. Return `is_one_of() == true` iff the `@oneOf` directive is present.
//!
//! ## Executable-document invariants (spec §5.6.3)
//! The input is also parsed as a mixed schema+executable document via
//! `to_mixed_validate()`.  If that succeeds, we verify that the
//! `coerce_variable_values` runtime path also rejects invalid @oneOf usage
//! by attempting to coerce an empty variable map — any @oneOf variable whose
//! type is nullable should already have been caught by document validation, so
//! coercion of a valid document must not panic.
#![no_main]
use apollo_compiler::ast::Document;
use apollo_compiler::Schema;
use libfuzzer_sys::fuzz_target;
use log::debug;

fuzz_target!(|data: &str| {
    let _ = env_logger::try_init();
    debug!("{data}");

    // -----------------------------------------------------------------------
    // Schema invariants
    // -----------------------------------------------------------------------
    if let Ok(schema) = Schema::parse_and_validate(data, "fuzz.graphql") {
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
    }

    // -----------------------------------------------------------------------
    // Executable-document invariants
    //
    // Parse the same input as a mixed schema+executable document. If it
    // fully validates, exercise the runtime coercion path to confirm it
    // doesn't panic on a valid (empty) variable map.
    // -----------------------------------------------------------------------
    let doc = Document::parse(data, "fuzz.graphql").unwrap_or_else(|invalid| invalid.partial);
    let Ok((schema, executable)) = doc.to_mixed_validate() else {
        return;
    };

    // Invariant: a fully-validated document must not panic during coercion
    // with an empty variable map (all nullable/optional variables).
    for op in executable.operations.iter() {
        // Only attempt coercion when all variables are optional (nullable with
        // no required value), to avoid expected "missing required variable"
        // errors that are not @oneOf bugs.
        let all_optional = op
            .variables
            .iter()
            .all(|v| !v.ty.is_non_null() && v.default_value.is_none());
        if all_optional {
            let empty = apollo_compiler::response::JsonMap::default();
            let _ = apollo_compiler::request::coerce_variable_values(&schema, op, &empty);
        }
    }
});
