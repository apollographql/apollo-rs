// The testing framework in this file is pretty much entirely copied from
// rust-analyzer's parser and lexer tests:
// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/syntax/src/tests.rs
//
// This is also an exact setup as we have in `apollo-parser`, in the future we
// might want to consider merging the two dirs. (@lrlna)

use std::{
    env, fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;

use crate::{ApolloCompiler, ApolloDiagnostic, AstDatabase};

// To run these tests and update files:
// ```bash
// env UPDATE_EXPECT=1 cargo test --package apollo-compiler
// ```
// or on windows
// ```bash
// $env:UPDATE_EXPECT=1; cargo test --package apollo-compiler
// ```
#[test]
fn compiler_tests() {
    dir_tests(&test_data_dir(), &["ok"], "txt", |text, path| {
        let ctx = ApolloCompiler::new(text);
        let errors = ctx.validate();
        let ast = ctx.db.ast();
        assert_diagnostics_are_absent(&errors, path);
        format!("{:?}", ast)
    });

    dir_tests(&test_data_dir(), &["diagnostics"], "txt", |text, path| {
        let ctx = ApolloCompiler::new(text);
        let diagnostics = ctx.validate();
        assert_diagnostics_are_present(&diagnostics, path);
        format!("{:#?}", diagnostics)
    });
}

fn assert_diagnostics_are_present(errors: &[ApolloDiagnostic], path: &Path) {
    assert!(
        !errors.is_empty(),
        "There should be diagnostics in the file {:?}",
        path.display()
    );
}

fn assert_diagnostics_are_absent(errors: &[ApolloDiagnostic], path: &Path) {
    if !errors.is_empty() {
        let formatted: Vec<String> = errors.iter().map(|e| format!("{:?}", e)).collect();
        println!("{:?}", formatted.join("\n"));
        panic!(
            "There should be no diagnostics in the file {:?}",
            path.display(),
        );
    }
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
        let mut actual = f(&input_code, &path);
        actual.push('\n');
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
                .unwrap_or_else(|_| panic!("File at {:?} should be valid", path));
            (path, text)
        })
        .collect()
}

/// Collects paths to all `.graphql` files from `dir` in a sorted `Vec<PathBuf>`.
fn graphql_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut acc = Vec::new();
    for file in fs::read_dir(dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        if path.extension().unwrap_or_default() == "graphql" {
            acc.push(path);
        }
    }
    acc.sort();
    acc
}

/// PathBuf of test fixtures directory.
fn test_data_dir() -> PathBuf {
    project_root().join("apollo-compiler/test_data")
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
