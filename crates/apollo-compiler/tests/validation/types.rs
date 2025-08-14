//! Ported from graphql-js, 2023-11-16
//! https://github.com/graphql/graphql-js/blob/0b7590f0a2b65e6210da2e49be0d8e6c27781af2/src/validation/__tests__/ValuesOfCorrectTypeRule-test.ts
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::{expect, Expect};
use std::sync::OnceLock;
use unindent::unindent;
use apollo_compiler::schema::SchemaBuilder;

const GRAPHQL_JS_TEST_SCHEMA: &str = r#"
  interface Mammal {
    mother: Mammal
    father: Mammal
  }

  interface Pet {
    name(surname: Boolean): String
  }

  interface Canine implements Mammal {
    name(surname: Boolean): String
    mother: Canine
    father: Canine
  }

  enum DogCommand {
    SIT
    HEEL
    DOWN
  }

  type Dog implements Pet & Mammal & Canine {
    name(surname: Boolean): String
    nickname: String
    barkVolume: Int
    barks: Boolean
    doesKnowCommand(dogCommand: DogCommand): Boolean
    isHouseTrained(atOtherHomes: Boolean = true): Boolean
    isAtLocation(x: Int, y: Int): Boolean
    mother: Dog
    father: Dog
  }

  type Cat implements Pet {
    name(surname: Boolean): String
    nickname: String
    meows: Boolean
    meowsVolume: Int
    furColor: FurColor
  }

  union CatOrDog = Cat | Dog

  type Human {
    name(surname: Boolean): String
    pets: [Pet]
    relatives: [Human]!
  }

  enum FurColor {
    BROWN
    BLACK
    TAN
    SPOTTED
    NO_FUR
    UNKNOWN
  }

  input ComplexInput {
    requiredField: Boolean!
    nonNullField: Boolean! = false
    intField: Int
    stringField: String
    booleanField: Boolean
    stringListField: [String]
  }

  # TODO oneOf not supported in apollo-rs
  input OneOfInput { # @oneOf
    stringField: String
    intField: Int
  }

  type ComplicatedArgs {
    # TODO List
    # TODO Coercion
    # TODO NotNulls
    intArgField(intArg: Int): String
    nonNullIntArgField(nonNullIntArg: Int!): String
    stringArgField(stringArg: String): String
    booleanArgField(booleanArg: Boolean): String
    enumArgField(enumArg: FurColor): String
    floatArgField(floatArg: Float): String
    idArgField(idArg: ID): String
    stringListArgField(stringListArg: [String]): String
    stringListNonNullArgField(stringListNonNullArg: [String!]): String
    complexArgField(complexArg: ComplexInput): String
    oneOfArgField(oneOfArg: OneOfInput): String
    multipleReqs(req1: Int!, req2: Int!): String
    nonNullFieldWithDefault(arg: Int! = 0): String
    multipleOpts(opt1: Int = 0, opt2: Int = 0): String
    multipleOptAndReq(req1: Int!, req2: Int!, opt1: Int = 0, opt2: Int = 0): String
  }

  type QueryRoot {
    human(id: ID): Human
    dog: Dog
    cat: Cat
    pet: Pet
    catOrDog: CatOrDog
    complicatedArgs: ComplicatedArgs
  }

  schema {
    query: QueryRoot
  }

  directive @onField on FIELD
"#;

fn test_schema() -> &'static Valid<Schema> {
    static SCHEMA: OnceLock<Valid<Schema>> = OnceLock::new();

    SCHEMA.get_or_init(|| {
        Schema::parse_and_validate(unindent(GRAPHQL_JS_TEST_SCHEMA), "schema.graphql").unwrap()
    })
}

#[track_caller]
fn expect_valid(query: &'static str) {
    let schema = test_schema();

    ExecutableDocument::parse_and_validate(schema, unindent(query), "query.graphql").unwrap();
}

fn expect_errors(query: &'static str, expect: Expect) {
    let schema = test_schema();

    let errors = ExecutableDocument::parse_and_validate(schema, unindent(query), "query.graphql")
        .expect_err("should have errors")
        .errors;
    expect.assert_eq(&errors.to_string());
}

mod valid_values {
    use super::expect_valid;

    #[test]
    fn good_int_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            intArgField(intArg: 2)
          }
        }
      ",
        );
    }

    #[test]
    fn good_negative_int_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            intArgField(intArg: -2)
          }
        }
      ",
        );
    }

    #[test]
    fn good_boolean_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            booleanArgField(booleanArg: true)
          }
        }
      ",
        );
    }

    #[test]
    fn good_string_value() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            stringArgField(stringArg: "foo")
          }
        }
      "#,
        );
    }

    #[test]
    fn good_float_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            floatArgField(floatArg: 1.1)
          }
        }
      ",
        );
    }

    #[test]
    fn good_negative_float_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            floatArgField(floatArg: -1.1)
          }
        }
      ",
        );
    }

    #[test]
    fn int_into_float() {
        expect_valid(
            "
        {
          complicatedArgs {
            floatArgField(floatArg: 1)
          }
        }
      ",
        );
    }

    #[test]
    fn int_into_id() {
        expect_valid(
            "
        {
          complicatedArgs {
            idArgField(idArg: 1)
          }
        }
      ",
        );
    }

    #[test]
    fn string_into_id() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            idArgField(idArg: "someIdString")
          }
        }
      "#,
        );
    }

    #[test]
    fn good_enum_value() {
        expect_valid(
            "
        {
          dog {
            doesKnowCommand(dogCommand: SIT)
          }
        }
      ",
        );
    }

    #[test]
    fn enum_with_undefined_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            enumArgField(enumArg: UNKNOWN)
          }
        }
      ",
        );
    }

    #[test]
    fn enum_with_null_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            enumArgField(enumArg: NO_FUR)
          }
        }
      ",
        );
    }

    #[test]
    fn null_into_nullable_type() {
        expect_valid(
            "
        {
          complicatedArgs {
            intArgField(intArg: null)
          }
        }
      ",
        );

        // TODO what is this meant to do?
        // expect_valid(
        //     "
        // {
        //   dog(a: null, b: null, c:{ requiredField: true, intField: null }) {
        //     name
        //   }
        // }
        // ",
        // );
    }
}

