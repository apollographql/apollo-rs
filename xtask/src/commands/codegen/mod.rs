use ungrammar::{Grammar};
use anyhow::Result;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Codegen {}

impl Codegen {
    pub(crate) fn run(&self, _verbose: bool) -> Result<()> {
        let grammar_src = include_str!("../../../../graphql.ungram");
        let grammar: Grammar = grammar_src.parse().unwrap();
        dbg!(&grammar);
        Ok(())
    }
}
