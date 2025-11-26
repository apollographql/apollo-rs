use super::*;
use crate::ty;
use crate::validation::WithErrors;
use std::sync::Arc;

pub(crate) struct BuildErrors<'a> {
    pub(crate) errors: &'a mut DiagnosticList,
    pub(crate) path: SelectionPath,
}

/// A builder for constructing an [`ExecutableDocument`] from multiple AST documents.
///
/// This builder allows you to parse and combine executable definitions (operations and fragments)
/// from multiple source files into a single [`ExecutableDocument`].
///
/// # Example
///
/// ```rust
/// use apollo_compiler::{Schema, ExecutableDocument};
/// use apollo_compiler::parser::Parser;
/// # let schema_src = "type Query { user: User } type User { id: ID }";
/// # let schema = Schema::parse_and_validate(schema_src, "schema.graphql").unwrap();
///
/// // Create a builder
/// let mut builder = ExecutableDocument::builder(Some(&schema));
///
/// // Add operations from multiple files
/// Parser::new().parse_into_executable_builder(
///     Some(&schema),
///     "query GetUser { user { id } }",
///     "query1.graphql",
///     &mut builder,
/// );
///
/// // Build the final document
/// let document = builder.build().unwrap();
/// ```
#[derive(Clone)]
pub struct ExecutableDocumentBuilder<'schema> {
    /// The executable document being built
    pub(crate) document: ExecutableDocument,
    /// Optional schema for type checking during build
    schema: Option<&'schema Schema>,
    /// Accumulated diagnostics
    pub(crate) errors: DiagnosticList,
    /// Track if we've seen multiple anonymous operations
    multiple_anonymous: bool,
}

impl Default for ExecutableDocumentBuilder<'_> {
    fn default() -> Self {
        Self::new(None)
    }
}

impl<'schema> ExecutableDocumentBuilder<'schema> {
    /// Creates a new [`ExecutableDocumentBuilder`].
    ///
    /// # Arguments
    ///
    /// * `schema` - Optional schema for type checking. If provided, the builder will validate
    ///   operations and fragments against the schema while building.
    pub fn new(schema: Option<&'schema Schema>) -> Self {
        Self {
            document: ExecutableDocument::new(),
            schema,
            errors: DiagnosticList::new(Default::default()),
            multiple_anonymous: false,
        }
    }

    /// Adds an AST document to the executable document being built.
    ///
    /// # Arguments
    ///
    /// * `document` - The AST document to add
    /// * `type_system_definitions_are_errors` - If true, type system definitions (types, directives, etc.)
    ///   in the document will be reported as errors
    pub fn add_ast_document(
        &mut self,
        document: &ast::Document,
        type_system_definitions_are_errors: bool,
    ) {
        Arc::make_mut(&mut self.errors.sources)
            .extend(document.sources.iter().map(|(k, v)| (*k, v.clone())));
        self.add_ast_document_not_adding_sources(document, type_system_definitions_are_errors);
    }

