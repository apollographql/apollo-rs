//! Block string syntax (`"""…"""`) is recorded in `ast::StringValue` and honoured
//! when serializing. Expected outputs were produced with graphql-js 17.0.2
//! (`print(parse(input))` and `printBlockString(value)`).

use apollo_compiler::ast;
use apollo_compiler::ast::StringValue;
use apollo_compiler::ast::Value;
use apollo_compiler::Node;
use expect_test::expect;
use expect_test::Expect;

fn first_argument_value(doc: &ast::Document) -> &Node<Value> {
    let selection = match &doc.definitions[0] {
        ast::Definition::OperationDefinition(op) => &op.selection_set[0],
        ast::Definition::FragmentDefinition(frag) => &frag.selection_set[0],
        _ => panic!("expected an executable definition"),
    };
    &selection.as_field().unwrap().arguments[0].value
}

/// Parse, serialize, compare with graphql-js output, re-parse,
/// and check that the value and its block flag survived.
fn round_trip(input: &str, expected: Expect) -> ast::Document {
    let doc = ast::Document::parse(input, "input.graphql").unwrap();
    let serialized = doc.to_string();
    expected.assert_eq(&serialized);
    let reparsed = ast::Document::parse(&serialized, "serialized.graphql").unwrap();
    assert_eq!(doc, reparsed);
    reparsed
}

#[test]
fn parsing_records_block_syntax() {
    let doc = ast::Document::parse(
        r#"query { field(block: """hello""", quoted: "hello") }"#,
        "input.graphql",
    )
    .unwrap();
    let field = doc.definitions[0]
        .as_operation_definition()
        .unwrap()
        .selection_set[0]
        .as_field()
        .unwrap();
    let Value::String(block) = &*field.arguments[0].value else {
        panic!()
    };
    let Value::String(quoted) = &*field.arguments[1].value else {
        panic!()
    };
    assert!(block.is_block());
    assert!(!quoted.is_block());
    assert_eq!(block.as_str(), "hello");
    assert_eq!(quoted.as_str(), "hello");
    // The flag is syntax metadata and does not affect equality or hashing
    assert_eq!(block, quoted);
    assert_eq!(field.arguments[0].value, field.arguments[1].value);
    assert_eq!(*block, "hello");
    assert_eq!(*block, *"hello");
    assert_eq!(*block, String::from("hello"));
    assert_eq!("hello", *block);
    // Deref to str
    assert_eq!(block.len(), 5);
    assert_eq!(block.to_uppercase(), "HELLO");
    assert_eq!(block.to_string(), "hello");
}

