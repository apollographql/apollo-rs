#[cfg(test)]
mod test {

    #[test]
    fn it_validates_undefined_variable_in_query() {
        let input = r#"
query ExampleQuery() {
  topProducts(first: $undefinedVariable) {
    name
  }
}"#;
    }
}
