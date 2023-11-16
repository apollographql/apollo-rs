//! Ported from graphql-js, 2023-11-16
//! https://github.com/graphql/graphql-js/blob/0b7590f0a2b65e6210da2e49be0d8e6c27781af2/src/validation/__tests__/ValuesOfCorrectTypeRule-test.ts
//!
//! Note all `expect_errors` calls do not check for the kind of errors right now, while in
//! graphql-js they do.
use std::sync::OnceLock;

use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

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

fn test_schema() -> &'static Schema {
    static SCHEMA: OnceLock<Schema> = OnceLock::new();

    SCHEMA.get_or_init(|| {
        let schema = Schema::parse(GRAPHQL_JS_TEST_SCHEMA, "schema.graphql");
        schema.validate().unwrap();
        schema
    })
}

#[track_caller]
fn expect_valid(query: &'static str) {
    let schema = test_schema();

    let executable = ExecutableDocument::parse(schema, query, "query.graphql");
    executable.validate(schema).unwrap();
}

fn expect_errors(query: &'static str) {
    let schema = test_schema();

    let executable = ExecutableDocument::parse(schema, query, "query.graphql");
    let _errors = executable.validate(schema).expect_err("should have errors");
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
        );
    }
}

mod invalid_int_values {
    use super::expect_errors;

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
        );
    }
}

mod invalid_float_values {
    use super::expect_errors;

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
        );
    }
}

mod invalid_boolean_values {
    use super::expect_errors;

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
        );
    }
}

mod invalid_id_values {
    use super::expect_errors;

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
        );
    }
}

mod invalid_enum_values {
    use super::expect_errors;

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
        );
    }

    #[test]
    fn custom_scalar_accept_complex_literals() {
        use apollo_compiler::ExecutableDocument;
        use apollo_compiler::Schema;

        let schema = Schema::parse(
            "
            scalar Any
            type Query {
              anyArg(arg: Any): String
            }
        ",
            "schema.graphql",
        );
        schema.validate().unwrap();

        let query = ExecutableDocument::parse(
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
        );

        query.validate(&schema).unwrap();
    }
}

mod directive_arguments {
    use super::expect_errors;
    use super::expect_valid;

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
        );
    }
}

mod variable_default_values {
    use super::expect_errors;
    use super::expect_valid;

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
        );
    }
}