    pub(crate) fn add_ast_document_not_adding_sources(
        &mut self,
        document: &ast::Document,
        type_system_definitions_are_errors: bool,
    ) {
        let mut errors = BuildErrors {
            errors: &mut self.errors,
            path: SelectionPath {
                nested_fields: Vec::new(),
                // overwritten:
                root: ExecutableDefinitionName::AnonymousOperation(ast::OperationType::Query),
            },
        };

        for definition in &document.definitions {
            debug_assert!(errors.path.nested_fields.is_empty());
            match definition {
                ast::Definition::OperationDefinition(operation) => {
                    if let Some(name) = &operation.name {
                        // Named operation
                        if let Some(anonymous) = &self.document.operations.anonymous {
                            errors.errors.push(
                                anonymous.location(),
                                BuildError::AmbiguousAnonymousOperation,
                            )
                        }
                        if let Entry::Vacant(entry) =
                            self.document.operations.named.entry(name.clone())
                        {
                            errors.path.root = ExecutableDefinitionName::NamedOperation(
                                operation.operation_type,
                                name.clone(),
                            );
                            if let Some(op) =
                                Operation::from_ast(self.schema, &mut errors, operation)
                            {
                                entry.insert(operation.same_location(op));
                            } else {
                                errors.errors.push(
                                    operation.location(),
                                    BuildError::UndefinedRootOperation {
                                        operation_type: operation.operation_type.name(),
                                    },
                                )
                            }
                        } else {
                            let (key, _) =
                                self.document.operations.named.get_key_value(name).unwrap();
                            errors.errors.push(
                                name.location(),
                                BuildError::OperationNameCollision {
                                    name_at_previous_location: key.clone(),
                                },
                            );
                        }
                    } else {
                        // Anonymous operation
                        if let Some(previous) = &self.document.operations.anonymous {
                            if !self.multiple_anonymous {
                                self.multiple_anonymous = true;
                                errors.errors.push(
                                    previous.location(),
                                    BuildError::AmbiguousAnonymousOperation,
                                )
                            }
                            errors.errors.push(
                                operation.location(),
                                BuildError::AmbiguousAnonymousOperation,
                            )
                        } else if !self.document.operations.named.is_empty() {
                            errors.errors.push(
                                operation.location(),
                                BuildError::AmbiguousAnonymousOperation,
                            )
                        } else {
                            errors.path.root = ExecutableDefinitionName::AnonymousOperation(
                                operation.operation_type,
                            );
                            if let Some(op) =
                                Operation::from_ast(self.schema, &mut errors, operation)
                            {
                                self.document.operations.anonymous =
                                    Some(operation.same_location(op));
                            } else {
                                errors.errors.push(
                                    operation.location(),
                                    BuildError::UndefinedRootOperation {
                                        operation_type: operation.operation_type.name(),
                                    },
                                )
                            }
                        }
                    }
                }
                ast::Definition::FragmentDefinition(fragment) => {
                    if let Entry::Vacant(entry) =
                        self.document.fragments.entry(fragment.name.clone())
                    {
                        errors.path.root =
                            ExecutableDefinitionName::Fragment(fragment.name.clone());
                        if let Some(node) = Fragment::from_ast(self.schema, &mut errors, fragment) {
                            entry.insert(fragment.same_location(node));
                        }
                    } else {
                        let (key, _) = self
                            .document
                            .fragments
                            .get_key_value(&fragment.name)
                            .unwrap();
                        errors.errors.push(
                            fragment.name.location(),
                            BuildError::FragmentNameCollision {
                                name_at_previous_location: key.clone(),
                            },
                        )
                    }
                }
                _ => {
                    if type_system_definitions_are_errors {
                        errors.errors.push(
                            definition.location(),
                            BuildError::TypeSystemDefinition {
                                name: definition.name().cloned(),
                                describe: definition.describe(),
                            },
                        )
                    }
                }
            }
        }

        // Merge sources into the document
        Arc::make_mut(&mut self.document.sources)
            .extend(document.sources.iter().map(|(k, v)| (*k, v.clone())));
    }

    /// Returns the executable document built from all added AST documents.
    #[allow(clippy::result_large_err)] // Typically not called very often
    pub fn build(self) -> Result<ExecutableDocument, WithErrors<ExecutableDocument>> {
        let (document, errors) = self.build_inner();
        errors.into_result_with(document)
    }

    pub(crate) fn build_inner(mut self) -> (ExecutableDocument, DiagnosticList) {
        self.document.sources = self.errors.sources.clone();
        (self.document, self.errors)
    }
}

pub(crate) fn document_from_ast(
    schema: Option<&Schema>,
    document: &ast::Document,
    errors: &mut DiagnosticList,
    type_system_definitions_are_errors: bool,
) -> ExecutableDocument {
    // Use the builder internally but maintain the same API
    let mut builder = ExecutableDocumentBuilder {
        document: ExecutableDocument::new(),
        schema,
        errors: std::mem::replace(errors, DiagnosticList::new(Default::default())),
        multiple_anonymous: false,
    };

    builder.add_ast_document_not_adding_sources(document, type_system_definitions_are_errors);

    let (doc, new_errors) = builder.build_inner();
    *errors = new_errors;

    ExecutableDocument {
        sources: document.sources.clone(),
        operations: doc.operations,
        fragments: doc.fragments,
    }
}

impl Operation {
    fn from_ast(
        schema: Option<&Schema>,
        errors: &mut BuildErrors,
        ast: &ast::OperationDefinition,
    ) -> Option<Self> {
        let ty = if let Some(s) = schema {
            s.root_operation(ast.operation_type)?.clone()
        } else {
            // Hack for validate_standalone_excutable
            ast.operation_type.default_type_name().clone()
        };
        let mut selection_set = SelectionSet::new(ty);
        selection_set.extend_from_ast(schema, errors, &ast.selection_set);
        Some(Self {
            operation_type: ast.operation_type,
            name: ast.name.clone(),
            variables: ast.variables.clone(),
            directives: ast.directives.clone(),
            selection_set,
        })
    }
}

impl Fragment {
    fn from_ast(
        schema: Option<&Schema>,
        errors: &mut BuildErrors,
        ast: &ast::FragmentDefinition,
    ) -> Option<Self> {
        if let Some(schema) = schema {
            if !schema.types.contains_key(&ast.type_condition) {
                errors.errors.push(
                    ast.type_condition.location(),
                    BuildError::UndefinedTypeInNamedFragmentTypeCondition {
                        type_name: ast.type_condition.clone(),
                        fragment_name: ast.name.clone(),
                    },
                );
                return None;
            }
        }
        let mut selection_set = SelectionSet::new(ast.type_condition.clone());
        selection_set.extend_from_ast(schema, errors, &ast.selection_set);
        Some(Self {
            name: ast.name.clone(),
            directives: ast.directives.clone(),
            selection_set,
        })
    }
}

