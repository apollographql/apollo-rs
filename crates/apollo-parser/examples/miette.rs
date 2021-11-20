use apollo_parser::Parser;
use miette::{Diagnostic, NamedSource, Result, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("{}", self.ty)]
#[diagnostic(code("parsing error found"))]
struct Error {
    ty: String,
    #[source_code]
    src: NamedSource,
    #[label("{}", self.ty)]
    span: SourceSpan,
}

fn this_fails() -> Result<()> {
    let src = "
    type Field {
        field: ö Cat
    }
    type Field {
        field: ö Cat
    }
    ";
    let parser = Parser::new(src);
    let ast = parser.parse();

    for err in ast.errors() {
        Err(Error {
            src: NamedSource::new("schema.graphql", src),
            span: (err.index(), err.data().len()).into(),
            ty: err.message().into(),
        })?;
    }

    Ok(())
}

fn main() -> Result<()> {
    // kaboom~
    this_fails()?;

    Ok(())
}
