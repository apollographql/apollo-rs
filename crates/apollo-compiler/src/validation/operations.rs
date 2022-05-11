use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();
    // It is possible to have an unnamed (anonymous) operation definition only
    // if there is **one** operation definition.
    //
    // Return a Missing Indent error if there are multiple operations and one or
    // more are missing a name.
    if db.operations().len() > 1 {
        let missing_ident: Vec<ApolloDiagnostic> = db
            .operations()
            .iter()
            .filter_map(|op| {
                if op.name().is_none() {
                    return Some(ApolloDiagnostic::Error(ErrorDiagnostic::MissingIdent(
                        "Missing operation name".into(),
                    )));
                }
                None
            })
            .collect();
        errors.extend(missing_ident);
    }

    // Operation definitions must have unique names.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen = HashSet::new();
    for op in db.operations().iter() {
        if let Some(name) = op.name() {
            if seen.contains(&name) {
                errors.push(ApolloDiagnostic::Error(
                    ErrorDiagnostic::UniqueOperationDefinition {
                        message: "Operation Definitions must have unique names".into(),
                        operation: name.to_string(),
                    },
                ));
            } else {
                seen.insert(name);
            }
        }
    }

    // A Subscription operation definition can only have **one** root level
    // field.
    if db.subscription_operations().len() >= 1 {
        let single_root_field: Vec<ApolloDiagnostic> = db
            .subscription_operations()
            .iter()
            .filter_map(|op| {
                let mut fields = op.fields(db).as_ref().clone();
                fields.extend(op.fields_in_inline_fragments(db).as_ref().clone());
                fields.extend(op.fields_in_fragment_spread(db).as_ref().clone());
                if fields.len() > 1 {
                    Some(ApolloDiagnostic::Error(ErrorDiagnostic::SingleRootField(
                        "Subscription operations can only have one root field {}".into(),
                    )))
                } else {
                    None
                }
            })
            .collect();
        errors.extend(single_root_field);
    }

    // All query, subscription and mutation operations must be against legally
    // defined schema root operation types.
    //
    //   * subscription operation - subscription root operation
    if db.subscription_operations().len() >= 1 && db.schema().subscription(db).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = db
            .subscription_operations()
            .iter()
            .map(|op| {
                ApolloDiagnostic::Error(ErrorDiagnostic::UnsupportedOperation {
                    message: "Subscription operation not supported by the schema".into(),
                    operation: op.name().map(|s| s.to_string()),
                })
            })
            .collect();
        errors.extend(unsupported_ops)
    }
    //
    //   * query operation - query root operation
    if db.query_operations().len() >= 1 && db.schema().query(db).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = db
            .query_operations()
            .iter()
            .map(|op| {
                ApolloDiagnostic::Error(ErrorDiagnostic::UnsupportedOperation {
                    message: "Query operation not supported by the schema".into(),
                    operation: op.name().map(|s| s.to_string()),
                })
            })
            .collect();
        errors.extend(unsupported_ops)
    }
    //
    //   * mutation operation - mutation root operation
    if db.mutation_operations().len() >= 1 && db.schema().mutation(db).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = db
            .mutation_operations()
            .iter()
            .map(|op| {
                ApolloDiagnostic::Error(ErrorDiagnostic::UnsupportedOperation {
                    message: "Mutation operation not supported by the schema".into(),
                    operation: op.name().map(|s| s.to_string()),
                })
            })
            .collect();
        errors.extend(unsupported_ops)
    }

    errors
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_duplicate_operation_names() {
        let input = r#"
query getName {
  cat {
    name
  }
}

query getName {
  cat {
    owner {
      name
    }
  }
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn it_validates_unique_operation_names() {
        let input = r#"
query getCatName {
  cat {
    name
  }
}

query getOwnerName {
  cat {
    owner {
      name
    }
  }
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert!(errors.is_empty());
    }
}
