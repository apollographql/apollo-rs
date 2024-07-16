// The testing framework in this file is pretty much entirely copied from
// rust-analyzer's parser and lexer tests:
// https://github.com/rust-analyzer/rust-analyzer/blob/master/crates/syntax/src/tests.rs
//
// This is also an exact setup as we have in `apollo-parser`, in the future we
// might want to consider merging the two dirs. (@lrlna)

// Note: ALL #[test] functions must also have #[serial], to make FileId::reset work correctly

use apollo_compiler::ast;
use apollo_compiler::collections::IndexMap;
use apollo_compiler::name;
use apollo_compiler::schema;
use apollo_compiler::ty;
use apollo_compiler::validation::DiagnosticList;
use apollo_compiler::FileId;
use apollo_compiler::Schema;
use expect_test::expect_file;
use serial_test::serial;
use std::env;
use std::fmt::Write;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

// To run these tests and update files:
// ```bash
// env UPDATE_EXPECT=1 cargo test --package apollo-compiler
// ```
// or on windows
// ```bash
// $env:UPDATE_EXPECT=1; cargo test --package apollo-compiler
// ```
#[test]
#[serial]
fn validation() {
    dir_tests(&test_data_dir(), &["ok"], "txt", |text, path| {
        let file_name = path.file_name().unwrap();
        match apollo_compiler::parse_mixed_validate(text, file_name) {
            Err(errors) => {
                println!("{errors}");
                panic!(
                    "There should be no diagnostics in the file {}",
                    path.display(),
                );
            }
            Ok((schema, executable)) => {
                format!("{:#?}\n{:#?}", schema.into_inner(), executable.into_inner())
            }
        }
    });

    dir_tests(&test_data_dir(), &["diagnostics"], "txt", |text, path| {
        let filename = path.file_name().unwrap().to_str().unwrap();
        let is_type_system = filename.contains("type_system_document");
        let is_executable = filename.contains("executable_document");
        let schema_validation_errors;
        let executable_validation_errors;
        if is_type_system {
            schema_validation_errors = Schema::parse_and_validate(text, filename)
                .err()
                .map(|e| e.errors);
            executable_validation_errors = None;
        } else if is_executable {
            schema_validation_errors = None;
            executable_validation_errors = match ast::Document::parse(text, filename) {
                Err(e) => Some(e.errors),
                Ok(ast) => ast.validate_standalone_executable().err(),
            };
        } else {
            schema_validation_errors = apollo_compiler::parse_mixed_validate(text, filename).err();
            executable_validation_errors = None;
        };
        let mut formatted = String::new();
        if let Some(errors) = &schema_validation_errors {
            write!(&mut formatted, "{errors:#}").unwrap()
        }
        if let Some(errors) = &executable_validation_errors {
            write!(&mut formatted, "{errors:#}").unwrap()
        }
        assert_diagnostics_are_present(
            &schema_validation_errors,
            &executable_validation_errors,
            path,
        );
        formatted
    });
}

fn assert_diagnostics_are_present(
    schema_validation_errors: &Option<DiagnosticList>,
    executable_validation_errors: &Option<DiagnosticList>,
    path: &Path,
) {
    assert!(
        schema_validation_errors.is_some() || executable_validation_errors.is_some(),
        "There should be diagnostics in the file {:?}",
        path.display()
    );
}

#[test]
#[serial]
fn serialize_and_reparse_ast() {
    FileId::reset();
    let test_data_dir = test_data_dir();
    for subdir in ["ok", "diagnostics"] {
        let output_dir = test_data_dir.join("serializer").join(subdir);
        let collected = collect_graphql_files(&test_data_dir, &[subdir]);
        for (input_path, input) in collected {
            let output_path = output_dir.join(input_path.file_name().unwrap());
            let original = ast::Document::parse(&input, "input.graphql")
                .unwrap_or_else(|invalid| invalid.partial);
            let serialized = original.to_string();
            expect_file![output_path].assert_eq(&serialized);

            let round_tripped = ast::Document::parse(&serialized, "serialized.graphql")
                .unwrap_or_else(|invalid| invalid.partial);
            if original != round_tripped {
                panic!(
                    "Serialization does not round-trip for {input_path:?}:\n\
                     {input}\n=>\n{original:#?}\n=>\n{serialized}\n=>\n\
                     {round_tripped:#?}\n=>\n{round_tripped}\n"
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
    let mut seen = IndexMap::with_hasher(Default::default());
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

#[test]
fn test_invalid_synthetic_node() {
    let mut schema = Schema::new();
    schema.types.insert(
        name!("Obj"),
        schema::ObjectType {
            description: Default::default(),
            name: name!("Obj"),
            implements_interfaces: Default::default(),
            directives: Default::default(),
            fields: [(
                name!("field"),
                schema::FieldDefinition {
                    description: Default::default(),
                    name: name!("field"),
                    arguments: Default::default(),
                    ty: ty!(UndefinedType),
                    directives: Default::default(),
                }
                .into(),
            )]
            .into_iter()
            .collect(),
        }
        .into(),
    );
    schema.schema_definition.make_mut().query = Some(name!("Obj").into());
    let expected = expect_test::expect![[r#"
        Error: cannot find type `UndefinedType` in this document
    "#]];
    expected.assert_eq(&schema.validate().unwrap_err().errors.to_string());
}