mod invalid_string_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn int_into_string() {
        expect_errors(
            "
        {
          complicatedArgs {
            stringArgField(stringArg: 1)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type String, found an integer
                    ╭─[ query.graphql:3:31 ]
                    │
                  3 │     stringArgField(stringArg: 1)
                    │                               ┬  
                    │                               ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:80:29 ]
                    │
                 80 │   stringArgField(stringArg: String): String
                    │                             ───┬──  
                    │                                ╰──── expected type declared here as String
                ────╯
            "#]],
        );
    }

    #[test]
    fn float_into_string() {
        expect_errors(
            "
        {
          complicatedArgs {
            stringArgField(stringArg: 1.0)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type String, found a float
                    ╭─[ query.graphql:3:31 ]
                    │
                  3 │     stringArgField(stringArg: 1.0)
                    │                               ─┬─  
                    │                                ╰─── provided value is a float
                    │
                    ├─[ schema.graphql:80:29 ]
                    │
                 80 │   stringArgField(stringArg: String): String
                    │                             ───┬──  
                    │                                ╰──── expected type declared here as String
                ────╯
            "#]],
        );
    }

    #[test]
    fn boolean_into_string() {
        expect_errors(
            "
        {
          complicatedArgs {
            stringArgField(stringArg: true)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type String, found a boolean
                    ╭─[ query.graphql:3:31 ]
                    │
                  3 │     stringArgField(stringArg: true)
                    │                               ──┬─  
                    │                                 ╰─── provided value is a boolean
                    │
                    ├─[ schema.graphql:80:29 ]
                    │
                 80 │   stringArgField(stringArg: String): String
                    │                             ───┬──  
                    │                                ╰──── expected type declared here as String
                ────╯
            "#]],
        );
    }

    #[test]
    fn unquoted_into_string() {
        expect_errors(
            "
        {
          complicatedArgs {
            stringArgField(stringArg: BAR)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type String, found an enum
                    ╭─[ query.graphql:3:31 ]
                    │
                  3 │     stringArgField(stringArg: BAR)
                    │                               ─┬─  
                    │                                ╰─── provided value is an enum
                    │
                    ├─[ schema.graphql:80:29 ]
                    │
                 80 │   stringArgField(stringArg: String): String
                    │                             ───┬──  
                    │                                ╰──── expected type declared here as String
                ────╯
            "#]],
        );
    }
}

mod invalid_int_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn string_into_int() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            intArgField(intArg: "3")
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Int, found a string
                    ╭─[ query.graphql:3:25 ]
                    │
                  3 │     intArgField(intArg: "3")
                    │                         ─┬─  
                    │                          ╰─── provided value is a string
                    │
                    ├─[ schema.graphql:78:23 ]
                    │
                 78 │   intArgField(intArg: Int): String
                    │                       ─┬─  
                    │                        ╰─── expected type declared here as Int
                ────╯
            "#]],
        );
    }

    #[test]
    fn big_int_into_int() {
        expect_errors(
            "
        {
          complicatedArgs {
            intArgField(intArg: 829384293849283498239482938)
          }
        }
      ",
            expect![[r#"
                Error: int cannot represent non 32-bit signed integer value
                   ╭─[ query.graphql:3:25 ]
                   │
                 3 │     intArgField(intArg: 829384293849283498239482938)
                   │                         ─────────────┬─────────────  
                   │                                      ╰─────────────── cannot be coerced to a 32-bit integer
                ───╯
            "#]],
        );
    }

    #[test]
    fn unquoted_string_into_int() {
        expect_errors(
            "
        {
          complicatedArgs {
            intArgField(intArg: FOO)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Int, found an enum
                    ╭─[ query.graphql:3:25 ]
                    │
                  3 │     intArgField(intArg: FOO)
                    │                         ─┬─  
                    │                          ╰─── provided value is an enum
                    │
                    ├─[ schema.graphql:78:23 ]
                    │
                 78 │   intArgField(intArg: Int): String
                    │                       ─┬─  
                    │                        ╰─── expected type declared here as Int
                ────╯
            "#]],
        );
    }

    #[test]
    fn simple_float_into_int() {
        expect_errors(
            "
        {
          complicatedArgs {
            intArgField(intArg: 3.0)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Int, found a float
                    ╭─[ query.graphql:3:25 ]
                    │
                  3 │     intArgField(intArg: 3.0)
                    │                         ─┬─  
                    │                          ╰─── provided value is a float
                    │
                    ├─[ schema.graphql:78:23 ]
                    │
                 78 │   intArgField(intArg: Int): String
                    │                       ─┬─  
                    │                        ╰─── expected type declared here as Int
                ────╯
            "#]],
        );
    }

    #[test]
    fn float_into_int() {
        expect_errors(
            "
        {
          complicatedArgs {
            intArgField(intArg: 3.333)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Int, found a float
                    ╭─[ query.graphql:3:25 ]
                    │
                  3 │     intArgField(intArg: 3.333)
                    │                         ──┬──  
                    │                           ╰──── provided value is a float
                    │
                    ├─[ schema.graphql:78:23 ]
                    │
                 78 │   intArgField(intArg: Int): String
                    │                       ─┬─  
                    │                        ╰─── expected type declared here as Int
                ────╯
            "#]],
        );
    }
}

mod invalid_float_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn string_into_float() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            floatArgField(floatArg: "3.333")
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Float, found a string
                    ╭─[ query.graphql:3:29 ]
                    │
                  3 │     floatArgField(floatArg: "3.333")
                    │                             ───┬───  
                    │                                ╰───── provided value is a string
                    │
                    ├─[ schema.graphql:83:27 ]
                    │
                 83 │   floatArgField(floatArg: Float): String
                    │                           ──┬──  
                    │                             ╰──── expected type declared here as Float
                ────╯
            "#]],
        );
    }

    #[test]
    fn boolean_into_float() {
        expect_errors(
            "
        {
          complicatedArgs {
            floatArgField(floatArg: true)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Float, found a boolean
                    ╭─[ query.graphql:3:29 ]
                    │
                  3 │     floatArgField(floatArg: true)
                    │                             ──┬─  
                    │                               ╰─── provided value is a boolean
                    │
                    ├─[ schema.graphql:83:27 ]
                    │
                 83 │   floatArgField(floatArg: Float): String
                    │                           ──┬──  
                    │                             ╰──── expected type declared here as Float
                ────╯
            "#]],
        );
    }

    #[test]
    fn unquoted_into_float() {
        expect_errors(
            "
        {
          complicatedArgs {
            floatArgField(floatArg: FOO)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Float, found an enum
                    ╭─[ query.graphql:3:29 ]
                    │
                  3 │     floatArgField(floatArg: FOO)
                    │                             ─┬─  
                    │                              ╰─── provided value is an enum
                    │
                    ├─[ schema.graphql:83:27 ]
                    │
                 83 │   floatArgField(floatArg: Float): String
                    │                           ──┬──  
                    │                             ╰──── expected type declared here as Float
                ────╯
            "#]],
        );
    }
}

mod invalid_boolean_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn int_into_boolean() {
        expect_errors(
            "
        {
          complicatedArgs {
            booleanArgField(booleanArg: 2)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Boolean, found an integer
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     booleanArgField(booleanArg: 2)
                    │                                 ┬  
                    │                                 ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:81:31 ]
                    │
                 81 │   booleanArgField(booleanArg: Boolean): String
                    │                               ───┬───  
                    │                                  ╰───── expected type declared here as Boolean
                ────╯
            "#]],
        );
    }

    #[test]
    fn float_into_boolean() {
        expect_errors(
            "
        {
          complicatedArgs {
            booleanArgField(booleanArg: 1.0)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Boolean, found a float
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     booleanArgField(booleanArg: 1.0)
                    │                                 ─┬─  
                    │                                  ╰─── provided value is a float
                    │
                    ├─[ schema.graphql:81:31 ]
                    │
                 81 │   booleanArgField(booleanArg: Boolean): String
                    │                               ───┬───  
                    │                                  ╰───── expected type declared here as Boolean
                ────╯
            "#]],
        );
    }

    #[test]
    fn string_into_boolean() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            booleanArgField(booleanArg: "true")
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Boolean, found a string
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     booleanArgField(booleanArg: "true")
                    │                                 ───┬──  
                    │                                    ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:81:31 ]
                    │
                 81 │   booleanArgField(booleanArg: Boolean): String
                    │                               ───┬───  
                    │                                  ╰───── expected type declared here as Boolean
                ────╯
            "#]],
        );
    }

    #[test]
    fn unquoted_into_boolean() {
        expect_errors(
            "
        {
          complicatedArgs {
            booleanArgField(booleanArg: TRUE)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Boolean, found an enum
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     booleanArgField(booleanArg: TRUE)
                    │                                 ──┬─  
                    │                                   ╰─── provided value is an enum
                    │
                    ├─[ schema.graphql:81:31 ]
                    │
                 81 │   booleanArgField(booleanArg: Boolean): String
                    │                               ───┬───  
                    │                                  ╰───── expected type declared here as Boolean
                ────╯
            "#]],
        );
    }
}

mod invalid_id_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn float_into_id() {
        expect_errors(
            "
        {
          complicatedArgs {
            idArgField(idArg: 1.0)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type ID, found a float
                    ╭─[ query.graphql:3:23 ]
                    │
                  3 │     idArgField(idArg: 1.0)
                    │                       ─┬─  
                    │                        ╰─── provided value is a float
                    │
                    ├─[ schema.graphql:84:21 ]
                    │
                 84 │   idArgField(idArg: ID): String
                    │                     ─┬  
                    │                      ╰── expected type declared here as ID
                ────╯
            "#]],
        );
    }

    #[test]
    fn boolean_into_id() {
        expect_errors(
            "
        {
          complicatedArgs {
            idArgField(idArg: true)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type ID, found a boolean
                    ╭─[ query.graphql:3:23 ]
                    │
                  3 │     idArgField(idArg: true)
                    │                       ──┬─  
                    │                         ╰─── provided value is a boolean
                    │
                    ├─[ schema.graphql:84:21 ]
                    │
                 84 │   idArgField(idArg: ID): String
                    │                     ─┬  
                    │                      ╰── expected type declared here as ID
                ────╯
            "#]],
        );
    }

    #[test]
    fn unquoted_into_id() {
        expect_errors(
            "
        {
          complicatedArgs {
            idArgField(idArg: SOMETHING)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type ID, found an enum
                    ╭─[ query.graphql:3:23 ]
                    │
                  3 │     idArgField(idArg: SOMETHING)
                    │                       ────┬────  
                    │                           ╰────── provided value is an enum
                    │
                    ├─[ schema.graphql:84:21 ]
                    │
                 84 │   idArgField(idArg: ID): String
                    │                     ─┬  
                    │                      ╰── expected type declared here as ID
                ────╯
            "#]],
        );
    }
}

mod invalid_enum_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn int_into_enum() {
        expect_errors(
            "
        {
          dog {
            doesKnowCommand(dogCommand: 2)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type DogCommand, found an integer
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: 2)
                    │                                 ┬  
                    │                                 ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:27:31 ]
                    │
                 27 │   doesKnowCommand(dogCommand: DogCommand): Boolean
                    │                               ─────┬────  
                    │                                    ╰────── expected type declared here as DogCommand
                ────╯
            "#]],
        );
    }

    #[test]
    fn float_into_enum() {
        expect_errors(
            "
        {
          dog {
            doesKnowCommand(dogCommand: 1.0)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type DogCommand, found a float
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: 1.0)
                    │                                 ─┬─  
                    │                                  ╰─── provided value is a float
                    │
                    ├─[ schema.graphql:27:31 ]
                    │
                 27 │   doesKnowCommand(dogCommand: DogCommand): Boolean
                    │                               ─────┬────  
                    │                                    ╰────── expected type declared here as DogCommand
                ────╯
            "#]],
        );
    }

    #[test]
    fn string_into_enum() {
        expect_errors(
            r#"
        {
          dog {
            doesKnowCommand(dogCommand: "SIT")
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type DogCommand, found a string
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: "SIT")
                    │                                 ──┬──  
                    │                                   ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:27:31 ]
                    │
                 27 │   doesKnowCommand(dogCommand: DogCommand): Boolean
                    │                               ─────┬────  
                    │                                    ╰────── expected type declared here as DogCommand
                ────╯
            "#]],
        );
    }

    #[test]
    fn boolean_into_enum() {
        expect_errors(
            "
        {
          dog {
            doesKnowCommand(dogCommand: true)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type DogCommand, found a boolean
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: true)
                    │                                 ──┬─  
                    │                                   ╰─── provided value is a boolean
                    │
                    ├─[ schema.graphql:27:31 ]
                    │
                 27 │   doesKnowCommand(dogCommand: DogCommand): Boolean
                    │                               ─────┬────  
                    │                                    ╰────── expected type declared here as DogCommand
                ────╯
            "#]],
        );
    }

    #[test]
    fn unknown_enum_value_into_enum() {
        expect_errors(
            "
        {
          dog {
            doesKnowCommand(dogCommand: JUGGLE)
          }
        }
      ",
            expect![[r#"
                Error: value `JUGGLE` does not exist on `DogCommand`
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: JUGGLE)
                    │                                 ───┬──  
                    │                                    ╰──── value does not exist on `DogCommand` enum
                    │
                    ├─[ schema.graphql:16:1 ]
                    │
                 16 │ ╭─▶ enum DogCommand {
                    ┆ ┆   
                 20 │ ├─▶ }
                    │ │       
                    │ ╰─────── enum defined here
                ────╯
            "#]],
        );
    }

    #[test]
    fn different_case_enum_value_into_enum() {
        expect_errors(
            "
        {
          dog {
            doesKnowCommand(dogCommand: sit)
          }
        }
      ",
            expect![[r#"
                Error: value `sit` does not exist on `DogCommand`
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     doesKnowCommand(dogCommand: sit)
                    │                                 ─┬─  
                    │                                  ╰─── value does not exist on `DogCommand` enum
                    │
                    ├─[ schema.graphql:16:1 ]
                    │
                 16 │ ╭─▶ enum DogCommand {
                    ┆ ┆   
                 20 │ ├─▶ }
                    │ │       
                    │ ╰─────── enum defined here
                ────╯
            "#]],
        );
    }
}

mod valid_list_values {
    use super::expect_valid;

    #[test]
    fn good_list_value() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            stringListArgField(stringListArg: ["one", null, "two"])
          }
        }
      "#,
        );
    }

    #[test]
    fn empty_list_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            stringListArgField(stringListArg: [])
          }
        }
      ",
        );
    }

    #[test]
    fn null_value() {
        expect_valid(
            "
        {
          complicatedArgs {
            stringListArgField(stringListArg: null)
          }
        }
      ",
        );
    }

    #[test]
    fn single_value_into_list() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            stringListArgField(stringListArg: "one")
          }
        }
      "#,
        );
    }
}

mod invalid_list_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn incorrect_item_type() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            stringListArgField(stringListArg: ["one", 2])
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type String, found an integer
                    ╭─[ query.graphql:3:47 ]
                    │
                  3 │     stringListArgField(stringListArg: ["one", 2])
                    │                                               ┬  
                    │                                               ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:85:37 ]
                    │
                 85 │   stringListArgField(stringListArg: [String]): String
                    │                                     ────┬───  
                    │                                         ╰───── expected type declared here as String
                ────╯
            "#]],
        );
    }

    #[test]
    fn single_value_of_incorrect_type() {
        expect_errors(
            "
        {
          complicatedArgs {
            stringListArgField(stringListArg: 1)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type [String], found an integer
                    ╭─[ query.graphql:3:39 ]
                    │
                  3 │     stringListArgField(stringListArg: 1)
                    │                                       ┬  
                    │                                       ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:85:37 ]
                    │
                 85 │   stringListArgField(stringListArg: [String]): String
                    │                                     ────┬───  
                    │                                         ╰───── expected type declared here as [String]
                ────╯
            "#]],
        );
    }
}

mod valid_non_nullable_values {
    use super::expect_valid;

    #[test]
    fn arg_on_optional_arg() {
        expect_valid(
            "
        {
          dog {
            isHouseTrained(atOtherHomes: true)
          }
        }
      ",
        );
    }

    #[test]
    fn no_arg_on_optional_arg() {
        expect_valid(
            "
        {
          dog {
            isHouseTrained
          }
        }
      ",
        );
    }

    #[test]
    fn multiple_args() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleReqs(req1: 1, req2: 2)
          }
        }
      ",
        );
    }

    #[test]
    fn multiple_args_reverse_order() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleReqs(req2: 2, req1: 1)
          }
        }
      ",
        );
    }

    #[test]
    fn no_args_on_multiple_optional() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOpts
          }
        }
      ",
        );
    }

    #[test]
    fn one_arg_on_multiple_optional() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOpts(opt1: 1)
          }
        }
      ",
        );
    }

    #[test]
    fn second_arg_on_multiple_optional() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOpts(opt2: 1)
          }
        }
      ",
        );
    }

    #[test]
    fn multiple_required_args_on_mixed_list() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOptAndReq(req1: 3, req2: 4)
          }
        }
      ",
        );
    }

    #[test]
    fn multiple_required_and_one_optional_arg_on_mixed_list() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOptAndReq(req1: 3, req2: 4, opt1: 5)
          }
        }
      ",
        );
    }

    #[test]
    fn all_required_and_optional_args_on_mixed_list() {
        expect_valid(
            "
        {
          complicatedArgs {
            multipleOptAndReq(req1: 3, req2: 4, opt1: 5, opt2: 6)
          }
        }
      ",
        );
    }
}

