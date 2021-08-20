/// A Util to compare ASTs.
#[cfg(test)]
pub(crate) fn check_ast(input: &str, expected: &str) {
    use pretty_assertions::assert_eq;
    let parser = crate::Parser::new(input);
    let ast = parser.parse();
    let expected = expected.trim();
    let actual = format!("{:?}", ast);

    // write!(std::io::stdout(), "{:?}", ast).unwrap();

    let actual = actual.trim();
    if actual != expected {
        println!("\nACTUAL:\n{}", actual);
        println!("EXPECTED:\n{}", expected);
        assert_eq!(actual, expected);
    }
}
