//! Regression tests for ariadne error formatting with multibyte UTF-8 characters.
//!
//! These tests ensure that the ariadne error reporting library correctly formats
//! errors when multibyte characters (Japanese, Chinese, Korean, emoji, etc.) are
//! present in the source code, without panicking or producing garbled output.
//!
//! Note: Correctness of parsing multibyte characters is tested in the lexer,
//! parser, and compiler test suites. These tests specifically focus on error
//! formatting and display.

use apollo_compiler::parser::Parser;

#[test]
fn test_multibyte_in_descriptions_with_error() {
    let schema = r#"
"""
月次為替レート (Monthly Exchange Rate)
用户名 (User Name)
안녕하세요 (Hello in Korean)
"""
type Query {
  monthlyRate: UndefinedType
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    assert!(
        result.is_err(),
        "Expected validation error for undefined type"
    );

    // Format the error to ensure ariadne doesn't panic with multibyte characters
    let errors = result.unwrap_err();
    let error_string = format!("{:?}", errors);
    assert!(error_string.contains("UndefinedType"));
}

#[test]
fn test_multibyte_in_comments_near_error() {
    let schema = r#"
type Query {
  "月次為替レート - Monthly exchange rate"
  field: String
  
  # 次のフィールドは無効な型を持っています (This field has an invalid type)
  invalidField: UndefinedTypeHere
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    assert!(
        result.is_err(),
        "Expected validation error for undefined type"
    );

    let errors = result.unwrap_err();
    let error_string = format!("{:?}", errors);
    assert!(error_string.contains("UndefinedTypeHere"));
}

#[test]
fn test_long_multibyte_description_with_error() {
    let schema = r#"
type Query {
  """
  これは非常に長い日本語の説明文です。
  月次為替レートを取得するためのフィールドです。
  用户可以使用此字段获取数据。
  이 필드를 사용하여 데이터를 가져올 수 있습니다。
  """
  monthlyExchangeRate: InvalidType
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    assert!(
        result.is_err(),
        "Expected validation error for undefined type"
    );

    let errors = result.unwrap_err();
    let error_string = format!("{:?}", errors);
    assert!(error_string.contains("InvalidType"));
}

#[test]
fn test_multibyte_in_string_values() {
    let schema = r#"
type Query {
  field: String @deprecated(reason: "このフィールドは廃止されました。月次為替レートを使用してください。")
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    // This should parse successfully
    assert!(
        result.is_ok(),
        "Schema with multibyte characters in directive arguments should parse successfully"
    );
}

#[test]
fn test_emoji_in_descriptions() {
    let schema = r#"
"""
🚀 GraphQL API for space missions
🌍 Earth observation data
"""
type Query {
  "🛰️ Satellite data"
  satellites: [String!]!
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    assert!(
        result.is_ok(),
        "Schema with emoji characters should parse successfully"
    );
}

#[test]
fn test_mixed_multibyte_and_ascii_error() {
    let schema = r#"
type Query {
  """
  Field description with 日本語 Japanese, 中文 Chinese, and English
  """
  mixedField: NonExistentType
}
"#;

    let result = Parser::new().parse_mixed_validate(schema.to_string(), "test.graphql");
    assert!(result.is_err(), "Expected validation error");

    let errors = result.unwrap_err();
    let error_string = format!("{:?}", errors);
    // Ensure the error message is properly formatted even with multibyte characters nearby
    assert!(error_string.contains("NonExistentType"));
}
