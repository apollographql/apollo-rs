//! Ported from graphql-js, 2024-02-01
//! https://github.com/graphql/graphql-js/blob/9c90a23dd430ba7b9db3d566b084e9f66aded346/src/validation/__tests__/OverlappingFieldsCanBeMergedRule-test.ts
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use expect_test::expect;
use expect_test::Expect;
use std::sync::OnceLock;
use unindent::unindent;

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

#[test]
fn unique_fields() {
    expect_valid(
        r#"
      fragment uniqueFields on Dog {
        name
        nickname
      }

      { dog { ...uniqueFields } }
    "#,
    );
}

#[test]
fn identical_fields() {
    expect_valid(
        r#"
      fragment mergeIdenticalFields on Dog {
        name
        name
      }

      { dog { ...mergeIdenticalFields } }
    "#,
    );
}

#[test]
fn identical_fields_with_identical_args() {
    expect_valid(
        r#"
      fragment mergeIdenticalFieldsWithIdenticalArgs on Dog {
        doesKnowCommand(dogCommand: SIT)
        doesKnowCommand(dogCommand: SIT)
      }

      { dog { ...mergeIdenticalFieldsWithIdenticalArgs } }
    "#,
    );
}

#[test]
fn identical_fields_with_identical_directives() {
    expect_valid(
        r#"
      fragment mergeSameFieldsWithSameDirectives on Dog {
        name @include(if: true)
        name @include(if: true)
      }

      { dog { ...mergeSameFieldsWithSameDirectives } }
    "#,
    );
}

#[test]
fn different_args_with_different_aliases() {
    expect_valid(
        r#"
      fragment differentArgsWithDifferentAliases on Dog {
        knowsSit: doesKnowCommand(dogCommand: SIT)
        knowsDown: doesKnowCommand(dogCommand: DOWN)
      }

      { dog { ...differentArgsWithDifferentAliases } }
    "#,
    );
}

#[test]
fn different_directives_with_different_aliases() {
    expect_valid(
        r#"
      fragment differentDirectivesWithDifferentAliases on Dog {
        nameIfTrue: name @include(if: true)
        nameIfFalse: name @include(if: false)
      }

      { dog { ...differentDirectivesWithDifferentAliases } }
    "#,
    );
}

#[test]
fn different_skip_include_directives() {
    expect_valid(
        r#"
      fragment differentDirectivesWithDifferentAliases on Dog {
        name @include(if: true)
        name @include(if: false)
      }

      { dog { ...differentDirectivesWithDifferentAliases } }
    "#,
    );
}

/* @stream tests snipped here -- not supported in apollo-rs */

