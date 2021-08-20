/// A Util to compare ASTs.
#[cfg(test)]
pub(crate) fn check_ast(input: &str, expected: &str) {
    use pretty_assertions::assert_eq;
    use unindent::Unindent;

    let parser = crate::Parser::new(input);
    let ast = parser.parse();

    let actual = format!("{:?}", ast);
    let actual = actual.trim();
    let expected = expected.unindent();
    let fmt_expected = expected.trim();

    if actual != fmt_expected {
        println!("\nACTUAL:\n\n{}", actual);
        println!("EXPECTED:\n\n{}", fmt_expected);
        assert_eq!(actual, fmt_expected);
    }
}
