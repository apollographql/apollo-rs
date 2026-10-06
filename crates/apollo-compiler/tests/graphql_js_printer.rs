//! Tests for `to_graphql_js_string()`, the graphql-js compatible printer.
//!
//! Expected outputs in `test_data/graphql_js_printer/expected` were generated
//! by graphql-js 17.0.2 `print(parse(input))`,
//! see `test_data/graphql_js_printer/README.md`.
//! They are deliberately *not* `expect_file!` snapshots:
//! `UPDATE_EXPECT=1` must not overwrite them with the Rust output.

use apollo_compiler::ast;
use apollo_compiler::name;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use pretty_assertions::assert_eq;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn test_data_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("test_data")
}

fn parse_partial(input: &str, path: &Path) -> ast::Document {
    ast::Document::parse(input, path.file_name().unwrap().to_str().unwrap())
        .unwrap_or_else(|invalid| invalid.partial)
}

/// Every input in `test_data/{ok,diagnostics}` and `test_data/graphql_js_printer/input`
/// prints exactly like graphql-js, or is listed in `expected/unsupported.txt`
/// because graphql-js cannot parse it.
#[test]
fn matches_graphql_js_print() {
    let test_data_dir = test_data_dir();
    let printer_dir = test_data_dir.join("graphql_js_printer");
    let expected_dir = printer_dir.join("expected");
    let unsupported = fs::read_to_string(expected_dir.join("unsupported.txt")).unwrap();
    let unsupported: Vec<&str> = unsupported
        .lines()
        .map(|line| line.split_once(':').unwrap().0)
        .collect();

    let mut checked = 0;
    let mut skipped = 0;
    for (subdir, input_dir) in [
        ("ok", test_data_dir.join("ok")),
        ("diagnostics", test_data_dir.join("diagnostics")),
        ("input", printer_dir.join("input")),
    ] {
        let mut input_paths: Vec<PathBuf> = fs::read_dir(&input_dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "graphql"))
            .collect();
        input_paths.sort();
        for input_path in input_paths {
            let file_name = input_path.file_name().unwrap().to_str().unwrap();
            let key = format!("{subdir}/{file_name}");
            let expected_path = expected_dir.join(subdir).join(file_name);
            if unsupported.contains(&key.as_str()) {
                assert!(
                    !expected_path.exists(),
                    "{key} is listed as unsupported but has an expected output"
                );
                skipped += 1;
                continue;
            }
            let input = fs::read_to_string(&input_path).unwrap();
            let expected = fs::read_to_string(&expected_path).unwrap_or_else(|_| {
                panic!(
                    "missing {}: regenerate with generate.js",
                    expected_path.display()
                )
            });
            let doc = parse_partial(&input, &input_path);
            let printed = doc.to_graphql_js_string();
            assert_eq!(printed, expected, "{key} differs from graphql-js output");
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} inputs were checked");
    assert_eq!(skipped, unsupported.len());

    let mut expected_count = 0;
    for subdir in ["ok", "diagnostics", "input"] {
        expected_count += fs::read_dir(expected_dir.join(subdir)).unwrap().count();
    }
    assert_eq!(
        expected_count,
        checked,
        "stale files in {}",
        expected_dir.display()
    );
}

/// The graphql-js output of a valid document parses back to the same AST
#[test]
fn round_trips_through_the_parser() {
    let test_data_dir = test_data_dir();
    let mut round_tripped = 0;
    for input_dir in [
        test_data_dir.join("ok"),
        test_data_dir.join("graphql_js_printer").join("input"),
    ] {
        for entry in fs::read_dir(&input_dir).unwrap() {
            let input_path = entry.unwrap().path();
            if input_path.extension().is_none_or(|ext| ext != "graphql") {
                continue;
            }
            let input = fs::read_to_string(&input_path).unwrap();
            let Ok(original) = ast::Document::parse(&input, "input.graphql") else {
                continue;
            };
            let printed = original.to_graphql_js_string();
            let reparsed = ast::Document::parse(&printed, "printed.graphql")
                .unwrap_or_else(|err| panic!("{}: {}", input_path.display(), err.errors));
            assert_eq!(
                original.definitions,
                reparsed.definitions,
                "{} does not round-trip",
                input_path.display()
            );
            round_tripped += 1;
        }
    }
    assert!(round_tripped > 40);
}

