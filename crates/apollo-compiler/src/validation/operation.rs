use std::collections::HashMap;

use crate::{
    diagnostics::{Diagnostic2, DiagnosticData, Label},
    hir::{OperationDefinition, Selection},
    ApolloDiagnostic, FileId, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase, file_id: FileId) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let operations = db.operations(file_id);
    let subscription_operations = db.upcast().subscription_operations(file_id);
    let query_operations = db.upcast().query_operations(file_id);
    let mutation_operations = db.upcast().mutation_operations(file_id);

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
                    return Some(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(db, op.loc().into(), DiagnosticData::MissingIdent)
                            .label(Label::new(op.loc(), "provide a name for this definition"))
                            .help(format!("GraphQL allows a short-hand form for defining query operations when only that one operation exists in the document. There are {op_len} operations in this document.")),
                    ));
                }
                None
            })
            .collect();
        diagnostics.extend(missing_ident);
    }

    // Operation definitions must have unique names.
    //
    // Return a Unique Operation Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &OperationDefinition> = HashMap::new();
    for op in operations.iter() {
        if let Some(name) = op.name() {
            if let Some(prev_def) = seen.get(&name) {
                let original_definition = prev_def
                    .name_src()
                    .and_then(|name| name.loc())
                    .unwrap_or(prev_def.loc());
                let redefined_definition = op
                    .name_src()
                    .and_then(|name| name.loc())
                    .unwrap_or(op.loc());
                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        db,
                        redefined_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "operation",
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
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                ));
            } else {
                seen.insert(name, op);
            }
        }
    }

    // A Subscription operation definition can only have **one** root level
    // field.
    if subscription_operations.len() >= 1 {
        let single_root_field: Vec<ApolloDiagnostic> = subscription_operations
            .iter()
            .filter_map(|op| {
                let mut fields = op.fields(db.upcast()).as_ref().clone();
                fields.extend(op.fields_in_inline_fragments(db.upcast()).as_ref().clone());
                fields.extend(op.fields_in_fragment_spread(db.upcast()).as_ref().clone());
                if fields.len() > 1 {
                    let field_names: Vec<&str> = fields.iter().map(|f| f.name()).collect();
                    Some(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
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
                    ))
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
    if subscription_operations.len() >= 1 && db.schema().subscription(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = subscription_operations
            .iter()
            .map(|op| {
                let diagnostic = Diagnostic2::new(db, op.loc().into(), DiagnosticData::UnsupportedOperation { ty: "subscription" })
                    .label(Label::new(op.loc(), "Subscription operation is not defined in the schema and is therefore not supported"));
                let diagnostic = if let Some(schema_loc) = db.schema().loc() {
                    diagnostic.label(Label::new(schema_loc, "Consider defining a `subscription` root operation type here"))
                } else {
                    diagnostic.help("consider defining a `subscription` root operation type in your schema")
                };
                ApolloDiagnostic::Diagnostic2(diagnostic)
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    //   * query operation - query root operation
    if query_operations.len() >= 1 && db.schema().query(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = query_operations
            .iter()
            .map(|op| {
                let diagnostic = Diagnostic2::new(
                    db,
                    op.loc().into(),
                    DiagnosticData::UnsupportedOperation { ty: "query" },
                )
                .label(Label::new(
                    op.loc(),
                    "Query operation is not defined in the schema and is therefore not supported",
                ));
                let diagnostic = if let Some(schema_loc) = db.schema().loc() {
                    diagnostic.label(Label::new(
                        schema_loc,
                        "Consider defining a `query` root operation type here",
                    ))
                } else {
                    diagnostic
                        .help("consider defining a `query` root operation type in your schema")
                };
                ApolloDiagnostic::Diagnostic2(diagnostic)
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    //   * mutation operation - mutation root operation
    if mutation_operations.len() >= 1 && db.schema().mutation(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = mutation_operations
            .iter()
            .map(|op| {
                let diagnostic = Diagnostic2::new(db, op.loc().into(), DiagnosticData::UnsupportedOperation { ty: "mutation" })
                    .label(Label::new(op.loc(), "Mutation operation is not defined in the schema and is therefore not supported"));
                let diagnostic = if let Some(schema_loc) = db.schema().loc() {
                    diagnostic.label(Label::new(schema_loc, "Consider defining a `mutation` root operation type here"))
                } else {
                    diagnostic.help("consider defining a `mutation` root operation type in your schema")
                };
                ApolloDiagnostic::Diagnostic2(diagnostic)
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    // Fields must exist on the type being queried.
    for op in operations.iter() {
        for selection in op.selection_set().selection() {
            let obj_name = op.object_type(db.upcast()).map(|obj| obj.name().to_owned());
            if let Selection::Field(field) = selection {
                if field.ty(db.upcast()).is_none() {
                    let field_name = field.name();
                    let help = if let Some(obj_type) = obj_name {
                        format!("`{field_name}` is not defined on `{obj_type}` type")
                    } else {
                        format!(
                            "`{}` is not defined on the current {} root operation type.",
                            field_name,
                            op.operation_ty()
                        )
                    };
                    diagnostics.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
                            db,
                            field.loc().into(),
                            DiagnosticData::UndefinedField {
                                field: field_name.into(),
                            },
                        )
                        .label(Label::new(
                            field.loc(),
                            format!("`{field_name} field is not in scope"),
                        ))
                        .help(help),
                    ))
                }
            }
        }
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

union CatOrDog = Cat | Dog
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 5)
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
            println!("{}", diagnostic)
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
            println!("{}", diagnostic)
        }

        assert_eq!(diagnostics.len(), 2)
    }

    #[test]
    fn it_validates_fields_in_operations() {
        let input = r#"
query getProduct {
  size
  weight
}

type Query {
  name: String
  topProducts: Product
}

type Product {
  inStock: Boolean @join__field(graph: INVENTORY)
  name: String @join__field(graph: PRODUCTS)
}
"#;

        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }

        assert_eq!(diagnostics.len(), 2)
    }
}