impl SelectionSet {
    pub(crate) fn extend_from_ast(
        &mut self,
        schema: Option<&Schema>,
        errors: &mut BuildErrors,
        ast_selections: &[ast::Selection],
    ) {
        for selection in ast_selections {
            match selection {
                ast::Selection::Field(ast) => {
                    let field_def_result = if let Some(s) = schema {
                        s.type_field(&self.ty, &ast.name).map(|c| c.node.clone())
                    } else {
                        Ok(Node::new(ast::FieldDefinition {
                            description: None,
                            name: ast.name.clone(),
                            arguments: Vec::new(),
                            ty: ty!(UNKNOWN),
                            directives: Default::default(),
                        }))
                    };
                    errors
                        .path
                        .nested_fields
                        .push(ast.alias.clone().unwrap_or_else(|| ast.name.clone()));
                    match field_def_result {
                        Ok(field_def) => {
                            let leaf = ast.selection_set.is_empty();
                            let type_name = field_def.ty.inner_named_type();
                            match schema
                                .as_ref()
                                .and_then(|schema| schema.types.get(type_name))
                            {
                                Some(schema::ExtendedType::Scalar(_)) if !leaf => {
                                    errors.errors.push(
                                        ast.location(),
                                        BuildError::SubselectionOnScalarType {
                                            type_name: type_name.clone(),
                                            path: errors.path.clone(),
                                        },
                                    )
                                }
                                Some(schema::ExtendedType::Enum(_)) if !leaf => errors.errors.push(
                                    ast.location(),
                                    BuildError::SubselectionOnEnumType {
                                        type_name: type_name.clone(),
                                        path: errors.path.clone(),
                                    },
                                ),
                                _ => self.push(
                                    ast.same_location(
                                        Field::new(ast.name.clone(), field_def)
                                            .with_opt_alias(ast.alias.clone())
                                            .with_arguments(ast.arguments.iter().cloned())
                                            .with_directives(ast.directives.iter().cloned())
                                            .with_ast_selections(
                                                schema,
                                                errors,
                                                &ast.selection_set,
                                            ),
                                    ),
                                ),
                            }
                        }
                        Err(schema::FieldLookupError::NoSuchField(type_name, _)) => {
                            errors.errors.push(
                                ast.name.location(),
                                BuildError::UndefinedField {
                                    type_name: type_name.clone(),
                                    field_name: ast.name.clone(),
                                    path: errors.path.clone(),
                                },
                            )
                        }
                        Err(schema::FieldLookupError::NoSuchType) => {
                            // `self.ty` is the name of a type not definied in the schema.
                            // It can come from:
                            // * A root operation type, or a field definition:
                            //   the schema is invalid, no need to record another error here.
                            // * An inline fragment with a type condition:
                            //   we emitted `UndefinedTypeInInlineFragmentTypeCondition` already
                            // * The type condition of a named fragment definition:
                            //   we emitted `UndefinedTypeInNamedFragmentTypeCondition` already
                        }
                    }
                    errors.path.nested_fields.pop();
                }
                ast::Selection::FragmentSpread(ast) => self.push(
                    ast.same_location(
                        self.new_fragment_spread(ast.fragment_name.clone())
                            .with_directives(ast.directives.iter().cloned()),
                    ),
                ),
                ast::Selection::InlineFragment(ast) => {
                    let opt_type_condition = ast.type_condition.clone();
                    match (&opt_type_condition, schema) {
                        (Some(type_condition), Some(schema))
                            if !schema.types.contains_key(type_condition) =>
                        {
                            errors.errors.push(
                                type_condition.location(),
                                BuildError::UndefinedTypeInInlineFragmentTypeCondition {
                                    type_name: type_condition.clone(),
                                    path: errors.path.clone(),
                                },
                            )
                        }
                        _ => self.push(
                            ast.same_location(
                                self.new_inline_fragment(opt_type_condition)
                                    .with_directives(ast.directives.iter().cloned())
                                    .with_ast_selections(schema, errors, &ast.selection_set),
                            ),
                        ),
                    }
                }
            }
        }
    }
}

impl Field {
    fn with_ast_selections(
        mut self,
        schema: Option<&Schema>,
        errors: &mut BuildErrors,
        ast_selections: &[ast::Selection],
    ) -> Self {
        self.selection_set
            .extend_from_ast(schema, errors, ast_selections);
        self
    }
}

impl InlineFragment {
    fn with_ast_selections(
        mut self,
        schema: Option<&Schema>,
        errors: &mut BuildErrors,
        ast_selections: &[ast::Selection],
    ) -> Self {
        self.selection_set
            .extend_from_ast(schema, errors, ast_selections);
        self
    }
}