#[test]
fn executable_document() {
    let schema = Schema::parse_and_validate(
        r#"
        type Query { user(id: ID!, options: Options): User }
        type User { id: ID! name: String friends(first: Int): [User] }
        input Options { aaaaaaaaaaaaaaaaaaaaaaaa: String, bbbbbbbbbbbbbbbbbbbbbbbbbb: String }
        "#,
        "schema.graphql",
    )
    .unwrap();
    let source = r#"
        query GetUser($id: ID!, $first: Int = 10) {
          user(id: $id, options: {aaaaaaaaaaaaaaaaaaaaaaaa: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", bbbbbbbbbbbbbbbbbbbbbbbbbb: "b"}) {
            ...UserFields
            friends(first: $first) { id }
          }
        }
        fragment UserFields on User @custom { id name }
    "#;
    let doc = ExecutableDocument::parse(&schema, source, "query.graphql").unwrap();
    let expected = "\
query GetUser($id: ID!, $first: Int = 10) {
  user(
    id: $id
    options: {
      aaaaaaaaaaaaaaaaaaaaaaaa: \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"
      bbbbbbbbbbbbbbbbbbbbbbbbbb: \"b\"
    }
  ) {
    ...UserFields
    friends(first: $first) {
      id
    }
  }
}

fragment UserFields on User @custom {
  id
  name
}";
    assert_eq!(doc.to_graphql_js_string(), expected);
    let operation = doc.operations.get(Some("GetUser")).unwrap();
    assert_eq!(
        operation.to_graphql_js_string(),
        expected.split("\n\n").next().unwrap()
    );
    let fragment = &doc.fragments["UserFields"];
    assert_eq!(
        fragment.to_graphql_js_string(),
        expected.split("\n\n").nth(1).unwrap()
    );
    assert_eq!(
        fragment.selection_set.to_graphql_js_string(),
        "{\n  id\n  name\n}"
    );
    assert_eq!(
        operation.selection_set.selections[0].to_graphql_js_string(),
        expected
            .lines()
            .skip(1)
            .take_while(|line| *line != "}")
            .map(|line| line.strip_prefix("  ").unwrap_or(line))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn string_escaping() {
    // Same table as graphql-js `printString`
    let cases = [
        ("", r#""""#),
        ("plain", r#""plain""#),
        ("say \"hi\"", r#""say \"hi\"""#),
        ("back\\slash", r#""back\\slash""#),
        ("\u{8}\t\n\u{c}\r", r#""\b\t\n\f\r""#),
        ("\0\u{1}\u{1f}", r#""\u0000\u0001\u001F""#),
        ("\u{7f}\u{85}\u{9f}", r#""\u007F\u0085\u009F""#),
        ("\u{a0}é中😀", "\"\u{a0}é中😀\""),
        ("a/b'c", r#""a/b'c""#),
    ];
    for (input, expected) in cases {
        assert_eq!(ast::Value::from(input).to_graphql_js_string(), expected);
    }
}

#[test]
fn descriptions_print_as_block_strings_when_printable() {
    fn print(description: &str) -> String {
        ast::ScalarTypeDefinition {
            description: Some(Node::new_str(description)),
            name: name!(S),
            directives: Default::default(),
        }
        .to_graphql_js_string()
    }
    // Generated with graphql-js 17.0.2 `print()` with `block: isPrintableAsBlockString(value)`
    let cases = [
        ("simple", "\"\"\"simple\"\"\"\nscalar S"),
        ("", "\"\"\"\"\"\"\nscalar S"),
        ("  leading space", "\"\"\"  leading space\"\"\"\nscalar S"),
        ("ends with quote\"", "\"\"\"\nends with quote\"\n\"\"\"\nscalar S"),
        ("ends with slash\\", "\"\"\"\nends with slash\\\n\"\"\"\nscalar S"),
        ("has \"\"\" inside", "\"\"\"has \\\"\"\" inside\"\"\"\nscalar S"),
        ("ends with \"\"\"", "\"\"\"\nends with \\\"\"\"\n\"\"\"\nscalar S"),
        ("two\nlines", "\"\"\"\ntwo\nlines\n\"\"\"\nscalar S"),
        ("first\n  indented", "\"\"\"\nfirst\n  indented\n\"\"\"\nscalar S"),
        ("line\n\nblank above", "\"\"\"\nline\n\nblank above\n\"\"\"\nscalar S"),
        ("\nleading newline", "\"\\nleading newline\"\nscalar S"),
        ("trailing newline\n", "\"trailing newline\\n\"\nscalar S"),
        ("  common\n  indent", "\"  common\\n  indent\"\nscalar S"),
        ("with\ttab", "\"\"\"with\ttab\"\"\"\nscalar S"),
        ("control \u{1}", "\"control \\u0001\"\nscalar S"),
        ("cr\r", "\"cr\\r\"\nscalar S"),
        (
            "exactly seventy characters long xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "\"\"\"\nexactly seventy characters long xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\n\"\"\"\nscalar S",
        ),
        (
            "exactly sixty-nine characters long xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx",
            "\"\"\"exactly sixty-nine characters long xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\"\"\"\nscalar S",
        ),
        // 35 emoji are 70 UTF-16 code units, which is not > 70
        (
            "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀",
            "\"\"\"😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀\"\"\"\nscalar S",
        ),
        (
            "😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀a",
            "\"\"\"\n😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀😀a\n\"\"\"\nscalar S",
        ),
    ];
    for (input, expected) in cases {
        assert_eq!(print(input), expected, "input: {input:?}");
    }
}

#[test]
fn default_display_is_unchanged() {
    let source = "query Q($a: Int = 1) { f(x: {a: 1}, y: [1, 2]) @d { g } }";
    let doc = ast::Document::parse(source, "q.graphql").unwrap();
    assert_eq!(
        doc.to_string(),
        "query Q($a: Int = 1) {\n  f(x: {a: 1}, y: [1, 2]) @d {\n    g\n  }\n}\n"
    );
    assert_eq!(
        doc.to_graphql_js_string(),
        "query Q($a: Int = 1) {\n  f(x: { a: 1 }, y: [1, 2]) @d {\n    g\n  }\n}"
    );
}
