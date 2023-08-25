// The testing framework in this file is pretty much entirely copied from rust-analyzer's parser and lexer tests:
// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/syntax/src/tests.rs

use indexmap::IndexMap;
use std::{
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;

use crate::{Error, Lexer, Parser, Token};

// To run these tests and update files:
// ```bash
// env UPDATE_EXPECT=1 cargo test --package apollo-parser
// ```
// or on windows
// ```bash
// $env:UPDATE_EXPECT=1; cargo test --package apollo-parser
// ```
#[test]
fn lexer_tests() {
    dir_tests(&test_data_dir(), &["lexer/ok"], "txt", |text, path| {
        let (tokens, errors) = Lexer::new(text).lex();
        assert_errors_are_absent(&errors, path);
        dump_tokens_and_errors(&tokens, &errors)
    });

    dir_tests(&test_data_dir(), &["lexer/err"], "txt", |text, path| {
        let (tokens, errors) = Lexer::new(text).lex();
        assert_errors_are_present(&errors, path);
        dump_tokens_and_errors(&tokens, &errors)
    });
}

#[test]
fn parser_tests() {
    dir_tests(&test_data_dir(), &["parser/ok"], "txt", |text, path| {
        let parser = Parser::new(text);
        let cst = parser.parse();
        assert_errors_are_absent(&cst.errors().cloned().collect::<Vec<_>>(), path);
        format!("{cst:?}")
    });

    dir_tests(&test_data_dir(), &["parser/err"], "txt", |text, path| {
        let parser = Parser::new(text);
        let cst = parser.parse();
        assert_errors_are_present(&cst.errors().cloned().collect::<Vec<_>>(), path);
        format!("{cst:?}")
    });
}

fn assert_errors_are_present(errors: &[Error], path: &Path) {
    assert!(
        !errors.is_empty(),
        "There should be errors in the file {:?}",
        path.display()
    );
}

fn assert_errors_are_absent(errors: &[Error], path: &Path) {
    if !errors.is_empty() {
        println!(
            "errors: {}",
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        );
        panic!("There should be no errors in the file {:?}", path.display(),);
    }
}

/// Concatenate tokens and errors.
fn dump_tokens_and_errors(tokens: &[Token], errors: &[Error]) -> String {
    let mut acc = String::new();
    for token in tokens {
        writeln!(acc, "{token:?}").unwrap();
    }
    for err in errors {
        writeln!(acc, "{err:?}").unwrap();
    }
    acc
}

/// Compares input code taken from a `.graphql` file in test_fixtures and its
/// expected output in the corresponding `.txt` file.
///
/// The test fails if the ouptut differs.
///
/// If a matching file does not exist, it will be created, filled with output,
/// but fail the test.
fn dir_tests<F>(test_data_dir: &Path, paths: &[&str], outfile_extension: &str, f: F)
where
    F: Fn(&str, &Path) -> String,
{
    for (path, input_code) in collect_graphql_files(test_data_dir, paths) {
        let actual = f(&input_code, &path);
        let path = path.with_extension(outfile_extension);
        expect_file![path].assert_eq(&actual)
    }
}

/// Collects all `.graphql` files from `dir` subdirectories defined by `paths`.
fn collect_graphql_files(root_dir: &Path, paths: &[&str]) -> Vec<(PathBuf, String)> {
    paths
        .iter()
        .flat_map(|path| {
            let path = root_dir.to_owned().join(path);
            graphql_files_in_dir(&path).into_iter()
        })
        .map(|path| {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("File at {path:?} should be valid"));
            (path, text)
        })
        .collect()
}

/// Collects paths to all `.graphql` files from `dir` in a sorted `Vec<PathBuf>`.
fn graphql_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut paths = fs::read_dir(dir)
        .unwrap()
        .map(|file| {
            let file = file?;
            let path = file.path();
            if path.extension().unwrap_or_default() == "graphql" {
                Ok(Some(path))
            } else {
                Ok(None)
            }
        })
        // Get rid of the `None`s
        .filter_map(|result: std::io::Result<_>| result.transpose())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    paths.sort();

    // Check for duplicate numbers.
    let mut seen = IndexMap::new();
    let next_number = paths.len() + 1;
    for path in &paths {
        let file_name = path.file_name().unwrap().to_string_lossy();
        let (number, name): (usize, _) = match file_name.split_once('_') {
            Some((number, name)) => match number.parse() {
                Ok(number) => (number, name),
                Err(err) => {
                    panic!("Invalid test file name: {path:?} does not start with a number ({err})")
                }
            },
            None => panic!("Invalid test file name: {path:?} does not start with a number"),
        };

        if let Some(existing) = seen.get(&number) {
            let suggest = dir.join(format!("{next_number:03}_{name}"));
            panic!("Conflicting test file: {path:?} has the same number as {existing:?}. Suggested name: {suggest:?}");
        }

        seen.insert(number, path);
    }

    paths
}

/// PathBuf of test fixtures directory.
fn test_data_dir() -> PathBuf {
    project_root().join("apollo-parser/test_data")
}

/// apollo-rs project root.
fn project_root() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}
