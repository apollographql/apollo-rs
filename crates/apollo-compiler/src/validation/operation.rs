use std::sync::Arc;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir, FileId, ValidationDatabase,
};

pub fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let operations = db.operations(file_id);
    for def in operations.iter() {
        diagnostics.extend(db.validate_directives(
            def.directives().to_vec(),
            def.operation_ty().into(),
            // assumption here is that operation definition's own directives can
            // use already defined variables
            def.variables.clone(),
        ));
        diagnostics.extend(db.validate_variable_definitions(def.variables.as_ref().clone()));

        // Validate the Selection Set recursively
        // Check that the root type exists
        if def.object_type(db.upcast()).is_some() {
            diagnostics.extend(
                db.validate_selection_set(def.selection_set().clone(), def.variables.clone()),
            );
        }
        diagnostics.extend(db.validate_unused_variable(def.clone()));
    }

    let subscription_operations = db.upcast().subscription_operations(file_id);
    let query_operations = db.upcast().query_operations(file_id);
    let mutation_operations = db.upcast().mutation_operations(file_id);

    diagnostics.extend(db.validate_subscription_operations(subscription_operations));
    diagnostics.extend(db.validate_query_operations(query_operations));
    diagnostics.extend(db.validate_mutation_operations(mutation_operations));

    // It is possible to have an unnamed (anonymous) operation definition only
    // if there is **one** operation definition.
    //
    // Return a Missing Indent error if there are multiple operations and one or
    // more are missing a name.
    let op_len = operations.len();
    if op_len > 1 {
        let missing_ident: Vec<ApolloDiagnostic> = operations
            .iter()
            .filter_map(|op| {
                if op.name().is_none() {
                    return Some(
                        ApolloDiagnostic::new(db, op.loc().into(), DiagnosticData::MissingIdent)
                            .label(Label::new(op.loc(), "provide a name for this definition"))
                            .help(format!("GraphQL allows a short-hand form for defining query operations when only that one operation exists in the document. There are {op_len} operations in this document.")),
                    );
                }
                None
            })
            .collect();
        diagnostics.extend(missing_ident);
    }

    diagnostics
}

pub fn validate_subscription_operations(
    db: &dyn ValidationDatabase,
    subscriptions: Arc<Vec<Arc<hir::OperationDefinition>>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Subscription fields must not have an introspection field in the selection set.
    //
    // Return an Introspection Field error in case of an introspection field existing as the root field in the set.
    for op in subscriptions.iter() {
        for selection in op.selection_set().selection() {
            if let hir::Selection::Field(field) = selection {
                if field.is_introspection() {
                    let field_name = field.name();
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            field.loc().into(),
                            DiagnosticData::IntrospectionField {
                                field: field_name.into(),
                            },
                        )
                        .label(Label::new(
                            field.loc(),
                            format!("{field_name} is an introspection field"),
                        )),
                    );
                }
            }
        }
    }

    // A Subscription operation definition can only have **one** root level
    // field.
    if !subscriptions.is_empty() {
        let single_root_field: Vec<ApolloDiagnostic> = subscriptions
            .iter()
            .filter_map(|op| {
                let mut fields = op.fields(db.upcast()).as_ref().clone();
                fields.extend(op.fields_in_inline_fragments(db.upcast()).as_ref().clone());
                fields.extend(op.fields_in_fragment_spread(db.upcast()).as_ref().clone());
                if fields.len() > 1 {
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name()).collect();
                    Some(
                        ApolloDiagnostic::new(
                            db,
                            op.loc().into(),
                            DiagnosticData::SingleRootField {
                                fields: fields.len(),
                                subscription: op.loc().into(),
                            },
                        )
                        .label(Label::new(
                            op.loc(),
                            format!("subscription with {} root fields", fields.len()),
                        ))
                        .help(format!(
                            "There are {} root fields: {}. This is not allowed.",
                            fields.len(),
                            field_names.join(", ")
                        )),
                    )
                } else {
                    None
                }
            })
            .collect();
        diagnostics.extend(single_root_field);
    }

    // All query, subscription and mutation operations must be against legally
    // defined schema root operation types.
    //
    //   * subscription operation - subscription root operation
    if !subscriptions.is_empty() && db.schema().subscription().is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = subscriptions
            .iter()
            .map(|op| {
                let diagnostic = ApolloDiagnostic::new(db, op.loc().into(), DiagnosticData::UnsupportedOperation { ty: "subscription" })
                    .label(Label::new(op.loc(), "Subscription operation is not defined in the schema and is therefore not supported"));
                if let Some(schema_loc) = db.schema().loc() {
                    diagnostic.label(Label::new(schema_loc, "Consider defining a `subscription` root operation type here"))
                } else {
                    diagnostic.help("consider defining a `subscription` root operation type in your schema")
                }
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    diagnostics
}

pub fn validate_query_operations(
    db: &dyn ValidationDatabase,
    queries: Arc<Vec<Arc<hir::OperationDefinition>>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // All query, subscription and mutation operations must be against legally
    // defined schema root operation types.
    //
    //   * query operation - query root operation
    if !queries.is_empty() && db.schema().query().is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> =
            queries
                .iter()
                .filter_map(|op| {
                    let has_cycles = op
                        .fragment_references(db.upcast())
                        .iter()
                        .any(|fragment| !db.validate_fragment_cycles(fragment.clone()).is_empty());

                    if !has_cycles && !op.is_introspection(db.upcast()) {
                        let diagnostic = ApolloDiagnostic::new(
                            db,
                            op.loc().into(),
                            DiagnosticData::UnsupportedOperation { ty: "query" },
                        )
                        .label(Label::new(
                            op.loc(),
                            "Query operation is not defined in the schema and is therefore not supported",
                        ));
                        if let Some(schema_loc) = db.schema().loc() {
                            Some(diagnostic.label(Label::new(
                                schema_loc,
                                "Consider defining a `query` root operation type here",
                            )))
                        } else {
                            Some(diagnostic.help(
                                "consider defining a `query` root operation type in your schema",
                            ))
                        }
                    } else {
                        None
                    }
                })
                .collect();
        diagnostics.extend(unsupported_ops)
    }

    diagnostics
}

