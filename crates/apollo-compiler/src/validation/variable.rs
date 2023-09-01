use crate::Arc;
use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, VariableDefinition},
    validation::ValidationSet,
    ValidationDatabase,
};
use std::collections::{HashMap, HashSet};

pub fn validate_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: Arc<Vec<hir::VariableDefinition>>,
    has_schema: bool,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let mut seen: HashMap<&str, &VariableDefinition> = HashMap::new();
    for variable in variables.iter() {
        diagnostics.extend(db.validate_directives(
            variable.directives().to_vec(),
            hir::DirectiveLocation::VariableDefinition,
            // let's assume that variable definitions cannot reference other
            // variables and provide them as arguments to directives
            Arc::new(Vec::new()),
        ));

        if has_schema {
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
    let mut diagnostics = Vec::new();

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

pub fn validate_variable_usage(
    db: &dyn ValidationDatabase,
    var_usage: hir::InputValueDefinition,
    var_defs: Arc<Vec<hir::VariableDefinition>>,
    arg: hir::Argument,
) -> Result<(), ApolloDiagnostic> {
    if let hir::Value::Variable(var) = arg.value() {
        // Let var_def be the VariableDefinition named
        // variable_name defined within operation.
        let var_def = var_defs.iter().find(|v| v.name() == var.name());
        if let Some(var_def) = var_def {
            let is_allowed = is_variable_usage_allowed(var_def, &var_usage);
            if !is_allowed {
                return Err(ApolloDiagnostic::new(
                    db,
                    arg.loc.into(),
                    DiagnosticData::DisallowedVariableUsage {
                        var_name: var_def.name().into(),
                        arg_name: arg.name().into(),
                    },
                )
                .labels([
                    Label::new(
                        var_def.loc,
                        format!(
                            "variable `{}` of type `{}` is declared here",
                            var_def.name(),
                            var_def.ty(),
                        ),
                    ),
                    Label::new(
                        arg.loc,
                        format!(
                            "argument `{}` of type `{}` is declared here",
                            arg.name(),
                            var_usage.ty()
                        ),
                    ),
                ]));
            }
        } else {
            return Err(ApolloDiagnostic::new(
                db,
                arg.loc().into(),
                DiagnosticData::UndefinedVariable {
                    name: var.name().into(),
                },
            )
            .label(Label::new(var.loc(), "not found in this scope")));
        }
    }
    // It's super confusing to produce a diagnostic here if either the
    // location_ty or variable_ty is missing, so just return Ok(());
    Ok(())
}

fn is_variable_usage_allowed(
    var_def: &hir::VariableDefinition,
    var_usage: &hir::InputValueDefinition,
) -> bool {
    // 1. Let variable_ty be the expected type of variable_def.
    let variable_ty = var_def.ty();
    // 2. Let location_ty be the expected type of the Argument,
    // ObjectField, or ListValue entry where variableUsage is
    // located.
    let location_ty = var_usage.ty();
    // 3. if location_ty is a non-null type AND variable_ty is
    // NOT a non-null type:
    if location_ty.is_non_null() && !variable_ty.is_non_null() {
        // 3.a. let hasNonNullVariableDefaultValue be true
        // if a default value exists for variableDefinition
        // and is not the value null.
        let has_non_null_default_value =
            !(matches!(var_def.default_value(), Some(&hir::Value::Null { .. })));
        // 3.b. Let hasLocationDefaultValue be true if a default
        // value exists for the Argument or ObjectField where
        // variableUsage is located.
        let has_location_default_value =
            !(matches!(var_usage.default_value(), Some(&hir::Value::Null { .. })));
        // 3.c. If hasNonNullVariableDefaultValue is NOT true
        // AND hasLocationDefaultValue is NOT true, return
        // false.
        if !has_non_null_default_value && !has_location_default_value {
            return false;
        }

        // 3.d. Let nullable_location_ty be the unwrapped
        // nullable type of location_ty.
        match location_ty {
            hir::Type::NonNull { ty: loc_ty, .. } => {
                // 3.e. Return AreTypesCompatible(variableType, nullableLocationType).
                return are_types_compatible(variable_ty, loc_ty);
            }
            hir::Type::List { ty: loc_ty, .. } => return are_types_compatible(variable_ty, loc_ty),
            hir::Type::Named { .. } => return are_types_compatible(variable_ty, location_ty),
        }
    }

    are_types_compatible(variable_ty, location_ty)
}

fn are_types_compatible(variable_ty: &hir::Type, location_ty: &hir::Type) -> bool {
    match (location_ty, variable_ty) {
        // 1. If location_ty is a non-null type:
        // 1.a. If variable_ty is NOT a non-null type, return false.
        (hir::Type::NonNull { .. }, hir::Type::Named { .. } | hir::Type::List { .. }) => false,
        // 1.b. Let nullable_location_ty be the unwrapped nullable type of location_ty.
        // 1.c. Let nullable_variable_type be the unwrapped nullable type of variable_ty.
        // 1.d. Return AreTypesCompatible(nullable_variable_ty, nullable_location_ty).
        (
            hir::Type::NonNull {
                ty: nullable_location_ty,
                ..
            },
            hir::Type::NonNull {
                ty: nullable_variable_ty,
                ..
            },
        ) => are_types_compatible(nullable_variable_ty, nullable_location_ty),
        // 2. Otherwise, if variable_ty is a non-null type:
        // 2.a. Let nullable_variable_ty be the nullable type of variable_ty.
        // 2.b. Return are_types_compatible(nullable_variable_ty, location_ty).
        (
            _,
            hir::Type::NonNull {
                ty: nullable_variable_ty,
                ..
            },
        ) => are_types_compatible(nullable_variable_ty, location_ty),
        // 3.Otherwise, if location_ty is a list type:
        // 3.a. If variable_ty is NOT a list type, return false.
        (hir::Type::List { .. }, hir::Type::Named { .. }) => false,
        // 3.b.Let item_location_ty be the unwrapped item type of location_ty.
        // 3.c. Let item_variable_ty be the unwrapped item type of variable_ty.
        // 3.d. Return AreTypesCompatible(item_variable_ty, item_location_ty).
        (
            hir::Type::List {
                ty: item_location_ty,
                ..
            },
            hir::Type::List {
                ty: item_variable_ty,
                ..
            },
        ) => are_types_compatible(item_variable_ty, item_location_ty),
        // 4. Otherwise, if variable_ty is a list type, return false.
        (hir::Type::Named { .. }, hir::Type::List { .. }) => false,
        // 5. Return true if variable_ty and location_ty are identical, otherwise false.
        (hir::Type::Named { name: loc_name, .. }, hir::Type::Named { name: var_name, .. }) => {
            var_name == loc_name
        }
    }
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

  me {
    ... on User {
      id
      name
      profilePic(size: $dimensions)
      status
    }
  }
}

type Query {
  topProducts(first: Int): Products
  me: User
}

type User {
    id: ID
    name: String
    profilePic(size: Int): String
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

  me {
    ... on User {
      id
      name
      status(membership: $goldStatus)
    }
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
