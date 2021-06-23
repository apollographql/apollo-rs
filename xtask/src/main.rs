mod commands;

use anyhow::Result;
use structopt::StructOpt;

fn main() -> Result<()> {
    let app = Xtask::from_args();
    app.run()
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "xtask",
    about = "apollo-rs development workflows"
)]
struct Xtask {
    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(long = "verbose", short = "v", global = true)]
    verbose: bool
}

#[derive(Debug, StructOpt)]
pub enum Command {
    Codegen(commands::Codegen),
}

impl Xtask {
    pub fn run(&self) -> Result<()> {
        match &self.command {
            Command::Codegen(command) => command.run(self.verbose),
        }?;

        Ok(())
    }
}