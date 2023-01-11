use std::collections::HashMap;

use crate::{
    diagnostics::{
        MissingIdent, SingleRootField, UndefinedField, UniqueDefinition, UnsupportedOperation,
    },
    hir::{OperationDefinition, Selection},
    ApolloDiagnostic, FileId, ValidationDatabase,
};
// use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, Document};

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
                    let offset = op.loc().offset();
                    let len= op.loc().node_len();
                    return Some(ApolloDiagnostic::MissingIdent(MissingIdent {
                        src: db.source_code(op.loc().file_id()),
                        definition: (offset, len).into(),
                        help: Some(format!("GraphQL allows a short-hand form for defining query operations when only that one operation exists in the document. There are {op_len} operations in this document."))
                    }));
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
                let prev_offset = prev_def.loc().offset();
                let prev_node_len = prev_def.loc().node_len();

                let current_offset = op.loc().offset();
                let current_node_len = op.loc().node_len();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "operation".into(),
                    name: name.into(),
                    src: db.source_code(prev_def.loc().file_id()),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
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
                    let offset = op.loc().offset();
                    let len = op.loc().node_len();
                    Some(ApolloDiagnostic::SingleRootField(SingleRootField {
                        fields: fields.len(),
                        src: db.source_code(op.loc().file_id()),
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
                let op_offset = op.loc().offset();
                let op_len = op.loc().node_len();

                if let Some(loc) = db.schema().loc() {
                    let schema_offset = loc.offset();
                    let schema_len = loc.node_len();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Subscription".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(loc.file_id()),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Subscription".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(op.loc().file_id()),
                        schema: None,
                        help: Some(
                            "consider defining a `subscription` root operation type in your schema"
                                .into(),
                        ),
                    })
                }
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    //   * query operation - query root operation
    if query_operations.len() >= 1 && db.schema().query(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = query_operations
            .iter()
            .map(|op| {
                let op_offset = op.loc().offset();
                let op_len = op.loc().node_len();

                if let Some(loc) = db.schema().loc() {
                    let schema_offset = loc.offset();
                    let schema_len = loc.node_len();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Query".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(loc.file_id()),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Query".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(op.loc().file_id()),
                        schema: None,
                        help: Some(
                            "consider defining a `query` root operation type in your schema".into(),
                        ),
                    })
                }
            })
            .collect();
        diagnostics.extend(unsupported_ops)
    }

    //   * mutation operation - mutation root operation
    if mutation_operations.len() >= 1 && db.schema().mutation(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = mutation_operations
            .iter()
            .map(|op| {
                let op_offset = op.loc().offset();
                let op_len = op.loc().node_len();

                if let Some(loc) = db.schema().loc() {
                    let schema_offset = loc.offset();
                    let schema_len = loc.node_len();
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Mutation".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(loc.file_id()),
                        schema: Some((schema_offset, schema_len).into()),
                        help: None,
                    })
                } else {
                    ApolloDiagnostic::UnsupportedOperation(UnsupportedOperation {
                        ty: "Mutation".into(),
                        operation: (op_offset, op_len).into(),
                        src: db.source_code(op.loc().file_id()),
                        schema: None,
                        help: Some(
                            "consider defining a `mutation` root operation type in your schema"
                                .into(),
                        ),
                    })
                }
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
                    let offset = field.loc().offset();
                    let len = field.loc().node_len();
                    let field_name = field.name().into();
                    let help = if let Some(obj_type) = obj_name {
                        format!("`{}` is not defined on `{}` type", field_name, obj_type)
                    } else {
                        format!(
                            "`{}` is not defined on the current {} root operation type.",
                            field_name,
                            op.operation_ty()
                        )
                    };
                    diagnostics.push(ApolloDiagnostic::UndefinedField(UndefinedField {
                        field: field_name,
                        src: db.source_code(field.loc().file_id()),
                        definition: (offset, len).into(),
                        help,
                    }))
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
        compiler.create_document(input, "schema.graphql");

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
        compiler.create_document(input, "schema.graphql");

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
        compiler.create_document(input, "schema.graphql");

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
        compiler.create_document(input, "schema.graphql");

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
        compiler.create_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic);
        }

        assert_eq!(diagnostics.len(), 2)
    }
}
