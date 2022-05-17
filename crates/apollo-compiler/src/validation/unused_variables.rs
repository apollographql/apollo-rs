use std::collections::HashSet;

use crate::{
    diagnostics::{ApolloDiagnostic, ErrorDiagnostic, WarningDiagnostic},
    SourceDatabase,
};

// check in scope
// check in use
// compare the two
pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    db.operations()
        .iter()
        .flat_map(|op| {
            let defined_vars: HashSet<String> =
                op.variables().iter().map(|var| var.name.clone()).collect();
            let used_vars: HashSet<String> = op
                .selection_set
                .clone()
                .iter()
                .flat_map(|sel| sel.variables(db))
                .collect();
            let undefined_vars = used_vars.difference(&defined_vars);
            let mut diagnostics: Vec<ApolloDiagnostic> = undefined_vars
                .map(|undefined_var| {
                    ApolloDiagnostic::Error(ErrorDiagnostic::UndefinedVariable {
                        message: "Variable undefined".into(),
                        variable: undefined_var.into(),
                    })
                })
                .collect();

            let unused_vars = defined_vars.difference(&used_vars);
            let warnings = unused_vars.map(|unused_var| {
                ApolloDiagnostic::Warning(WarningDiagnostic::UnusedVariable {
                    message: "unused variable".into(),
                    variable: unused_var.into(),
                })
            });

            diagnostics.extend(warnings);
            diagnostics
        })
        .collect()
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_raises_undefined_variable_in_query_error() {
        let input = r#"
query ExampleQuery {
  topProducts(first: $undefinedVariable) {
    name
  }

  ... VipCustomer on User {
    id
    name
    profilePic(size: $dimensions)
    status
  }

}

type Query {
  topProducts: Products 
}
"#;

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn it_raises_undefined_variable_in_query_in_fragments_error() {
        let input = r#"
query ExampleQuery {
  topProducts {
    name
  }

  ... on User {
    id
    name
    status(membership: $goldStatus)
  }

  ... fragmentOne
}

fragment fragmentOne on User {
    profilePic(size: $dimensions)
}

type Query {
  topProducts: Products 
}
"#;

        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();

        assert_eq!(diagnostics.len(), 2);
    }
}