pub fn validate_mutation_operations(
    db: &dyn ValidationDatabase,
    mutations: Arc<Vec<Arc<hir::OperationDefinition>>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // All query, subscription and mutation operations must be against legally
    // defined schema root operation types.
    //
    //   * mutation operation - mutation root operation
    if !mutations.is_empty() && db.schema().mutation().is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = mutations
            .iter()
            .map(|op| {
                let diagnostic = ApolloDiagnostic::new(db, op.loc().into(), DiagnosticData::UnsupportedOperation { ty: "mutation" })
                    .label(Label::new(op.loc(), "Mutation operation is not defined in the schema and is therefore not supported"));
                if let Some(schema_loc) = db.schema().loc() {
                    diagnostic.label(Label::new(schema_loc, "Consider defining a `mutation` root operation type here"))
                } else {
                    diagnostic.help("consider defining a `mutation` root operation type in your schema")
                }
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

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
    name
  }
}

query getOtherPet {
  cat {
    nickname
  }
}

type Query {
  cat: Cat
}

type Cat {
  name: String
  nickname: String
  meowVolume: Int
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}")
        }
        assert_eq!(diagnostics.len(), 1)
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
    name
    nickname
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
  nickname: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}")
        }
        assert_eq!(diagnostics.len(), 1)
    }

    #[test]
    fn it_validates_unique_operation_names() {
        let input = r#"
query getCatName {
  cat {
    name
  }
}

query getPetNickname {
  cat {
    nickname
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
  nickname: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}")
        }
        assert!(diagnostics.is_empty());
    }

    #[test]
    fn it_raises_an_error_for_illegal_operations() {
        let input = r#"
subscription sub {
  newMessage {
    body
    sender
  }
}

type Query {
  cat: Pet
}

union CatOrDog = Cat | Dog

interface Pet {
  name: String
}

type Dog implements Pet {
  name: String
  nickname: String
  barkVolume: Int
}

type Cat implements Pet {
  name: String
  nickname: String
  meowVolume: Int
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}")
        }

        assert_eq!(diagnostics.len(), 1)
    }

    #[test]
    fn it_validates_fields_in_operations() {
        let input = r#"
query getProduct {
  name
  noName
  topProducts {
    inStock
    price
  }
}

type Query {
  name: String
  topProducts: Product
}

type Product {
  inStock: Boolean
  name: String
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 2)
    }
}
