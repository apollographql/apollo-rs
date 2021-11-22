use std::{env, fs, path::Path};

use apollo_parser::{ast, Parser};
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("{}", self.ty)]
#[diagnostic(code("apollo-parser parsing error."))]
struct ApolloParserError {
    ty: String,
    #[source_code]
    src: NamedSource,
    #[label("{}", self.ty)]
    span: SourceSpan,
}

fn parse_schema() -> Result<()> {
    let file = Path::new("crates/apollo-parser/examples/schema_with_errors.graphql");
    let src = fs::read_to_string(file).expect("Could not read schema file.");
    let file_name = file
        .file_name()
        .expect("Could not get file name.")
        .to_str()
        .expect("Could not get &str from file name.");

    let parser = Parser::new(&src);
    let ast = parser.parse();
    dbg!(&ast);

    for err in ast.errors() {
        Err(ApolloParserError {
            src: NamedSource::new(file_name, src.clone()),
            span: (err.index(), err.data().len()).into(),
            ty: err.message().into(),
        })?;
    }

    Ok(())
}

fn main() -> Result<()> {
    parse_schema()?;

    Ok(())
}
