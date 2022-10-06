#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_duplicate_field_argument_names() {
        let input = r#"
interface Duplicate {
  duplicate(arg: Boolean, arg: Boolean): Int
}

type Query implements Duplicate {
  single(arg: Boolean): Int
  duplicate(arg: Boolean, arg: Boolean): Int
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn it_fails_validation_with_duplicate_directive_argument_names() {
        let input = r#"directive @example(arg: Boolean, arg: Boolean) on FIELD"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_duplicate_field_arguments() {
        let input = r#"
type Query {
  single(arg: Boolean): Int
}
query GetDuplicate {
  single(arg: true, arg: false)
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_duplicate_directive_arguments() {
        let input = r#"
type X @deprecated(reason: "as a test", reason: "just for fun") {}
type Query {
  something: X @skip(if: false, if: true)
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 2);
    }
}
