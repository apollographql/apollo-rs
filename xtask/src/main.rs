mod ast_src;
mod codegen;
mod utils;

use anyhow::{bail, Result};
use std::{
    env,
    path::{Path, PathBuf},
};
use structopt::StructOpt;
use xshell::{cmd, pushenv};

fn main() -> Result<()> {
    let app = Xtask::from_args();
    app.run()
}

#[derive(Debug, StructOpt)]
#[structopt(name = "xtask", about = "apollo-rs development workflows")]
struct Xtask {
    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(long = "verbose", short = "v", global = true)]
    verbose: bool,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    /// Perform code generation for the parser
    Codegen(codegen::Codegen),
}

impl Xtask {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Codegen(command) => command.run(self.verbose),
        }?;

        Ok(())
    }
}

fn root_path() -> PathBuf {
    Path::new(
        &env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| env!("CARGO_MANIFEST_DIR").to_owned()),
    )
    .ancestors()
    .nth(1)
    .unwrap()
    .to_path_buf()
}

fn rustfmt() -> Result<()> {
    let out = cmd!("rustfmt --version").read()?;
    if !out.contains("stable") {
        bail!(
            "Failed to run rustfmt from toolchain 'stable'. \
             Please run `rustup component add rustfmt --toolchain stable` to install it.",
        )
    }
    Ok(())
}

fn reformat(text: &str) -> Result<String> {
    let _e = pushenv("RUSTUP_TOOLCHAIN", "stable");
    rustfmt()?;
    let stdout = cmd!("rustfmt --config fn_single_line=true")
        .stdin(text)
        .read()?;
    Ok(format!(
        "//! {}\n\n{}\n",
        "This is a generated file, please do not edit.", stdout
    ))
}

pub(crate) fn ensure_file_contents(file: &Path, contents: &str) -> Result<()> {
    match std::fs::read_to_string(file) {
        Ok(old_contents) if normalize_newlines(&old_contents) == normalize_newlines(contents) => {
            return Ok(())
        }
        _ => (),
    }
    let display_path = file.strip_prefix(&root_path()).unwrap_or(file);
    eprintln!(
        "\n\x1b[31;1merror\x1b[0m: {} was not up-to-date, updating\n",
        display_path.display()
    );
    if std::env::var("CI").is_ok() {
        eprintln!("    NOTE: run `cargo test` locally and commit the updated files\n");
    }
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::write(file, contents).unwrap();
    bail!("{} was not up to date and has been updated. Make sure to re-run cargo check and cargo test to accomodate the updates.", file.display());
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}
