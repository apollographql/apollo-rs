fn main() {
    let ctx = ApolloCompiler::new(input);
    // let errors = ctx.validate();

    let operation_names: Vec<String> = ctx.operations().iter().filter_map(|op| op.name()).collect();
    assert_eq!(["ExampleQuery"], operation_names.as_slice());
    let fragments: Vec<String> = ctx.fragments().iter().map(|frag| frag.name()).collect();
    assert_eq!(["vipCustomer"], fragments.as_slice());

    let operation_variables: Vec<String> = ctx
        .operations()
        .find("ExampleQuery")?
        .variables()?
        .find("defaultVariable")?;
    assert_eq!(["definedVariable"], operation_variables.as_slice());
    // let operation_variables = ctx.operations().find("ExampleQuery").variables().find("definedVariable").ty();
    let fragment_fields = ctx.fragments().find("vipCustomer").unwrap();
    dbg!(fragment_fields);
}
