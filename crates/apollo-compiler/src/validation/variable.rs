use std::collections::HashSet;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationSet,
    FileId, ValidationDatabase,
};

pub fn validate_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: Vec<hir::VariableDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for variable in variables.iter() {
        diagnostics.extend(db.validate_directives(
            variable.directives().to_vec(),
            hir::DirectiveLocation::VariableDefinition,
        ));
    }

    diagnostics
}

// check in scope
// check in use
// compare the two
pub fn validate_unused_variables(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    db.operations(file_id)
        .iter()
        .flat_map(|op| {
            let defined_vars: HashSet<ValidationSet> = op
                .variables()
                .iter()
                .map(|var| ValidationSet {
                    name: var.name().into(),
                    loc: var.loc(),
                })
                .collect();
            let used_vars: HashSet<ValidationSet> = op
                .selection_set
                .clone()
                .selection()
                .iter()
                .flat_map(|sel| {
                    let vars: HashSet<ValidationSet> = sel
                        .variables(db.upcast())
                        .iter()
                        .map(|var| ValidationSet {
                            name: var.name().into(),
                            loc: var.loc(),
                        })
                        .collect();
                    vars
                })
                .collect();
            let undefined_vars = used_vars.difference(&defined_vars);
            let mut diagnostics: Vec<ApolloDiagnostic> = undefined_vars
                .map(|undefined_var| {
                    ApolloDiagnostic::new(
                        db,
                        undefined_var.loc.into(),
                        DiagnosticData::UndefinedDefinition {
                            name: undefined_var.name.clone(),
                        },
                    )
                    .label(Label::new(undefined_var.loc, "not found in this scope"))
                })
                .collect();

            let unused_vars = defined_vars.difference(&used_vars);
            let warnings = unused_vars.map(|unused_var| {
                ApolloDiagnostic::new(
                    db,
                    unused_var.loc.into(),
                    DiagnosticData::UnusedVariable {
                        name: unused_var.name.clone(),
                    },
                )
                .label(Label::new(unused_var.loc, "this variable is never used"))
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

  ... on User {
    id
    name
    profilePic(size: $dimensions)
    status
  }

}

type Query {
  topProducts: Products
}

type Products {
  weight: Float
  size: Int
  name: String
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "document.graphql");

        let diagnostics = compiler.validate();

        for error in &diagnostics {
            println!("{error}")
        }

        assert_eq!(diagnostics.len(), 2);
    }

    #[test]
    fn it_raises_unused_variable_in_query_error() {
        let input = r#"
query ExampleQuery($unusedVariable: Int) {
  topProducts {
    name
  }
  ... multipleSubscriptions
}

type Query {
  topProducts(first: Int): Product,
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "document.graphql");

        let diagnostics = compiler.validate();

        for error in diagnostics {
            println!("{error}")
        }
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

fragment fragmentOne on Query {
    profilePic(size: $dimensions)
}

type Query {
  topProducts: Product
  profilePic(size: Int): String
}

type Product {
  name: String
  price(setPrice: Int): Int
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "document.graphql");

        let diagnostics = compiler.validate();

        for error in &diagnostics {
            println!("{error}")
        }

        assert_eq!(diagnostics.len(), 2);
    }
}