mod invalid_non_nullable_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn incorrect_value_type() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            multipleReqs(req2: "two", req1: "one")
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Int!, found a string
                    ╭─[ query.graphql:3:24 ]
                    │
                  3 │     multipleReqs(req2: "two", req1: "one")
                    │                        ──┬──  
                    │                          ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:89:34 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                                  ──┬─  
                    │                                    ╰─── expected type declared here as Int!
                ────╯
                Error: expected value of type Int!, found a string
                    ╭─[ query.graphql:3:37 ]
                    │
                  3 │     multipleReqs(req2: "two", req1: "one")
                    │                                     ──┬──  
                    │                                       ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:89:22 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                      ──┬─  
                    │                        ╰─── expected type declared here as Int!
                ────╯
            "#]],
        );
    }

    #[test]
    fn incorrect_value_and_missing_argument() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            multipleReqs(req1: "one")
          }
        }
      "#,
            expect![[r#"
                Error: the required argument `ComplicatedArgs.multipleReqs(req2:)` is not provided
                    ╭─[ query.graphql:3:5 ]
                    │
                  3 │     multipleReqs(req1: "one")
                    │     ────────────┬────────────  
                    │                 ╰────────────── missing value for argument `req2`
                    │
                    ├─[ schema.graphql:89:28 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                            ─────┬────  
                    │                                 ╰────── argument defined here
                ────╯
                Error: expected value of type Int!, found a string
                    ╭─[ query.graphql:3:24 ]
                    │
                  3 │     multipleReqs(req1: "one")
                    │                        ──┬──  
                    │                          ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:89:22 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                      ──┬─  
                    │                        ╰─── expected type declared here as Int!
                ────╯
            "#]],
        );
    }

    #[test]
    fn null_value() {
        expect_errors(
            "
        {
          complicatedArgs {
            multipleReqs(req1: null)
          }
        }
      ",
            expect![[r#"
                Error: the required argument `ComplicatedArgs.multipleReqs(req1:)` is not provided
                    ╭─[ query.graphql:3:5 ]
                    │
                  3 │     multipleReqs(req1: null)
                    │     ────────────┬───────────  
                    │                 ╰───────────── missing value for argument `req1`
                    │
                    ├─[ schema.graphql:89:16 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                ─────┬────  
                    │                     ╰────── argument defined here
                ────╯
                Error: the required argument `ComplicatedArgs.multipleReqs(req2:)` is not provided
                    ╭─[ query.graphql:3:5 ]
                    │
                  3 │     multipleReqs(req1: null)
                    │     ────────────┬───────────  
                    │                 ╰───────────── missing value for argument `req2`
                    │
                    ├─[ schema.graphql:89:28 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                            ─────┬────  
                    │                                 ╰────── argument defined here
                ────╯
                Error: expected value of type Int!, found null
                    ╭─[ query.graphql:3:24 ]
                    │
                  3 │     multipleReqs(req1: null)
                    │                        ──┬─  
                    │                          ╰─── provided value is null
                    │
                    ├─[ schema.graphql:89:22 ]
                    │
                 89 │   multipleReqs(req1: Int!, req2: Int!): String
                    │                      ──┬─  
                    │                        ╰─── expected type declared here as Int!
                ────╯
            "#]],
        );
    }
}

mod valid_input_object_values {
    use super::expect_valid;

    #[test]
    fn optional_arg_required_field() {
        expect_valid(
            "
        {
          complicatedArgs {
            complexArgField
          }
        }
      ",
        );
    }

    #[test]
    fn partial_object_only_required() {
        expect_valid(
            "
        {
          complicatedArgs {
            complexArgField(complexArg: { requiredField: true })
          }
        }
      ",
        );
    }

    #[test]
    fn partial_object_required_boolean_false() {
        expect_valid(
            "
        {
          complicatedArgs {
            complexArgField(complexArg: { requiredField: false })
          }
        }
      ",
        );
    }

    #[test]
    fn partial_object_including_required() {
        expect_valid(
            "
        {
          complicatedArgs {
            complexArgField(complexArg: { requiredField: true, intField: 4 })
          }
        }
      ",
        );
    }

    #[test]
    fn full_object() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            complexArgField(complexArg: {
              requiredField: true,
              intField: 4,
              stringField: "foo",
              booleanField: false,
              stringListField: ["one", "two"]
            })
          }
        }
      "#,
        );
    }

    #[test]
    fn full_object_unordered() {
        expect_valid(
            r#"
        {
          complicatedArgs {
            complexArgField(complexArg: {
              stringListField: ["one", "two"],
              booleanField: false,
              requiredField: true,
              stringField: "foo",
              intField: 4,
            })
          }
        }
      "#,
        );
    }
}

