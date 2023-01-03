use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::Duration,
};

use anyhow::{anyhow, Result};
use apollo_compiler::{ApolloCompiler, ApolloDiagnostic, FileId};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

#[derive(Default)]
struct Manifest {
    sources: HashMap<PathBuf, FileId>,
}

fn main() -> Result<()> {
    validate()?;
    Ok(())
}

fn validate() -> Result<()> {
    let mut compiler = ApolloCompiler::new();

    let dir = Path::new("crates/apollo-compiler/examples/documents");
    let mut manifest = Manifest::default();

    for entry in fs::read_dir(dir)? {
        let (proposed_document, src_path) = get_schema_and_maybe_path(entry?)?;
        let file_id = compiler.create_document(&proposed_document, &src_path);
        manifest.sources.insert(src_path, file_id);
    }

    print_diagnostics(compiler.validate());

    let (broadcaster, listener) = channel();
    let mut watcher = watcher(broadcaster, Duration::from_secs(1))?;
    watcher.watch(&dir, RecursiveMode::NonRecursive)?;

    println!("{}", format!("watching {} for changes", dir.display()));
    loop {
        match listener.recv() {
            Ok(event) => match &event {
                DebouncedEvent::NoticeWrite(path) => {
                    println!("{}", format!("Change detected in {}", &path.display()))
                }
                DebouncedEvent::Write(path) => {
                    match fs::read_to_string(&path) {
                        Ok(contents) => {
                            let file_id = manifest.sources.get(path);
                            if let Some(file_id) = file_id {
                                compiler.update_document(*file_id, &contents);
                            } else {
                                let file_id = compiler.create_document(&contents, &path);
                                manifest.sources.insert(path.to_path_buf(), file_id);
                            }
                            print_diagnostics(compiler.validate());
                        }
                        Err(e) => {
                            println!(
                                "{} {:?}",
                                format!("Could not read {} from disk", &dir.display()),
                                Some(anyhow!("{}", e)),
                            );
                        }
                    };
                }
                DebouncedEvent::Error(e, _) => {
                    println!(
                        "{} {:?}",
                        format!("unknown error while watching {}", &dir.display()),
                        Some(anyhow!("{}", e)),
                    );
                }
                _ => {}
            },
            Err(e) => {
                println!(
                    "{} {:?}",
                    format!("unknown error while watching {}", &dir.display()),
                    Some(anyhow!(e)),
                );
            }
        }
    }
}

fn get_schema_and_maybe_path(entry: DirEntry) -> Result<(String, PathBuf)> {
    let src = fs::read_to_string(entry.path()).expect("Could not read document file.");
    Ok((src, entry.path()))
}

fn print_diagnostics(diagnostics: Vec<ApolloDiagnostic>) {
    for diag in diagnostics {
        println!("{}", diag)
    }
}