#[test]
fn same_aliases_with_different_field_targets() {
    expect_errors(
        r#"
      fragment sameAliasesWithDifferentFieldTargets on Dog {
        fido: name
        fido: nickname
      }

      { dog { ...sameAliasesWithDifferentFieldTargets } }
    "#,
        expect![[r#"
            Error: operation must not select different fields to the same alias `fido`
               ╭─[query.graphql:3:3]
               │
             2 │   fido: name
               │   ─────┬────  
               │        ╰────── field `fido` is selected from field `name` here
             3 │   fido: nickname
               │   ───────┬──────  
               │          ╰──────── but the same field `fido` is also selected from field `nickname` here
            ───╯
        "#]],
    );
}

#[test]
fn same_aliases_on_non_overlapping_fields() {
    expect_valid(
        r#"
      fragment sameAliasesWithDifferentFieldTargets on Pet {
        ... on Dog {
          name
        }
        ... on Cat {
          name: nickname
        }
      }

      { pet { ...sameAliasesWithDifferentFieldTargets } }
    "#,
    );
}

#[test]
fn alias_masking_direct_field_access() {
    expect_errors(
        r#"
      fragment aliasMaskingDirectFieldAccess on Dog {
        name: nickname
        name
      }

      { dog { ...aliasMaskingDirectFieldAccess } }
    "#,
        expect![[r#"
            Error: operation must not select different fields to the same alias `name`
               ╭─[query.graphql:3:3]
               │
             2 │   name: nickname
               │   ───────┬──────  
               │          ╰──────── field `name` is selected from field `nickname` here
             3 │   name
               │   ──┬─  
               │     ╰─── but the same field `name` is also selected from field `name` here
            ───╯
        "#]],
    );
}

#[test]
fn different_args_second_adds_argument() {
    expect_errors(
        r#"
      fragment conflictingArgs on Dog {
        doesKnowCommand
        doesKnowCommand(dogCommand: HEEL)
      }

      { dog { ...conflictingArgs } }
    "#,
        expect![[r#"
            Error: operation must not provide conflicting field arguments for the same field name `doesKnowCommand`
               ╭─[query.graphql:3:3]
               │
             2 │   doesKnowCommand
               │   ───────┬───────  
               │          ╰───────── but argument `dogCommand` is not provided here
             3 │   doesKnowCommand(dogCommand: HEEL)
               │   ────────────────┬────────────────  
               │                   ╰────────────────── field `doesKnowCommand` is selected with argument `dogCommand` here
               │ 
               │ Help: Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.
            ───╯
        "#]],
    );
}

#[test]
fn different_args_second_missess_argument() {
    expect_errors(
        r#"
      fragment conflictingArgs on Dog {
        doesKnowCommand(dogCommand: SIT)
        doesKnowCommand
      }

      { dog { ...conflictingArgs } }
    "#,
        expect![[r#"
            Error: operation must not provide conflicting field arguments for the same field name `doesKnowCommand`
               ╭─[query.graphql:3:3]
               │
             2 │   doesKnowCommand(dogCommand: SIT)
               │   ────────────────┬───────────────  
               │                   ╰───────────────── field `doesKnowCommand` is selected with argument `dogCommand` here
             3 │   doesKnowCommand
               │   ───────┬───────  
               │          ╰───────── but argument `dogCommand` is not provided here
               │ 
               │ Help: Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.
            ───╯
        "#]],
    );
}

#[test]
fn conflicting_arg_values() {
    expect_errors(
        r#"
      fragment conflictingArgs on Dog {
        doesKnowCommand(dogCommand: SIT)
        doesKnowCommand(dogCommand: HEEL)
      }

      { dog { ...conflictingArgs } }
    "#,
        expect![[r#"
            Error: operation must not provide conflicting field arguments for the same field name `doesKnowCommand`
               ╭─[query.graphql:3:3]
               │
             2 │   doesKnowCommand(dogCommand: SIT)
               │   ────────────────┬───────────────  
               │                   ╰───────────────── field `doesKnowCommand` provides one argument value here
             3 │   doesKnowCommand(dogCommand: HEEL)
               │   ────────────────┬────────────────  
               │                   ╰────────────────── but a different value here
               │ 
               │ Help: Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.
            ───╯
        "#]],
    );
}

#[test]
fn conflicting_arg_names() {
    expect_errors(
        r#"
      fragment conflictingArgs on Dog {
        isAtLocation(x: 0)
        isAtLocation(y: 0)
      }

      { dog { ...conflictingArgs } }
    "#,
        expect![[r#"
            Error: operation must not provide conflicting field arguments for the same field name `isAtLocation`
               ╭─[query.graphql:3:3]
               │
             2 │   isAtLocation(x: 0)
               │   ─────────┬────────  
               │            ╰────────── field `isAtLocation` is selected with argument `x` here
             3 │   isAtLocation(y: 0)
               │   ─────────┬────────  
               │            ╰────────── but argument `x` is not provided here
               │ 
               │ Help: Fields with the same response name must provide the same set of arguments. Consider adding an alias if you need to select fields with different arguments.
            ───╯
        "#]],
    );
}

#[test]
fn different_non_conflicting_args() {
    expect_valid(
        r#"
      fragment conflictingArgs on Pet {
        ... on Dog {
          name(surname: true)
        }
        ... on Cat {
          name
        }
      }

      { pet { ...conflictingArgs } }
    "#,
    );
}

#[test]
fn different_order_args() {
    expect_valid(
        r#"
      {
        dog {
          isAtLocation(x: 0, y: 1)
          isAtLocation(y: 1, x: 0)
        }
      }
    "#,
    );
}

#[test]
fn different_order_input_args() {
    expect_valid(
        r#"
      {
        complicatedArgs {
          complexArgField(complexArg: { intField: 1, requiredField: true })
          complexArgField(complexArg: { requiredField: true, intField: 1 })
        }
      }
    "#,
    );
}

#[test]
fn conflicts_in_fragments() {
    expect_errors(
        r#"
      {
        ...A
        ...B
      }
      fragment A on Type {
        x: a
      }
      fragment B on Type {
        x: b
      }
    "#,
        expect![[r#"
            Error: cannot find fragment `A` in this document
               ╭─[query.graphql:2:3]
               │
             2 │   ...A
               │   ──┬─  
               │     ╰─── fragment `A` is not defined
            ───╯
            Error: cannot find fragment `B` in this document
               ╭─[query.graphql:3:3]
               │
             3 │   ...B
               │   ──┬─  
               │     ╰─── fragment `B` is not defined
            ───╯
            Error: type condition `Type` of fragment `A` is not a type defined in the schema
               ╭─[query.graphql:5:15]
               │
             5 │ fragment A on Type {
               │               ──┬─  
               │                 ╰─── type condition here
            ───╯
            Error: type condition `Type` of fragment `B` is not a type defined in the schema
               ╭─[query.graphql:8:15]
               │
             8 │ fragment B on Type {
               │               ──┬─  
               │                 ╰─── type condition here
            ───╯
        "#]],
    );
}

#[test]
fn dedupe_conflicts() {
    expect_errors(
        r#"
      {
        f1 {
          ...A
          ...B
        }
        f2 {
          ...B
          ...A
        }
        f3 {
          ...A
          ...B
          x: c
        }
      }
      fragment A on Type {
        x: a
      }
      fragment B on Type {
        x: b
      }
    "#,
        expect![[r#"
            Error: type `QueryRoot` does not have a field `f1`
                ╭─[query.graphql:2:3]
                │
              2 │   f1 {
                │   ─┬  
                │    ╰── field `f1` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → f1`
            ────╯
            Error: type `QueryRoot` does not have a field `f2`
                ╭─[query.graphql:6:3]
                │
              6 │   f2 {
                │   ─┬  
                │    ╰── field `f2` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → f2`
            ────╯
            Error: type `QueryRoot` does not have a field `f3`
                ╭─[query.graphql:10:3]
                │
             10 │   f3 {
                │   ─┬  
                │    ╰── field `f3` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → f3`
            ────╯
            Error: type condition `Type` of fragment `A` is not a type defined in the schema
                ╭─[query.graphql:16:15]
                │
             16 │ fragment A on Type {
                │               ──┬─  
                │                 ╰─── type condition here
            ────╯
            Error: type condition `Type` of fragment `B` is not a type defined in the schema
                ╭─[query.graphql:19:15]
                │
             19 │ fragment B on Type {
                │               ──┬─  
                │                 ╰─── type condition here
            ────╯
        "#]],
    );
}

#[test]
fn deep_conflict() {
    expect_errors(
        r#"
      {
        field {
          x: a
        },
        field {
          x: b
        }
      }
    "#,
        expect![[r#"
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:2:3]
                │
              2 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:5:3]
                │
              5 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
        "#]],
    );
}

#[test]
fn deep_conflict_multiple_issues() {
    expect_errors(
        r#"
      {
        field {
          x: a
          y: c
        },
        field {
          x: b
          y: d
        }
      }
    "#,
        expect![[r#"
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:2:3]
                │
              2 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:6:3]
                │
              6 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
        "#]],
    );
}

#[test]
fn very_deep_conflict() {
    expect_errors(
        r#"
      {
        field {
          deepField {
            x: a
          }
        },
        field {
          deepField {
            x: b
          }
        }
      }
    "#,
        expect![[r#"
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:2:3]
                │
              2 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:7:3]
                │
              7 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
        "#]],
    );
}

#[test]
fn deep_conflict_in_fragments() {
    expect_errors(
        r#"
      {
        field {
          ...F
        }
        field {
          ...I
        }
      }
      fragment F on T {
        x: a
        ...G
      }
      fragment G on T {
        y: c
      }
      fragment I on T {
        y: d
        ...J
      }
      fragment J on T {
        x: b
      }
    "#,
        expect![[r#"
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:2:3]
                │
              2 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
            Error: type `QueryRoot` does not have a field `field`
                ╭─[query.graphql:5:3]
                │
              5 │   field {
                │   ──┬──  
                │     ╰──── field `field` selected here
                │
                ├─[schema.graphql:95:6]
                │
             95 │ type QueryRoot {
                │      ────┬────  
                │          ╰────── type `QueryRoot` defined here
                │ 
                │ Note: path to the field: `query → field`
            ────╯
            Error: type condition `T` of fragment `F` is not a type defined in the schema
               ╭─[query.graphql:9:15]
               │
             9 │ fragment F on T {
               │               ┬  
               │               ╰── type condition here
            ───╯
            Error: type condition `T` of fragment `G` is not a type defined in the schema
                ╭─[query.graphql:13:15]
                │
             13 │ fragment G on T {
                │               ┬  
                │               ╰── type condition here
            ────╯
            Error: type condition `T` of fragment `I` is not a type defined in the schema
                ╭─[query.graphql:16:15]
                │
             16 │ fragment I on T {
                │               ┬  
                │               ╰── type condition here
            ────╯
            Error: type condition `T` of fragment `J` is not a type defined in the schema
                ╭─[query.graphql:20:15]
                │
             20 │ fragment J on T {
                │               ┬  
                │               ╰── type condition here
            ────╯
        "#]],
    );
}

mod return_types {
    use apollo_compiler::validation::Valid;
    use apollo_compiler::ExecutableDocument;
    use apollo_compiler::Schema;
    use expect_test::expect;
    use expect_test::Expect;
    use std::sync::OnceLock;
    use unindent::unindent;

    const RETURN_TYPES_TEST_SCHEMA: &str = r#"
      interface SomeBox {
        deepBox: SomeBox
        unrelatedField: String
      }

      type StringBox implements SomeBox {
        scalar: String
        deepBox: StringBox
        unrelatedField: String
        listStringBox: [StringBox]
        stringBox: StringBox
        intBox: IntBox
      }

      type IntBox implements SomeBox {
        scalar: Int
        deepBox: IntBox
        unrelatedField: String
        listStringBox: [StringBox]
        stringBox: StringBox
        intBox: IntBox
      }

      interface NonNullStringBox1 {
        scalar: String!
      }

      type NonNullStringBox1Impl implements SomeBox & NonNullStringBox1 {
        scalar: String!
        unrelatedField: String
        deepBox: SomeBox
      }

      interface NonNullStringBox2 {
        scalar: String!
      }

      type NonNullStringBox2Impl implements SomeBox & NonNullStringBox2 {
        scalar: String!
        unrelatedField: String
        deepBox: SomeBox
      }

      type Connection {
        edges: [Edge]
      }

      type Edge {
        node: Node
      }

      type Node {
        id: ID
        name: String
      }

      type Query {
        someBox: SomeBox
        connection: Connection
      }
    "#;

    fn test_schema() -> &'static Valid<Schema> {
        static SCHEMA: OnceLock<Valid<Schema>> = OnceLock::new();

        SCHEMA.get_or_init(|| {
            Schema::parse_and_validate(unindent(RETURN_TYPES_TEST_SCHEMA), "schema.graphql")
                .unwrap()
        })
    }

    #[track_caller]
    fn expect_valid(query: &'static str) {
        let schema = test_schema();

        ExecutableDocument::parse_and_validate(schema, unindent(query), "query.graphql").unwrap();
    }

    fn expect_errors(query: &'static str, expect: Expect) {
        let schema = test_schema();

        let errors =
            ExecutableDocument::parse_and_validate(schema, unindent(query), "query.graphql")
                .expect_err("should have errors")
                .errors;
        expect.assert_eq(&errors.to_string());
    }

    #[test]
    fn conflicting_return_types_with_potential_overlap() {
        expect_errors(
            r#"
          {
            someBox {
              ...on IntBox {
                scalar
              }
              ...on NonNullStringBox1 {
                scalar
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `scalar`
                   ╭─[query.graphql:7:7]
                   │
                 4 │       scalar
                   │       ───┬──  
                   │          ╰──── `scalar` has type `Int` here
                   │ 
                 7 │       scalar
                   │       ───┬──  
                   │          ╰──── but the same field name has type `String!` here
                ───╯
            "#]],
        );
    }

    #[test]
    fn compatible_return_shapes_on_different_return_types() {
        expect_valid(
            r#"
          {
            someBox {
              ... on SomeBox {
                deepBox {
                  unrelatedField
                }
              }
              ... on StringBox {
                deepBox {
                  unrelatedField
                }
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn no_differing_return_types_despite_no_overlap() {
        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                scalar
              }
              ... on StringBox {
                scalar
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `scalar`
                   ╭─[query.graphql:7:7]
                   │
                 4 │       scalar
                   │       ───┬──  
                   │          ╰──── `scalar` has type `Int` here
                   │ 
                 7 │       scalar
                   │       ───┬──  
                   │          ╰──── but the same field name has type `String` here
                ───╯
            "#]],
        );
    }

    #[test]
    fn non_exclusive_follows_exclusive() {
        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                deepBox {
                  ...X
                }
              }
            }
            someBox {
              ... on StringBox {
                deepBox {
                  ...Y
                }
              }
            }
            memoed: someBox {
              ... on IntBox {
                deepBox {
                  ...X
                }
              }
            }
            memoed: someBox {
              ... on StringBox {
                deepBox {
                  ...Y
                }
              }
            }
            other: someBox {
              ...X
            }
            other: someBox {
              ...Y
            }
          }
          fragment X on SomeBox {
            scalar
          }
          fragment Y on SomeBox {
            scalar: unrelatedField
          }
        "#,
            expect![[r#"
                Error: type `SomeBox` does not have a field `scalar`
                    ╭─[query.graphql:38:3]
                    │
                 38 │   scalar
                    │   ───┬──  
                    │      ╰──── field `scalar` selected here
                    │
                    ├─[schema.graphql:1:11]
                    │
                  1 │ interface SomeBox {
                    │           ───┬───  
                    │              ╰───── type `SomeBox` defined here
                    │ 
                    │ Note: path to the field: `fragment X → scalar`
                ────╯
            "#]],
        );
    }

    #[test]
    fn no_differing_nullability_despite_no_overlap() {
        expect_errors(
            r#"
          {
            someBox {
              ... on NonNullStringBox1 {
                scalar
              }
              ... on StringBox {
                scalar
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `scalar`
                   ╭─[query.graphql:7:7]
                   │
                 4 │       scalar
                   │       ───┬──  
                   │          ╰──── `scalar` has type `String!` here
                   │ 
                 7 │       scalar
                   │       ───┬──  
                   │          ╰──── but the same field name has type `String` here
                ───╯
            "#]],
        );
    }

    #[test]
    fn no_differing_list_despite_no_overlap() {
        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                box: listStringBox {
                  scalar
                }
              }
              ... on StringBox {
                box: stringBox {
                  scalar
                }
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `box`
                    ╭─[query.graphql:9:7]
                    │
                  4 │ ╭───▶       box: listStringBox {
                    ┆ ┆     
                  6 │ ├───▶       }
                    │ │               
                    │ ╰─────────────── `box` has type `[StringBox]` here
                    │ 
                  9 │   ╭─▶       box: stringBox {
                    ┆   ┆   
                 11 │   ├─▶       }
                    │   │             
                    │   ╰───────────── but the same field name has type `StringBox` here
                ────╯
            "#]],
        );

        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                box: stringBox {
                  scalar
                }
              }
              ... on StringBox {
                box: listStringBox {
                  scalar
                }
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `box`
                    ╭─[query.graphql:9:7]
                    │
                  4 │   ╭─▶       box: stringBox {
                    ┆   ┆   
                  6 │   ├─▶       }
                    │   │             
                    │   ╰───────────── `box` has type `StringBox` here
                    │ 
                  9 │ ╭───▶       box: listStringBox {
                    ┆ ┆     
                 11 │ ├───▶       }
                    │ │               
                    │ ╰─────────────── but the same field name has type `[StringBox]` here
                ────╯
            "#]],
        );
    }

    #[test]
    fn differing_sub_fields() {
        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                box: stringBox {
                  val: scalar
                  val: unrelatedField
                }
              }
              ... on StringBox {
                box: stringBox {
                  val: scalar
                }
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different fields to the same alias `val`
                   ╭─[query.graphql:6:9]
                   │
                 5 │         val: scalar
                   │         ─────┬─────  
                   │              ╰─────── field `val` is selected from field `scalar` here
                 6 │         val: unrelatedField
                   │         ─────────┬─────────  
                   │                  ╰─────────── but the same field `val` is also selected from field `unrelatedField` here
                ───╯
            "#]],
        );
    }

    #[test]
    fn differing_deep_return_types() {
        expect_errors(
            r#"
          {
            someBox {
              ... on IntBox {
                box: stringBox {
                  scalar
                }
              }
              ... on StringBox {
                box: intBox {
                  scalar
                }
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `scalar`
                    ╭─[query.graphql:10:9]
                    │
                  5 │         scalar
                    │         ───┬──  
                    │            ╰──── `scalar` has type `String` here
                    │ 
                 10 │         scalar
                    │         ───┬──  
                    │            ╰──── but the same field name has type `Int` here
                ────╯
            "#]],
        );
    }

    #[test]
    fn non_conflicting_overlapping_types() {
        expect_valid(
            r#"
          {
            someBox {
              ... on IntBox {
                scalar: unrelatedField
              }
              ... on StringBox {
                scalar
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn same_scalars() {
        expect_valid(
            r#"
          {
            someBox {
              ...on NonNullStringBox1 {
                scalar
              }
              ...on NonNullStringBox2 {
                scalar
              }
            }
          }
        "#,
        );
    }

    #[test]
    fn deep_types_including_list() {
        expect_errors(
            r#"
          {
            connection {
              ...edgeID
              edges {
                node {
                  id: name
                }
              }
            }
          }

          fragment edgeID on Connection {
            edges {
              node {
                id
              }
            }
          }
        "#,
            expect![[r#"
                Error: operation must not select different types using the same field name `id`
                    ╭─[query.graphql:15:7]
                    │
                  6 │         id: name
                    │         ────┬───  
                    │             ╰───── `id` has type `String` here
                    │ 
                 15 │       id
                    │       ─┬  
                    │        ╰── but the same field name has type `ID` here
                ────╯
                Error: operation must not select different fields to the same alias `id`
                    ╭─[query.graphql:15:7]
                    │
                  6 │         id: name
                    │         ────┬───  
                    │             ╰───── field `id` is selected from field `name` here
                    │ 
                 15 │       id
                    │       ─┬  
                    │        ╰── but the same field `id` is also selected from field `id` here
                ────╯
            "#]],
        );
    }

    #[test]
    fn unknown_types() {
        // The important part is that it doesn't emit a field merging error.
        expect_errors(
            r#"
            someBox {
              ...on UnknownType {
                scalar
              }
              ...on NonNullStringBox2 {
                scalar
              }
            }
          }
        "#,
            expect![[r#"
                Error: syntax error: expected definition
                   ╭─[query.graphql:1:3]
                   │
                 1 │   someBox {
                   │   ───┬───  
                   │      ╰───── expected definition
                ───╯
                Error: type condition `UnknownType` of inline fragment is not a type defined in the schema
                   ╭─[query.graphql:2:11]
                   │
                 2 │     ...on UnknownType {
                   │           ─────┬─────  
                   │                ╰─────── type condition here
                   │ 
                   │ Note: path to the inline fragment: `query → ...`
                ───╯
                Error: inline fragment with type condition `NonNullStringBox2` cannot be applied to `Query`
                    ╭─[query.graphql:5:5]
                    │
                  5 │ ╭─▶     ...on NonNullStringBox2 {
                    ┆ ┆   
                  7 │ ├─▶     }
                    │ │           
                    │ ╰─────────── inline fragment cannot be applied
                    │
                    ├─[schema.graphql:57:1]
                    │
                 57 │ ╭─▶ type Query {
                    ┆ ┆   
                 60 │ ├─▶ }
                    │ │       
                    │ ╰─────── type condition `NonNullStringBox2` is not assignable to this type
                ────╯
                Error: syntax error: expected a StringValue, Name or OperationDefinition
                   ╭─[query.graphql:9:1]
                   │
                 9 │ }
                   │ ┬  
                   │ ╰── expected a StringValue, Name or OperationDefinition
                ───╯
            "#]],
        );
    }
}
