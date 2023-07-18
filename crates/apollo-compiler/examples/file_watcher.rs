use std::{
    collections::HashMap,
    fs::{self, DirEntry},
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::Duration,
};

use anyhow::{anyhow, Result};
use apollo_compiler::{ApolloCompiler, FileId};
use notify::{Config, EventKind, PollWatcher, RecursiveMode, Watcher};

fn main() -> Result<()> {
    let dir = Path::new("crates/apollo-compiler/examples/documents");
    let mut watcher = FileWatcher::new();
    watcher.watch(dir)
}

pub struct FileWatcher {
    compiler: ApolloCompiler,
    manifest: HashMap<PathBuf, FileId>,
}

#[allow(clippy::new_without_default)]
impl FileWatcher {
    pub fn new() -> Self {
        Self {
            compiler: ApolloCompiler::new(),
            manifest: HashMap::new(),
        }
    }

    // The `watch` fn first goes over every document in a given directory and
    // creates it as a new document with compiler's
    // `compiler.add_document()`.
    //
    // We then watch for file changes, and update
    // each changed document with `compiler.update_document()`.
    pub fn watch(&mut self, dir: impl AsRef<Path>) -> Result<()> {
        for entry in fs::read_dir(&dir)? {
            let (proposed_document, src_path) = get_schema_and_maybe_path(entry?)?;
            self.add_document(proposed_document, src_path)?;
        }

        self.validate();

        self.watch_broadcast(dir.as_ref())
    }

    fn watch_broadcast(&mut self, dir: &Path) -> Result<()> {
        let (broadcaster, listener) = channel();
        let mut watcher = PollWatcher::new(
            move |res| {
                broadcaster.send(res).unwrap();
            },
            Config::default().with_poll_interval(Duration::from_secs(1)),
        )?;
        watcher.watch(&dir, RecursiveMode::NonRecursive)?;
        println!("watching {} for changes", dir.display());
        loop {
            match listener.recv() {
                Ok(Ok(event)) => {
                    for path in event.paths {
                        if path.is_dir() {
                            continue;
                        }

                        match event.kind {
                            EventKind::Any => {
                                println!("changes detected in {}", path.display())
                            }
                            EventKind::Create(_) | EventKind::Modify(_) => {
                                match fs::read_to_string(&path) {
                                    Ok(contents) => {
                                        println!("changes detected in {}", path.display());
                                        let file_id = self.manifest.get(&fs::canonicalize(&path)?);
                                        if let Some(file_id) = file_id {
                                            self.compiler.update_document(*file_id, &contents);
                                        } else {
                                            self.add_document(contents, path)?;
                                        }
                                        self.validate();
                                    }
                                    Err(e) => {
                                        println!(
                                            "could not read {} from disk, {:?}",
                                            dir.display(),
                                            Some(anyhow!("{}", e)),
                                        );
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!(
                        "unknown error while watching {},  {:?}",
                        &dir.display(),
                        Some(anyhow!("{}", e)),
                    );
                }
                Err(e) => {
                    println!(
                        "unknown error while watching {},  {:?}",
                        &dir.display(),
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
        let file_id = self.compiler.add_document(&proposed_document, &src_path);
        let full_path = fs::canonicalize(&src_path)?;
        self.manifest.insert(full_path, file_id);
        Ok(())
    }

    fn validate(&self) {
        let diagnostics = self.compiler.validate();
        for diag in diagnostics {
            println!("{diag}")
        }
    }
}

fn get_schema_and_maybe_path(entry: DirEntry) -> Result<(String, PathBuf)> {
    let src = fs::read_to_string(entry.path()).expect("Could not read document file.");
    Ok((src, entry.path()))
}
