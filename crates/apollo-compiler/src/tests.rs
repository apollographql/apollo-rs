// The testing framework in this file is pretty much entirely copied from
// rust-analyzer's parser and lexer tests:
// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/syntax/src/tests.rs
//
// This is also an exact setup as we have in `apollo-parser`, in the future we
// might want to consider merging the two dirs. (@lrlna)

use indexmap::IndexMap;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

use expect_test::expect_file;

use crate::{ast, ApolloCompiler, ApolloDiagnostic, CstDatabase, FileId};

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
        let mut compiler = ApolloCompiler::new();
        let file_id = compiler.add_document(text, path.file_name().unwrap());

        let errors = compiler.validate();
        assert_diagnostics_are_absent(&errors, path);

        let cst = compiler.db.cst(file_id);
        format!("{cst:?}")
    });

    dir_tests(&test_data_dir(), &["diagnostics"], "txt", |text, path| {
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(text, path.file_name().unwrap());

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }
        assert_diagnostics_are_present(&diagnostics, path);
        format!("{diagnostics:#?}")
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
        for diagnostic in errors {
            println!("{diagnostic}");
        }
        panic!(
            "There should be no diagnostics in the file {:?}",
            path.display(),
        );
    }
}

#[test]
fn serializer_tests() {
    let test_data_dir = test_data_dir();
    for subdir in ["ok", "diagnostics"] {
        let output_dir = test_data_dir.join("serializer").join(subdir);
        let collected = collect_graphql_files(&test_data_dir, &[subdir]);
        for (input_path, input) in collected {
            let output_path = output_dir.join(input_path.file_name().unwrap());
            let ast = ast::Document::parse(&input).document;
            let serialized = ast.to_string();
            expect_file![output_path].assert_eq(&serialized);
            let ast2 = ast::Document::parse(&serialized).document;
            if ast != ast2 {
                panic!(
                    "Serialization does not round-trip:\n\
                    {input}\n=>\n{ast:#?}\n=>\n{serialized}\n=>\n{ast2:#?}\n=>\n{ast2}\n"
                );
            }
        }
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
    FileId::reset();

    for (path, input_code) in collect_graphql_files(test_data_dir, paths) {
        let mut actual = f(&input_code, &path);
        actual.push('\n');
        let path = path.with_extension(outfile_extension);
        expect_file![path].assert_eq(&actual)
    }
}

/// Collects all `.graphql` files from `dir` subdirectories defined by `paths`.
fn collect_graphql_files(root_dir: &Path, paths: &[&str]) -> Vec<(PathBuf, String)> {
    let mut files = paths
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
        .collect::<Vec<_>>();
    // Sort alphabetically to ensure consistent File IDs
    files.sort_by(|a, b| a.0.cmp(&b.0));
    files
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