#[test]
fn constructors_and_conversions() {
    assert!(!StringValue::new("a").is_block());
    assert!(StringValue::new_block("a").is_block());
    assert!(!StringValue::from("a").is_block());
    assert!(!StringValue::from(String::from("a")).is_block());
    assert!(!StringValue::from(&String::from("a")).is_block());
    let mut value = StringValue::new("a");
    value.set_block(true);
    assert!(value.is_block());
    assert_eq!(value.clone().into_string(), "a");
    assert_eq!(String::from(value), "a");
    assert_eq!(Value::from("a"), Value::String(StringValue::new("a")));
    assert_eq!(Value::from("a"), Value::String(StringValue::new_block("a")));
    assert_eq!(Value::from(StringValue::new_block("a")).as_str(), Some("a"));
    assert_eq!(format!("{:?}", StringValue::new("a\"b")), r#""a\"b""#);
    assert_eq!(
        format!("{:?}", StringValue::new_block("a\"b")),
        r#"block "a\"b""#
    );
}

#[test]
fn field_argument() {
    round_trip(
        r#"query { field(arg: """hello""") }"#,
        expect![[r#"
            {
              field(arg: """hello""")
            }
        "#]],
    );
}

#[test]
fn quoted_string_unchanged() {
    round_trip(
        r#"query { field(arg: "hello") }"#,
        expect![[r#"
            {
              field(arg: "hello")
            }
        "#]],
    );
}

#[test]
fn directive_argument() {
    round_trip(
        r#"query @dir(arg: """hello""") { field }"#,
        expect![[r#"
            query @dir(arg: """hello""") {
              field
            }
        "#]],
    );
}

#[test]
fn variable_default_value() {
    round_trip(
        r#"query Q($v: String = """hello""") { field(arg: $v) }"#,
        expect![[r#"
            query Q($v: String = """hello""") {
              field(arg: $v)
            }
        "#]],
    );
}

#[test]
fn list_and_object_members() {
    // apollo-compiler prints lists and objects one item per line,
    // unlike graphql-js, but each string keeps its own syntax.
    let doc = round_trip(
        r#"query { field(list: ["""a""", "b"], object: { a: """a""", b: "b" }) }"#,
        expect![[r#"
            {
              field(list: ["""a""", "b"], object: {a: """a""", b: "b"})
            }
        "#]],
    );
    let field = doc.definitions[0]
        .as_operation_definition()
        .unwrap()
        .selection_set[0]
        .as_field()
        .unwrap();
    let list = field.arguments[0].value.as_list().unwrap();
    let Value::String(a) = &*list[0] else {
        panic!()
    };
    let Value::String(b) = &*list[1] else {
        panic!()
    };
    assert!(a.is_block());
    assert!(!b.is_block());
}

#[test]
fn input_value_definition_default() {
    let doc = ast::Document::parse(
        r#"input I { f: String = """hello""" g: String = "hello" }"#,
        "input.graphql",
    )
    .unwrap();
    expect![[r#"
        input I {
          f: String = """hello"""
          g: String = "hello"
        }
    "#]]
    .assert_eq(&doc.to_string());
}

#[test]
fn multi_line_in_nested_selection() {
    round_trip(
        "query { a { f(a: \"\"\"\n    first\n    second\n  \"\"\") } }",
        expect![[r#"
            {
              a {
                f(a: """
                first
                second
                """)
              }
            }
        "#]],
    );
}

#[test]
fn block_syntax_is_dropped_when_newlines_are_disabled() {
    let doc =
        ast::Document::parse(r#"query { field(arg: """hello""") }"#, "input.graphql").unwrap();
    expect![[r#"{ field(arg: "hello") }"#]].assert_eq(&doc.serialize().no_indent().to_string());
}

#[test]
fn block_syntax_is_dropped_when_the_value_cannot_be_a_block_string() {
    // `BlockStringValue` would strip the leading blank line, the trailing blank line,
    // the common indentation, and the carriage return.
    for value in [
        "\nleading newline",
        "trailing newline\n",
        "  a\n  b",
        "a\rb",
        "   ",
    ] {
        let value = Value::String(StringValue::new_block(value));
        let serialized = value.to_string();
        assert!(serialized.starts_with('"'), "{serialized}");
        assert!(!serialized.starts_with("\"\"\""), "{serialized}");
        let doc = ast::Document::parse(format!("{{ f(a: {serialized}) }}"), "s.graphql").unwrap();
        assert_eq!(**first_argument_value(&doc), value);
    }
}

/// Expected strings are graphql-js 17.0.2 `printBlockString(value)`,
/// which apollo-compiler matches exactly outside of any indentation
/// (and up to indentation of empty lines inside a selection set).
#[test]
fn print_block_string_edge_cases() {
    #[track_caller]
    fn check(value: &str, expected: Expect) {
        let value = Value::String(StringValue::new_block(value));
        let serialized = value.to_string();
        expected.assert_eq(&serialized);
        let doc = ast::Document::parse(format!("{{ f(a: {serialized}) }}"), "s.graphql").unwrap();
        let reparsed = first_argument_value(&doc);
        assert_eq!(**reparsed, value);
        assert!(reparsed.as_str().is_some());
        let Value::String(reparsed) = &**reparsed else {
            panic!()
        };
        assert!(reparsed.is_block());
    }

    check(
        "ends with \"quote\"",
        expect![[r#"
            """
            ends with "quote"
            """"#]],
    );
    check(
        "ends with backslash \\",
        expect![[r#"
            """
            ends with backslash \
            """"#]],
    );
    check(
        "ends with \"\"\"",
        expect![[r#"
            """
            ends with \"""
            """"#]],
    );
    check("has \"\"\" inside", expect![[r#""""has \""" inside""""#]]);
    check(
        &"x".repeat(70),
        expect![[
            r#""""xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx""""#
        ]],
    );
    check(
        &"x".repeat(71),
        expect![[r#"
            """
            xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
            """"#]],
    );
    // graphql-js measures the length in UTF-16 code units: 36 emoji are 72 units
    check(
        &"😀".repeat(36),
        expect![[r#"
            """
            😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀
            """"#]],
    );
    check(
        "   leading whitespace",
        expect![[r#""""   leading whitespace""""#]],
    );
    check("\ttab start", expect![[r#""""	tab start""""#]]);
    check(
        &format!("   {}", "x".repeat(70)),
        expect![[r#"
            """   xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
            """"#]],
    );
    check(
        "a\n  b",
        expect![[r#"
            """
            a
              b
            """"#]],
    );
    check(
        "a\n\tb",
        expect![[r#"
            """
            a
            	b
            """"#]],
    );
    check(
        "a\n\nb",
        expect![[r#"
            """
            a

            b
            """"#]],
    );
    check(
        "first\nsecond",
        expect![[r#"
            """
            first
            second
            """"#]],
    );
}

#[test]
fn serde() {
    let quoted = Value::String(StringValue::new("hello"));
    let block = Value::String(StringValue::new_block("hello"));
    let quoted_json = serde_json::to_string(&quoted).unwrap();
    let block_json = serde_json::to_string(&block).unwrap();
    expect![[r#"{"String":"hello"}"#]].assert_eq(&quoted_json);
    expect![[r#"{"String":{"value":"hello","block":true}}"#]].assert_eq(&block_json);
    for (json, expected_block) in [(quoted_json, false), (block_json, true)] {
        let Value::String(value) = serde_json::from_str::<Value>(&json).unwrap() else {
            panic!()
        };
        assert_eq!(value.as_str(), "hello");
        assert_eq!(value.is_block(), expected_block);
    }
    let Value::String(value) =
        serde_json::from_str::<Value>(r#"{"String":{"value":"x"}}"#).unwrap()
    else {
        panic!()
    };
    assert!(!value.is_block());
    assert!(serde_json::from_str::<Value>(r#"{"String":{"block":true}}"#).is_err());
    assert!(serde_json::from_str::<Value>(r#"{"String":{"value":"x","other":1}}"#).is_err());
}
