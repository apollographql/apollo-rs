use std::collections::HashMap;

use crate::{
    diagnostics::{MissingIdent, SingleRootField, UniqueDefinition, UnsupportedOperation},
    values::OperationDefinition,
    ApolloDiagnostic, SourceDatabase,
};
// use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();
    // It is possible to have an unnamed (anonymous) operation definition only
    // if there is **one** operation definition.
    //
    // Return a Missing Indent error if there are multiple operations and one or
    // more are missing a name.
    let op_len = db.operations().len();
    if op_len > 1 {
        let missing_ident: Vec<ApolloDiagnostic> = db
            .operations()
            .iter()
            .filter_map(|op| {
                if op.name().is_none() {
                    let offset = op.ast_node(db).text_range().start().into();
                    let len: usize = op.ast_node(db).text_range().len().into();
                    return Some(ApolloDiagnostic::MissingIdent(MissingIdent {
                        src: db.input_string(()).to_string(),
                        definition: (offset, len).into(),
                        help: Some(format!("GraphQL allows a short-hand form for defining query operations when only that one operation exists in the document. There are {op_len} operations in this document."))
                    }));
                }
                None
            })
            .collect();
        errors.extend(missing_ident);
    }

    // Operation definitions must have unique names.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &OperationDefinition> = HashMap::new();
    for op in db.operations().iter() {
        if let Some(name) = op.name() {
            if let Some(prev_def) = seen.get(&name) {
                let prev_offset: usize = prev_def.ast_node(db).text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db).text_range().len().into();

                let current_offset: usize = op.ast_node(db).text_range().start().into();
                let current_node_len: usize = op.ast_node(db).text_range().len().into();
                errors.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: name.into(),
                    src: db.input_string(()).to_string(),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "{name} must only be defined once in this document."
                    )),
                }));
            } else {
                seen.insert(name, op);
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
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name()).collect();
                    let offset = op.ast_node(db).text_range().start().into();
                    let len: usize = op.ast_node(db).text_range().len().into();
                    Some(ApolloDiagnostic::SingleRootField(SingleRootField {
                        fields: fields.len(),
                        src: db.input_string(()).to_string(),
                        subscription: (offset, len).into(),
                        help: Some(format!(
                            "There are {} root fields: {}. This is not allowed.",
                            fields.len(),
                            field_names.join(", ")
                        )),
                    }))
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
                let op_offset: usize = op.ast_node(db).text_range().start().into();
                let op_len: usize = op.ast_node(db).text_range().len().into();

                if let Some(schema_node) = db.schema().ast_node(db) {
                    let schema_offset: usize = schema_node.text_range().start().into();
                    let schema_len: usize = schema_node.text_range().len().into();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Subscription".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Subscription".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: None,
                        help: Some(
                            "consider defining a `subscription` root operation type in your schema"
                                .into(),
                        ),
                    })
                }
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
                let op_offset: usize = op.ast_node(db).text_range().start().into();
                let op_len: usize = op.ast_node(db).text_range().len().into();

                if let Some(schema_node) = db.schema().ast_node(db) {
                    let schema_offset: usize = schema_node.text_range().start().into();
                    let schema_len: usize = schema_node.text_range().len().into();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Query".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Query".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: None,
                        help: Some(
                            "consider defining a `query` root operation type in your schema".into(),
                        ),
                    })
                }
            })
            .collect();
        errors.extend(unsupported_ops)
    }

    //   * mutation operation - mutation root operation
    if db.mutation_operations().len() >= 1 && db.schema().mutation(db).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = db
            .mutation_operations()
            .iter()
            .map(|op| {
                let op_offset: usize = op.ast_node(db).text_range().start().into();
                let op_len: usize = op.ast_node(db).text_range().len().into();

                if let Some(schema_node) = db.schema().ast_node(db) {
                    let schema_offset: usize = schema_node.text_range().start().into();
                    let schema_len: usize = schema_node.text_range().len().into();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Mutation".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Mutation".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.input_string(()).to_string(),
                        schema: None,
                        help: Some(
                            "consider defining a `mutation` root operation type in your schema"
                                .into(),
                        ),
                    })
                }
            })
            .collect();
        errors.extend(unsupported_ops)
    }

    errors
}

#[cfg(test)]
mod test {
    use crate::{diagnostics, ApolloCompiler};
    use miette::Report;

    #[test]
    fn it_fails_validation_with_missing_ident() {
        let input = r#"
query {
  cat {
    name
  }
}

query getPet {
  cat {
    owner {
      name
    }
  }
}

query getPet {
  cat {
    treat
  }
}

subscription sub {
  newMessage {
    body
    sender
  }
  disallowedSecondRootField
}

type Query {
  cat: Pet
}

type Subscription {
  newMessage: Result
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();

        let display_errors: Vec<Report> = errors
            .iter()
            .map(|err| match err {
                diagnostics::ApolloDiagnostic::MissingIdent(i) => Report::new(i.clone()),
                diagnostics::ApolloDiagnostic::UniqueDefinition(i) => Report::new(i.clone()),
                diagnostics::ApolloDiagnostic::SingleRootField(i) => Report::new(i.clone()),
                diagnostics::ApolloDiagnostic::UnsupportedOperation(i) => Report::new(i.clone()),
            })
            .collect();

        println!("{:?}", display_errors);
    }

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

type Query {
  cat: Pet
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

type Query {
  cat: Pet
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert!(errors.is_empty());
    }
}
