use apollo_parser::Parser;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Unexpected Token")]
#[diagnostic(
    code(LexicalError::unexpected_token),
    url(docsrs),
    help("GraphQL grammar does not accept this token.")
)]
struct Error {
    #[source_code]
    src: NamedSource,
    #[label("This bit here")]
    span: SourceSpan,
}

fn this_fails() -> Result<()> {
    let src = r#"
    type Field {
        field: ö Cat
    }
    "#;
    let parser = Parser::new(src);
    let ast = parser.parse();

    for err in ast.errors() {
        Err(Error {
            src: NamedSource::new("schema.graphql", src),
            span: (err.index(), err.data().len()).into(),
        })?;
    }

    Ok(())
}

fn main() -> Result<()> {
    // kaboom~
    this_fails()?;

    Ok(())
}