mod invalid_input_object_values {
    use super::expect_errors;
    use expect_test::expect;

    #[test]
    fn partial_object_missing_required() {
        expect_errors(
            "
        {
          complicatedArgs {
            complexArgField(complexArg: { intField: 4 })
          }
        }
      ",
            expect![[r#"
                Error: the required field `ComplexInput.requiredField` is not provided
                    ╭─[ query.graphql:3:33 ]
                    │
                  3 │     complexArgField(complexArg: { intField: 4 })
                    │                                 ───────┬───────  
                    │                                        ╰───────── missing value for field `requiredField`
                    │
                    ├─[ schema.graphql:60:3 ]
                    │
                 60 │   requiredField: Boolean!
                    │   ───────────┬───────────  
                    │              ╰───────────── field defined here
                ────╯
            "#]],
        );
    }

    #[test]
    fn partial_object_invalid_field_type() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            complexArgField(complexArg: {
              stringListField: ["one", 2],
              requiredField: true,
            })
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type String, found an integer
                    ╭─[ query.graphql:4:32 ]
                    │
                  4 │       stringListField: ["one", 2],
                    │                                ┬  
                    │                                ╰── provided value is an integer
                    │
                    ├─[ schema.graphql:65:20 ]
                    │
                 65 │   stringListField: [String]
                    │                    ────┬───  
                    │                        ╰───── expected type declared here as String
                ────╯
            "#]],
        );
    }

