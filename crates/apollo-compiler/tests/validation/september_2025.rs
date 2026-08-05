//! Tests for type-system validation rules added in the September 2025 edition
//! of the GraphQL specification.

use apollo_compiler::Schema;
use expect_test::expect;
use expect_test::Expect;

#[track_caller]
fn expect_schema_errors(sdl: &'static str, expect: Expect) {
    let errors = Schema::parse_and_validate(sdl, "schema.graphql")
        .expect_err("should have errors")
        .errors;
    expect.assert_eq(&errors.to_string());
}

#[track_caller]
fn expect_valid_schema(sdl: &'static str) {
    Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
}

/// https://spec.graphql.org/September2025/#sec--deprecated
/// > The @deprecated directive must not appear on required (non-null without a
/// > default) arguments or input object field definitions.
mod deprecated_required_input_values {
    use super::*;

    #[test]
    fn required_argument_cannot_be_deprecated() {
        expect_schema_errors(
            r#"
            type Query {
                field(arg: Int! @deprecated): String
            }
            "#,
            expect![[r#"
                Error: an argument `arg` is required (non-null without a default value) and must not be deprecated
                   ╭─[ schema.graphql:3:33 ]
                   │
                 3 │                 field(arg: Int! @deprecated): String
                   │                       ──────────┬────┬─────  
                   │                                 ╰──────────── an argument `arg` defined as required here
                   │                                      │       
                   │                                      ╰─────── `@deprecated` used here
                   │ 
                   │ Help: make the type nullable or add a default value, or remove `@deprecated`.
                ───╯
            "#]],
        );
    }

    #[test]
    fn required_input_field_cannot_be_deprecated() {
        expect_schema_errors(
            r#"
            type Query { field(arg: In): String }
            input In {
                a: Int! @deprecated
            }
            "#,
            expect![[r#"
                Error: an input object field `a` is required (non-null without a default value) and must not be deprecated
                   ╭─[ schema.graphql:4:25 ]
                   │
                 4 │                 a: Int! @deprecated
                   │                 ─────────┬───┬─────  
                   │                          ╰─────────── an input object field `a` defined as required here
                   │                              │       
                   │                              ╰─────── `@deprecated` used here
                   │ 
                   │ Help: make the type nullable or add a default value, or remove `@deprecated`.
                ───╯
            "#]],
        );
    }

    #[test]
    fn optional_input_values_can_be_deprecated() {
        expect_valid_schema(
            r#"
            type Query {
                field(nullable: Int @deprecated, defaulted: Int! = 0 @deprecated, arg: In): String
            }
            input In {
                a: Int @deprecated
                b: Int! = 3 @deprecated
            }
            "#,
        );
    }
}

/// https://spec.graphql.org/September2025/#IsValidImplementation()
/// > If field is deprecated then implementedField must also be deprecated.
mod deprecated_implementation_fields {
    use super::*;

    #[test]
    fn implementing_field_cannot_be_deprecated_alone() {
        expect_schema_errors(
            r#"
            interface I {
                a: Int
            }
            type Query implements I {
                a: Int @deprecated
            }
            "#,
            expect![[r#"
                Error: field `Query.a` is deprecated but implements the non-deprecated interface field `I.a`
                   ╭─[ schema.graphql:6:17 ]
                   │
                 6 │                 a: Int @deprecated
                   │                 ─────────┬────────  
                   │                          ╰────────── this field is deprecated
                   │
                   ├─[ schema.graphql:6:17 ]
                   │
                 3 │                 a: Int
                   │                 ───┬──  
                   │                    ╰──── `I.a` is not deprecated
                   │ 
                   │ Help: deprecate `I.a` as well, or remove `@deprecated` from the implementing field.
                ───╯
            "#]],
        );
    }

    #[test]
    fn deprecated_with_deprecated_interface_field_is_valid() {
        expect_valid_schema(
            r#"
            interface I {
                a: Int @deprecated
                b: Int
            }
            type Query implements I {
                a: Int @deprecated
                b: Int
            }
            "#,
        );
    }
}

/// https://spec.graphql.org/September2025/#InputObjectDefaultValueHasCycle()
mod input_object_default_value_cycles {
    use super::*;

    #[test]
    fn self_referential_default_is_invalid() {
        expect_schema_errors(
            r#"
            type Query { field(arg: A): String }
            input A {
                self: A = {}
            }
            "#,
            expect![[r#"
                Error: the default value of input object field `A.self` cycles back to itself
                   ╭─[ schema.graphql:4:17 ]
                   │
                 4 │                 self: A = {}
                   │                           ─┬  
                   │                            ╰── coercing this default value requires coercing it again
                   │ 
                   │ Help: break the cycle by providing an explicit value for one of the fields involved, or by removing a default value.
                ───╯
            "#]],
        );
    }

    #[test]
    fn mutually_recursive_defaults_are_invalid() {
        expect_schema_errors(
            r#"
            type Query { field(arg: A): String }
            input A {
                b: B = {}
            }
            input B {
                a: A = {}
            }
            "#,
            expect![[r#"
                Error: the default value of input object field `A.b` cycles back to itself
                   ╭─[ schema.graphql:4:17 ]
                   │
                 4 │                 b: B = {}
                   │                        ─┬  
                   │                         ╰── coercing this default value requires coercing it again
                   │ 
                   │ Help: break the cycle by providing an explicit value for one of the fields involved, or by removing a default value.
                ───╯
                Error: the default value of input object field `B.a` cycles back to itself
                   ╭─[ schema.graphql:7:17 ]
                   │
                 7 │                 a: A = {}
                   │                        ─┬  
                   │                         ╰── coercing this default value requires coercing it again
                   │ 
                   │ Help: break the cycle by providing an explicit value for one of the fields involved, or by removing a default value.
                ───╯
            "#]],
        );
    }

    #[test]
    fn explicit_value_breaks_the_cycle() {
        expect_valid_schema(
            r#"
            type Query { field(arg: A): String }
            input A {
                self: A = { self: null }
                other: Int
            }
            "#,
        );
    }

    #[test]
    fn defaults_without_cycles_are_valid() {
        expect_valid_schema(
            r#"
            type Query { field(arg: A): String }
            input A {
                b: B = {}
                x: Int = 4
            }
            input B {
                y: Int = 2
            }
            "#,
        );
    }
}
