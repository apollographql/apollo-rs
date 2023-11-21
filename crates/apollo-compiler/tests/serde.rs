use apollo_compiler::ast;
use expect_test::expect;

#[test]
fn test_serde_serialization() {
    let input = r#"
        query($var: I = {
            null_field: null,
            enum_field: EXAMPLE,
            var_field: $var,
            string_field: "example"
            float_field: 1.5
            int_field: 47
            list_field: [1, 2, 3]
        }) {
            selection
        }
    "#;
    let value = ast::Document::parse(input, "input.graphql")
        .unwrap()
        .definitions[0]
        .as_operation_definition()
        .unwrap()
        .variables[0]
        .default_value
        .clone()
        .unwrap();
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
fn test_serde_deserialization_errors() {
    #[track_caller]
    fn assert_err<T: serde::de::DeserializeOwned>(input: &str, expected: expect_test::Expect) {
        expected.assert_eq(&serde_json::from_str::<T>(input).err().unwrap().to_string());
    }
    assert_err::<ast::Name>(
        r#""1nvalid""#,
        expect![[r#"invalid value: string "1nvalid", expected a string a GraphQL Name syntax at line 1 column 9"#]],
    );
    assert_err::<ast::IntValue>(
        r#""+3""#,
        expect![[r#"invalid value: string "+3", expected a string a GraphQL IntValue syntax at line 1 column 4"#]],
    );
    assert_err::<ast::FloatValue>(
        r#""+3.5""#,
        expect![[r#"invalid value: string "+3.5", expected a string a GraphQL FloatValue syntax at line 1 column 6"#]],
    );
}