    #[test]
    fn partial_object_null_to_non_null_field() {
        expect_errors(
            "
        {
          complicatedArgs {
            complexArgField(complexArg: {
              requiredField: true,
              nonNullField: null,
            })
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Boolean!, found null
                    ╭─[ query.graphql:5:21 ]
                    │
                  5 │       nonNullField: null,
                    │                     ──┬─  
                    │                       ╰─── provided value is null
                    │
                    ├─[ schema.graphql:61:17 ]
                    │
                 61 │   nonNullField: Boolean! = false
                    │                 ────┬───  
                    │                     ╰───── expected type declared here as Boolean!
                ────╯
            "#]],
        );
    }

    #[test]
    fn partial_object_unknown_field() {
        expect_errors(
            r#"
        {
          complicatedArgs {
            complexArgField(complexArg: {
              requiredField: true,
              invalidField: "value"
            })
          }
        }
      "#,
            expect![[r#"
                Error: field `invalidField` does not exist on `ComplexInput`
                    ╭─[ query.graphql:5:21 ]
                    │
                  5 │       invalidField: "value"
                    │                     ───┬───  
                    │                        ╰───── value does not exist on `ComplexInput` input object
                    │
                    ├─[ schema.graphql:59:1 ]
                    │
                 59 │ ╭─▶ input ComplexInput {
                    ┆ ┆   
                 66 │ ├─▶ }
                    │ │       
                    │ ╰─────── input object defined here
                ────╯
            "#]],
        );
    }

    #[test]
    fn custom_scalar_accept_complex_literals() {
        use apollo_compiler::ExecutableDocument;
        use apollo_compiler::Schema;

        let schema = Schema::parse_and_validate(
            "
                scalar Any
                type Query {
                  anyArg(arg: Any): String
                }
            ",
            "schema.graphql",
        )
        .unwrap();

        ExecutableDocument::parse_and_validate(
            &schema,
            r#"
                {
                  test1: anyArg(arg: 123)
                  test2: anyArg(arg: "abc")
                  test3: anyArg(arg: [123, "abc"])
                  test4: anyArg(arg: {deep: [123, "abc"]})
                }
            "#,
            "query.graphql",
        )
        .unwrap();
    }
}

mod directive_arguments {
    use super::expect_errors;
    use super::expect_valid;
    use expect_test::expect;

    #[test]
    fn with_directives_of_valid_types() {
        expect_valid(
            "
        {
          dog @include(if: true) {
            name
          }
          human @skip(if: false) {
            name
          }
        }
      ",
        );
    }

    #[test]
    fn with_directives_of_invalid_types() {
        expect_errors(
            r#"
        {
          dog @include(if: "yes") {
            name @skip(if: ENUM)
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Boolean!, found a string
                     ╭─[ query.graphql:2:20 ]
                     │
                   2 │   dog @include(if: "yes") {
                     │                    ──┬──  
                     │                      ╰──── provided value is a string
                     │
                     ├─[ built_in.graphql:146:7 ]
                     │
                 146 │   if: Boolean!
                     │       ────┬───  
                     │           ╰───── expected type declared here as Boolean!
                ─────╯
                Error: expected value of type Boolean!, found an enum
                     ╭─[ query.graphql:3:20 ]
                     │
                   3 │     name @skip(if: ENUM)
                     │                    ──┬─  
                     │                      ╰─── provided value is an enum
                     │
                     ├─[ built_in.graphql:140:7 ]
                     │
                 140 │   if: Boolean!
                     │       ────┬───  
                     │           ╰───── expected type declared here as Boolean!
                ─────╯
            "#]],
        );
    }
}

mod variable_default_values {
    use super::expect_errors;
    use super::expect_valid;
    use expect_test::expect;

    #[test]
    fn variables_with_valid_default_values() {
        expect_valid(
            r#"
        query WithDefaultValues(
          $a: Int = 1,
          $b: String = "ok",
          $c: ComplexInput = { requiredField: true, intField: 3 }
          $d: Int! = 123
        ) {
          dog { name }
          complicatedArgs {
              intArgField(intArg: $a)
              stringArgField(stringArg: $b)
              complexArgField(complexArg: $c)
              intArgField2: intArgField(intArg: $d)
          }
        }
      "#,
        );
    }

    #[test]
    fn variables_with_valid_default_null_values() {
        expect_valid(
            "
        query WithDefaultValues(
          $a: Int = null,
          $b: String = null,
          $c: ComplexInput = { requiredField: true, intField: null }
        ) {
          dog { name }
          complicatedArgs {
              intArgField(intArg: $a)
              stringArgField(stringArg: $b)
              complexArgField(complexArg: $c)
          }
        }
      ",
        );
    }

    #[test]
    fn variables_with_invalid_default_null_values() {
        expect_errors(
            "
        query WithDefaultValues(
          $a: Int! = null,
          $b: String! = null,
          $c: ComplexInput = { requiredField: null, intField: null }
        ) {
          dog { name }
          complicatedArgs {
              intArgField(intArg: $a)
              stringArgField(stringArg: $b)
              complexArgField(complexArg: $c)
          }
        }
      ",
            expect![[r#"
                Error: expected value of type Int!, found null
                   ╭─[ query.graphql:2:14 ]
                   │
                 2 │   $a: Int! = null,
                   │       ──┬─   ──┬─  
                   │         ╰────────── expected type declared here as Int!
                   │                │   
                   │                ╰─── provided value is null
                ───╯
                Error: expected value of type String!, found null
                   ╭─[ query.graphql:3:17 ]
                   │
                 3 │   $b: String! = null,
                   │       ───┬───   ──┬─  
                   │          ╰──────────── expected type declared here as String!
                   │                   │   
                   │                   ╰─── provided value is null
                ───╯
                Error: the required field `ComplexInput.requiredField` is not provided
                    ╭─[ query.graphql:4:22 ]
                    │
                  4 │   $c: ComplexInput = { requiredField: null, intField: null }
                    │                      ───────────────────┬───────────────────  
                    │                                         ╰───────────────────── missing value for field `requiredField`
                    │
                    ├─[ schema.graphql:60:3 ]
                    │
                 60 │   requiredField: Boolean!
                    │   ───────────┬───────────  
                    │              ╰───────────── field defined here
                ────╯
                Error: expected value of type Boolean!, found null
                    ╭─[ query.graphql:4:39 ]
                    │
                  4 │   $c: ComplexInput = { requiredField: null, intField: null }
                    │                                       ──┬─  
                    │                                         ╰─── provided value is null
                    │
                    ├─[ schema.graphql:60:18 ]
                    │
                 60 │   requiredField: Boolean!
                    │                  ────┬───  
                    │                      ╰───── expected type declared here as Boolean!
                ────╯
            "#]],
        );
    }

    #[test]
    fn variables_with_invalid_default_values() {
        expect_errors(
            r#"
        query InvalidDefaultValues(
          $a: Int = "one",
          $b: String = 4,
          $c: ComplexInput = "NotVeryComplex"
        ) {
          dog { name }
          complicatedArgs {
              intArgField(intArg: $a)
              stringArgField(stringArg: $b)
              complexArgField(complexArg: $c)
          }
        }
      "#,
            expect![[r#"
                Error: expected value of type Int, found a string
                   ╭─[ query.graphql:2:13 ]
                   │
                 2 │   $a: Int = "one",
                   │       ─┬─   ──┬──  
                   │        ╰─────────── expected type declared here as Int
                   │               │    
                   │               ╰──── provided value is a string
                ───╯
                Error: expected value of type String, found an integer
                   ╭─[ query.graphql:3:16 ]
                   │
                 3 │   $b: String = 4,
                   │       ───┬──   ┬  
                   │          ╰──────── expected type declared here as String
                   │                │  
                   │                ╰── provided value is an integer
                ───╯
                Error: expected value of type ComplexInput, found a string
                   ╭─[ query.graphql:4:22 ]
                   │
                 4 │   $c: ComplexInput = "NotVeryComplex"
                   │       ──────┬─────   ────────┬───────  
                   │             ╰────────────────────────── expected type declared here as ComplexInput
                   │                              │         
                   │                              ╰───────── provided value is a string
                ───╯
            "#]],
        );
    }

    #[test]
    fn variables_with_complex_invalid_default_values() {
        expect_errors(
            r#"
        query WithDefaultValues(
          $a: ComplexInput = { requiredField: 123, intField: "abc" }
        ) {
          dog { name }
          complicatedArgs { complexArgField(complexArg: $a) }
        }
      "#,
            expect![[r#"
                Error: expected value of type Boolean!, found an integer
                    ╭─[ query.graphql:2:39 ]
                    │
                  2 │   $a: ComplexInput = { requiredField: 123, intField: "abc" }
                    │                                       ─┬─  
                    │                                        ╰─── provided value is an integer
                    │
                    ├─[ schema.graphql:60:18 ]
                    │
                 60 │   requiredField: Boolean!
                    │                  ────┬───  
                    │                      ╰───── expected type declared here as Boolean!
                ────╯
                Error: expected value of type Int, found a string
                    ╭─[ query.graphql:2:54 ]
                    │
                  2 │   $a: ComplexInput = { requiredField: 123, intField: "abc" }
                    │                                                      ──┬──  
                    │                                                        ╰──── provided value is a string
                    │
                    ├─[ schema.graphql:62:13 ]
                    │
                 62 │   intField: Int
                    │             ─┬─  
                    │              ╰─── expected type declared here as Int
                ────╯
            "#]],
        );
    }

    #[test]
    fn complex_variables_missing_required_field() {
        expect_errors(
            "
        query MissingRequiredField($a: ComplexInput = {intField: 3}) {
          dog { name }
          complicatedArgs { complexArgField(complexArg: $a) }
        }
      ",
            expect![[r#"
                Error: the required field `ComplexInput.requiredField` is not provided
                    ╭─[ query.graphql:1:47 ]
                    │
                  1 │ query MissingRequiredField($a: ComplexInput = {intField: 3}) {
                    │                                               ──────┬──────  
                    │                                                     ╰──────── missing value for field `requiredField`
                    │
                    ├─[ schema.graphql:60:3 ]
                    │
                 60 │   requiredField: Boolean!
                    │   ───────────┬───────────  
                    │              ╰───────────── field defined here
                ────╯
            "#]],
        );
    }

    #[test]
    fn list_variables_with_invalid_item() {
        expect_errors(
            r#"
        query InvalidItem($a: [String] = ["one", 2]) {
          dog { name }
          complicatedArgs { stringListArgField(stringListArg: $a) }
        }
      "#,
            expect![[r#"
                Error: expected value of type String, found an integer
                   ╭─[ query.graphql:1:42 ]
                   │
                 1 │ query InvalidItem($a: [String] = ["one", 2]) {
                   │                       ────┬───           ┬  
                   │                           ╰───────────────── expected type declared here as String
                   │                                          │  
                   │                                          ╰── provided value is an integer
                ───╯
            "#]],
        );
    }
}

#[test]
fn handles_built_in_type_redefinition() {
    let schema = r#"
scalar String

type Query {
  foo: String
}
"#;

    let errors = Schema::parse_and_validate(schema, "schema.graphql")
        .expect_err("invalid schema")
        .errors;
    let expected = expect![[r#"
        Error: built-in scalar definitions must be omitted
           ╭─[ schema.graphql:2:1 ]
           │
         2 │ scalar String
           │ ──────┬──────  
           │       ╰──────── remove this scalar definition
        ───╯
    "#]];
    expected.assert_eq(&errors.to_string());

    let builder = SchemaBuilder::new()
        .allow_builtin_redefinitions();
    let _ = builder.parse(schema, "schema.graphql").build().expect("schema parsed successfully");
}
