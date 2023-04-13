use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, VariableDefinition},
    validation::ValidationSet,
    ValidationDatabase,
};

pub fn validate_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: Vec<hir::VariableDefinition>,
    parent_op: Option<hir::Name>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen: HashMap<&str, &VariableDefinition> = HashMap::new();
    for variable in variables.iter() {
        diagnostics.extend(db.validate_directives(
            variable.directives().to_vec(),
            hir::DirectiveLocation::VariableDefinition,
            parent_op.clone(),
        ));

        let ty = variable.ty();
        if !ty.is_input_type(db.upcast()) {
            let type_def = ty.type_def(db.upcast());
            if let Some(type_def) = type_def {
                let ty_name = type_def.kind();
                diagnostics.push(
                    ApolloDiagnostic::new(db, variable.loc().into(), DiagnosticData::InputType {
                        name: variable.name().into(),
                        ty: ty_name,
                    })
                    .label(Label::new(ty.loc().unwrap(), format!("this is of `{ty_name}` type")))
                    .help("objects, unions, and interfaces cannot be used because variables can only be of input type"),
                );
            } else {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        variable.loc.into(),
                        DiagnosticData::UndefinedDefinition { name: ty.name() },
                    )
                    .label(Label::new(variable.loc, "not found in the type system")),
                )
            }
        }

        // Variable definitions must be unique.
        //
        // Return a Unique Definition error in case of a duplicate value.
        let name = variable.name();
        if let Some(prev_def) = seen.get(&name) {
            let original_definition = prev_def.loc();
            let redefined_definition = variable.loc();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    redefined_definition.into(),
                    DiagnosticData::UniqueDefinition {
                        ty: "enum",
                        name: name.into(),
                        original_definition: original_definition.into(),
                        redefined_definition: redefined_definition.into(),
                    },
                )
                .labels([
                    Label::new(
                        original_definition,
                        format!("previous definition of `{name}` here"),
                    ),
                    Label::new(redefined_definition, format!("`{name}` redefined here")),
                ])
                .help(format!("{name} must only be defined once in this enum.")),
            );
        } else {
            seen.insert(name, variable);
        }
    }

    diagnostics
}

pub fn validate_unused_variables(
    db: &dyn ValidationDatabase,
    op: Arc<hir::OperationDefinition>,
) -> Vec<ApolloDiagnostic> {
    let defined_vars: HashSet<ValidationSet> = op
        .variables()
        .iter()
        .map(|var| ValidationSet {
            name: var.name().into(),
            loc: Some(var.loc()),
        })
        .collect();
    let used_vars: HashSet<ValidationSet> = op
        .selection_set
        .variables(db.upcast())
        .into_iter()
        .map(|var| ValidationSet {
            name: var.name().into(),
            loc: Some(var.loc()),
        })
        .collect();
    let undefined_vars = used_vars.difference(&defined_vars);
    let mut diagnostics: Vec<ApolloDiagnostic> = undefined_vars
        .map(|undefined_var| {
            // undefined var location is always Some
            let loc = undefined_var.loc.expect("missing location information");
            ApolloDiagnostic::new(
                db,
                loc.into(),
                DiagnosticData::UndefinedDefinition {
                    name: undefined_var.name.clone(),
                },
            )
            .label(Label::new(loc, "not found in this scope"))
        })
        .collect();

    let unused_vars = defined_vars.difference(&used_vars);
    diagnostics.extend(unused_vars.map(|unused_var| {
        // unused var location is always Some
        let loc = unused_var.loc.expect("missing location information");
        ApolloDiagnostic::new(
            db,
            loc.into(),
            DiagnosticData::UnusedVariable {
                name: unused_var.name.clone(),
            },
        )
        .label(Label::new(loc, "this variable is never used"))
    }));

    diagnostics
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
  topProducts(first: Int): Products
  me: User
}

type User {
    id: ID
    name: String
    status(membership: String): String
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
  me: User
}

type User {
    id: ID
    name: String
    status(membership: String): String
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
