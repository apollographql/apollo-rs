use apollo_compiler::ast;
use apollo_compiler::ty;
use expect_test::expect;

#[test]
fn test_serde_value() {
    let input = r#"
        query {
            field(arg: {
                null_field: null,
                enum_field: EXAMPLE,
                var_field: $var,
                string_field: "example"
                float_field: 1.5
                int_field: 47
                list_field: [1, 2, 3]
            })
        }
    "#;
    let value = ast::Document::parse(input, "input.graphql")
        .unwrap()
        .definitions[0]
        .as_operation_definition()
        .unwrap()
        .selection_set[0]
        .as_field()
        .unwrap()
        .arguments[0]
        .value
        .clone();
    let graphql = value.to_string();
    let expected_graphql = expect![[r#"
        {
          null_field: null,
          enum_field: EXAMPLE,
          var_field: $var,
          string_field: "example",
          float_field: 1.5,
          int_field: 47,
          list_field: [
            1,
            2,
            3,
          ],
        }"#]];
    expected_graphql.assert_eq(&graphql);
    let json = serde_json::to_string_pretty(&value).unwrap();
    let expected_json = expect![[r#"
        {
          "Object": [
            [
              "null_field",
              "Null"
            ],
            [
              "enum_field",
              {
                "Enum": "EXAMPLE"
              }
            ],
            [
              "var_field",
              {
                "Variable": "var"
              }
            ],
            [
              "string_field",
              {
                "String": "example"
              }
            ],
            [
              "float_field",
              {
                "Float": "1.5"
              }
            ],
            [
              "int_field",
              {
                "Int": "47"
              }
            ],
            [
              "list_field",
              {
                "List": [
                  {
                    "Int": "1"
                  },
                  {
                    "Int": "2"
                  },
                  {
                    "Int": "3"
                  }
                ]
              }
            ]
          ]
        }"#]];
    expected_json.assert_eq(&json);
    let enum_value = value.as_object().unwrap()[1].1.as_enum().unwrap();
    assert!(enum_value.location().is_some());

    let value_deserialized: ast::Value = serde_json::from_str(&json).unwrap();
    let expected_debug = expect![[r#"
        Object(
            [
                (
                    "null_field",
                    Null,
                ),
                (
                    "enum_field",
                    Enum(
                        "EXAMPLE",
                    ),
                ),
                (
                    "var_field",
                    Variable(
                        "var",
                    ),
                ),
                (
                    "string_field",
                    String(
                        "example",
                    ),
                ),
                (
                    "float_field",
                    Float(
                        1.5,
                    ),
                ),
                (
                    "int_field",
                    Int(
                        47,
                    ),
                ),
                (
                    "list_field",
                    List(
                        [
                            Int(
                                1,
                            ),
                            Int(
                                2,
                            ),
                            Int(
                                3,
                            ),
                        ],
                    ),
                ),
            ],
        )
    "#]];
    expected_debug.assert_debug_eq(&value_deserialized);
    assert_eq!(*value, value_deserialized);
    assert_eq!(graphql, value_deserialized.to_string());
    let enum_value = value_deserialized.as_object().unwrap()[1]
        .1
        .as_enum()
        .unwrap();
    // Locations are not preserved through serialization
    assert!(enum_value.location().is_none());
}

#[test]
fn test_serde_type() {
    let ty_1 = ty!(a);
    let ty_2 = ty!([[a!]]!);
    expect!["a"].assert_eq(&ty_1.to_string());
    expect!["[[a!]]!"].assert_eq(&ty_2.to_string());
    expect![[r#"
        Named(
            "a",
        )
    "#]]
    .assert_debug_eq(&ty_1);
    expect![[r#"
        NonNullList(
            List(
                NonNullNamed(
                    "a",
                ),
            ),
        )
    "#]]
    .assert_debug_eq(&ty_2);
    let json_1 = serde_json::to_string(&ty_1).unwrap();
    let json_2 = serde_json::to_string(&ty_2).unwrap();
    expect![[r#"{"Named":"a"}"#]].assert_eq(&json_1);
    expect![[r#"{"NonNullList":{"List":{"NonNullNamed":"a"}}}"#]].assert_eq(&json_2);
    let ty_1_deserialized: ast::Type = serde_json::from_str(&json_1).unwrap();
    let ty_2_deserialized: ast::Type = serde_json::from_str(&json_2).unwrap();
    assert_eq!(ty_1, ty_1_deserialized);
    assert_eq!(ty_2, ty_2_deserialized);
}

#[test]
fn test_serde_deserialization_errors() {
    #[track_caller]
    fn assert_err<T: serde::de::DeserializeOwned>(input: &str, expected: expect_test::Expect) {
        expected.assert_eq(&serde_json::from_str::<T>(input).err().unwrap().to_string());
    }
    assert_err::<ast::Name>(
        r#""1nvalid""#,
        expect![[
            r#"invalid value: string "1nvalid", expected a string in GraphQL Name syntax at line 1 column 9"#
        ]],
    );
    assert_err::<ast::IntValue>(
        r#""+3""#,
        expect![[
            r#"invalid value: string "+3", expected a string in GraphQL IntValue syntax at line 1 column 4"#
        ]],
    );
    assert_err::<ast::FloatValue>(
        r#""+3.5""#,
        expect![[
            r#"invalid value: string "+3.5", expected a string in GraphQL FloatValue syntax at line 1 column 6"#
        ]],
    );
}
