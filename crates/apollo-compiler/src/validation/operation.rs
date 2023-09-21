use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    FileId, Node, ValidationDatabase,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct OperationValidationConfig<'vars> {
    /// When false, rules that require a schema to validate are disabled.
    pub has_schema: bool,
    /// The variables defined for this operation.
    pub variables: &'vars [Node<ast::VariableDefinition>],
}

pub(crate) fn validate_operation(
    db: &dyn ValidationDatabase,
    operation: Node<ast::OperationDefinition>,
    has_schema: bool,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = vec![];

    let config = OperationValidationConfig {
        has_schema,
        variables: &operation.variables,
    };

    let schema = db.schema();
    let against_type = schema.root_operation(operation.operation_type);

    let named_fragments = db.ast_named_fragments(operation.location().unwrap().file_id());
    let q = ast::NamedType::new("Query");
    let is_introspection_query = operation.operation_type == ast::OperationType::Query
        && super::selection::operation_fields(
            &named_fragments,
            against_type
                .map(|component| &component.node)
                .unwrap_or_else(|| &q),
            &operation.selection_set,
        )
        .iter()
        .all(|field| {
            matches!(
                field.field.name.as_str(),
                "__type" | "__schema" | "__typename"
            )
        });

    if against_type.is_none() && config.has_schema && !is_introspection_query {
        let operation_word = match operation.operation_type {
            ast::OperationType::Query => "query",
            ast::OperationType::Mutation => "mutation",
            ast::OperationType::Subscription => "subscription",
        };

        let diagnostic = ApolloDiagnostic::new(
            db,
            operation.location().unwrap().into(),
            DiagnosticData::UnsupportedOperation { ty: operation_word },
        )
        .label(Label::new(
            operation.location().unwrap(),
            format!(
                "{} operation is not defined in the schema and is therefore not supported",
                match operation.operation_type {
                    ast::OperationType::Query => "Query",
                    ast::OperationType::Mutation => "Mutation",
                    ast::OperationType::Subscription => "Subscription",
                }
            ),
        ))
        .help(format!(
            "consider defining a `{operation_word}` root operation type in your schema"
        ));
        diagnostics.push(diagnostic);
    }

    if operation.operation_type == ast::OperationType::Subscription {
        let fields = super::selection::operation_fields(
            &named_fragments,
            against_type
                .map(|component| &component.node)
                .unwrap_or_else(|| &q),
            &operation.selection_set,
        );

        if fields.len() > 1 {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    (operation.location().unwrap()).into(),
                    DiagnosticData::SingleRootField {
                        fields: fields.len(),
                        subscription: (operation.location().unwrap()).into(),
                    },
                )
                .label(Label::new(
                    operation.location().unwrap(),
                    format!("subscription with {} root fields", fields.len()),
                ))
                .help(format!(
                    "There are {} root fields: {}. This is not allowed.",
                    fields.len(),
                    fields
                        .iter()
                        .map(|field| field.field.name.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
            );
        }

        let has_introspection_fields = fields
            .iter()
            .find(|field| {
                matches!(
                    field.field.name.as_str(),
                    "__type" | "__schema" | "__typename"
                )
            })
            .map(|field| field.field);
        if let Some(field) = has_introspection_fields {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    (field.location().unwrap()).into(),
                    DiagnosticData::IntrospectionField {
                        field: field.name.to_string(),
                    },
                )
                .label(Label::new(
                    field.location().unwrap(),
                    format!("{} is an introspection field", field.name),
                )),
            );
        }
    }

    diagnostics.extend(super::directive::validate_directives(
        db,
        operation.directives.iter(),
        operation.operation_type.into(),
        &operation.variables,
    ));
    diagnostics.extend(super::variable::validate_variable_definitions2(
        db,
        &operation.variables,
        config.has_schema,
    ));

    diagnostics.extend(super::variable::validate_unused_variables(
        db,
        operation.clone(),
    ));
    diagnostics.extend(super::selection::validate_selection_set2(
        db,
        against_type.map(|component| &component.node),
        &operation.selection_set,
        config,
    ));

    diagnostics
}

pub(crate) fn validate_operation_definitions_inner(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    has_schema: bool,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut anonymous_definitions: Vec<&Node<ast::OperationDefinition>> = vec![];
    let mut num_definitions = 0;

    let document = db.ast(file_id);

    for definition in &document.definitions {
        if let ast::Definition::OperationDefinition(operation) = definition {
            diagnostics.extend(validate_operation(db, operation.clone(), has_schema));

            num_definitions += 1;
            if operation.name.is_none() {
                anonymous_definitions.push(operation);
            }
        }
    }

    if num_definitions > 1 {
        diagnostics.extend(
            anonymous_definitions.into_iter().map(|operation| {
                ApolloDiagnostic::new(db, operation.location().unwrap().into(), DiagnosticData::MissingIdent)
                    .label(Label::new(operation.location().unwrap(), "provide a name for this definition"))
                    .help(format!("GraphQL only allows a short-hand form for defining query operations when only that one operation exists in the document."))
            }),
        );
    }

    diagnostics
}

pub fn validate_operation_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    validate_operation_definitions_inner(db, file_id, false)
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
