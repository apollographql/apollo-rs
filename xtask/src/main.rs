mod codegen;
mod cst_src;
mod utils;

use std::{
    env,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use clap::{Parser, Subcommand};
use xshell::{cmd, Shell};

fn main() -> Result<()> {
    let app = Xtask::parse();
    app.run()
}

#[derive(Debug, Parser)]
#[clap(name = "xtask", about = "apollo-rs development workflows")]
struct Xtask {
    #[clap(subcommand)]
    pub command: Command,

    #[clap(long = "verbose", short = 'v', global = true)]
    verbose: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Perform code generation for the parser
    Codegen(codegen::Codegen),
    /// Check Rust code formating and run clippy
    Lint,
    /// Reformat Rust code
    Fmt,
}

fn run_lint() -> Result<()> {
    let sh = Shell::new()?;

    cmd!(sh, "cargo fmt --all -- --check").run()?;

    cmd!(
        sh,
        "cargo clippy --all-targets --all-features -- -D warnings"
    )
    .run()?;

    Ok(())
}


fn run_fmt() -> Result<()> {
    let sh = Shell::new()?;

    cmd!(sh, "cargo fmt --all").run()?;

    Ok(())
}

impl Xtask {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Codegen(command) => command.run(self.verbose),
            Command::Lint => run_lint(),
            Command::Fmt => run_fmt(),
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
    let sh = Shell::new()?;

    let out = cmd!(sh, "rustfmt --version").read()?;
    if !out.contains("stable") {
        bail!(
            "Failed to run rustfmt from toolchain 'stable'. \
             Please run `rustup component add rustfmt --toolchain stable` to install it.",
        )
    }
    Ok(())
}

fn reformat(text: &str) -> Result<String> {
    let sh = Shell::new()?;
    let _e = sh.push_env("RUSTUP_TOOLCHAIN", "stable");
    rustfmt()?;
    let stdout = cmd!(sh, "rustfmt --config fn_single_line=true")
        .stdin(text)
        .read()?;
    Ok(format!(
        "{}\n\n{}\n",
        "//! This is a generated file, please do not edit manually. Changes can be
//! made in codegeneration that lives in `xtask` top-level dir.",
        stdout
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
