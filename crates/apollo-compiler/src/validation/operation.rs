use std::{collections::HashMap, sync::Arc};

use crate::hir::{Field, ObjectTypeDefinition, Selection, SelectionSet, TypeDefinition};
use crate::{
    diagnostics::{
        IntrospectionField, MissingIdent, SingleRootField, UndefinedField, UniqueDefinition,
        UnsupportedOperation,
    },
    hir, ApolloDiagnostic, FileId, ValidationDatabase,
};

pub fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let operations = db.operations(file_id);
    for def in operations.iter() {
        diagnostics
            .extend(db.validate_directives(def.directives().to_vec(), def.operation_ty().into()));
        diagnostics.extend(db.validate_variable_definitions(def.variables.as_ref().clone()));
        diagnostics.extend(db.validate_selection_set(
            def.selection_set().clone(),
            TypeDefinition::ObjectTypeDefinition(def.object_type(db.upcast()).unwrap().clone()),
        ));
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
    let mut seen: HashMap<&str, &hir::OperationDefinition> = HashMap::new();
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

    diagnostics
}

/*
pub fn check_selection_set(db: &dyn ValidationDatabase, selection_set: &SelectionSet, obj_type: Arc<ObjectTypeDefinition>, diagnostics: &mut Vec<ApolloDiagnostic>) {
    for selection in selection_set.selection() {
        if let Selection::Field(field) = selection {
            check_field_selection(db, field, obj_type.clone(), diagnostics);
        } /*
        TODO How can I get the type of the selection as an ObjectTypeDefinition / InterfaceTypeDefinition ? */
        // TODO handle fragments somewhere else
        /*else if let Selection::FragmentSpread(fragment_spread) = selection {
            let fragment = db.fragment(fragment_spread.fragment_name());
            let fragment_type = fragment.type_condition().unwrap();
            let fragment_type = db.object_type(fragment_type);
            check_selection_set(db, fragment.selection_set(), fragment_type, diagnostics);
        } */
        else if let Selection::InlineFragment(inline_fragment) = selection {
            let fragment_type = inline_fragment.type_condition().unwrap();
            let fragment_type = db.object_type(fragment_type);
            check_selection_set(db, inline_fragment.selection_set(), fragment_type, diagnostics);
        }

    }
}

pub fn format_invalid_field_error(db: &dyn ValidationDatabase, field: &Field, help: String, diagnostics: &mut Vec<ApolloDiagnostic>) {

    let field_name = field.name().into();
    let op_offset = field.loc().offset();
    let op_len = field.loc().node_len();

    diagnostics.push(ApolloDiagnostic::UndefinedField(UndefinedField {
        field: field_name,
        src: db.source_code(field.loc().file_id()),
        definition: (op_offset, op_len).into(),
        help,
    }));
}

pub fn check_field_selection(db: &dyn ValidationDatabase, field: &Field, obj_type: Arc<ObjectTypeDefinition>, diagnostics: &mut Vec<ApolloDiagnostic>) {
    // Field type is none -> not defined on object type
    let field_type = field.ty(db.upcast());
    if field_type.is_none() {
        let help = format!("`{}` is not defined on `{}` type", field.name(), obj_type.name());
        format_invalid_field_error(db, field, help, diagnostics);
    } else {
        // Get the type system definition for the type of the field - is there a better way to do this?
        let field_type_def = field.ty(db.upcast()).unwrap().type_def(db.upcast());

        // TODO Handle Interface here - should be the same handling as object type
        if let Some(TypeDefinition::ObjectTypeDefinition(field_type)) = field_type_def {
            check_selection_set(db, field.selection_set(), field_type.clone(), diagnostics);
        }
    }
}
*/
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
                    diagnostics.push(ApolloDiagnostic::IntrospectionField(IntrospectionField {
                        field: field.name().into(),
                        src: db.source_code(op.loc().file_id()),
                        definition: (field.loc.offset(), field.loc.node_len()).into(),
                    }));
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
    if !subscriptions.is_empty() && db.schema().subscription(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = subscriptions
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
    if !queries.is_empty() && db.schema().query(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = queries
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
    if !mutations.is_empty() && db.schema().mutation(db.upcast()).is_none() {
        let unsupported_ops: Vec<ApolloDiagnostic> = mutations
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
            println!("{diagnostic}")
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
            println!("{diagnostic}")
        }

        assert_eq!(diagnostics.len(), 2)
    }

    #[test]
    fn it_validates_fields_in_operations() {
        let input = r#"
query getProduct {
  name
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

        assert_eq!(diagnostics.len(), 1)
    }
}
