use crate::{ApolloDiagnostic, ValidationDatabase};

pub fn check(_db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    todo!()
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_duplicate_argument_names() {
        let input = r#"
type Query {
  method(arg: Boolean, arg: Boolean): Int
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }
}
