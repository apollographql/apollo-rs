use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::Duration,
};

use anyhow::{anyhow, Result};
use apollo_compiler::{ApolloCompiler, FileId};
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};

fn main() -> Result<()> {
    let dir = Path::new("crates/apollo-compiler/examples/documents");
    let mut watcher = FileWatcher::default();
    watcher.watch(dir)
}

#[derive(Default)]
pub struct FileWatcher {
    compiler: ApolloCompiler,
    manifest: HashMap<PathBuf, FileId>,
}

impl FileWatcher {
    pub fn new() -> Self {
        Self {
            compiler: ApolloCompiler::new(),
            manifest: HashMap::new(),
        }
    }

    // The `watch` fn first goes over every document in a given directory and
    // creates it as a new document with compiler's
    // `compiler.create_document()`.
    //
    // We then watch for file changes, and update
    // each changed document with `compiler.update_document()`.
    pub fn watch(&mut self, dir: impl AsRef<Path>) -> Result<()> {
        for entry in fs::read_dir(&dir)? {
            let (proposed_document, src_path) = get_schema_and_maybe_path(entry?)?;
            self.add_document(proposed_document, src_path)?;
        }

        self.validate();

        self.watch_broadcast(dir)
    }

    fn watch_broadcast(&mut self, dir: impl AsRef<Path>) -> Result<()> {
        let (broadcaster, listener) = channel();
        let mut watcher = watcher(broadcaster, Duration::from_secs(1))?;
        watcher.watch(&dir, RecursiveMode::NonRecursive)?;
        println!("watching {} for changes", dir.as_ref().display());
        loop {
            match listener.recv() {
                Ok(event) => match &event {
                    DebouncedEvent::NoticeWrite(path) => {
                        println!("changes detected in {}", &path.display())
                    }
                    DebouncedEvent::Create(path) => match fs::read_to_string(path) {
                        Ok(contents) => {
                            println!("detected a new file {}", &path.display());
                            self.add_document(contents, path.to_path_buf())?;
                            self.validate();
                        }
                        Err(e) => {
                            println!(
                                "could not read {} from disk, {:?}",
                                &dir.as_ref().display(),
                                Some(anyhow!("{}", e)),
                            );
                        }
                    },
                    DebouncedEvent::Write(path) => {
                        match fs::read_to_string(path) {
                            Ok(contents) => {
                                let file_id = self.manifest.get(path);
                                if let Some(file_id) = file_id {
                                    self.compiler.update_document(*file_id, &contents);
                                } else {
                                    self.add_document(contents, path.to_path_buf())?;
                                }
                                self.validate();
                            }
                            Err(e) => {
                                println!(
                                    "could not read {} from disk, {:?}",
                                    &dir.as_ref().display(),
                                    Some(anyhow!("{}", e)),
                                );
                            }
                        };
                    }
                    DebouncedEvent::Error(e, _) => {
                        println!(
                            "unknown error while watching {},  {:?}",
                            &dir.as_ref().display(),
                            Some(anyhow!("{}", e)),
                        );
                    }
                    _ => {}
                },
                Err(e) => {
                    println!(
                        "unknown error while watching {},  {:?}",
                        &dir.as_ref().display(),
                        Some(anyhow!("{}", e)),
                    );
                }
            }
        }
    }

    fn add_document(
        &mut self,
        proposed_document: String,
        src_path: PathBuf,
    ) -> Result<(), anyhow::Error> {
        let file_id = self.compiler.create_document(&proposed_document, &src_path);
        let full_path = fs::canonicalize(src_path)?;
        self.manifest.insert(full_path, file_id);
        Ok(())
    }

    fn validate(&self) {
        let diagnostics = self.compiler.validate();
        for diag in diagnostics {
            println!("{}", diag)
        }
    }
}

fn get_schema_and_maybe_path(entry: DirEntry) -> Result<(String, PathBuf)> {
    let src = fs::read_to_string(entry.path()).expect("Could not read document file.");
    Ok((src, entry.path()))
}
