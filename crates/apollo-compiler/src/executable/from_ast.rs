use super::*;
use crate::ty;

pub(crate) struct BuildErrors<'a> {
    pub(crate) errors: &'a mut DiagnosticList,
    pub(crate) path: SelectionPath,
}

pub(crate) fn document_from_ast(
    schema: Option<&Schema>,
    document: &ast::Document,
    errors: &mut DiagnosticList,
    type_system_definitions_are_errors: bool,
) -> ExecutableDocument {
    let mut named_operations = IndexMap::new();
    let mut anonymous_operation = None::<Node<Operation>>;
    let mut multiple_anonymous = false;
    let mut fragments = IndexMap::new();
    let mut errors = BuildErrors {
        errors,
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
                    if let Some(anonymous) = &anonymous_operation {
                        errors.errors.push(
                            anonymous.location(),
                            BuildError::AmbiguousAnonymousOperation,
                        )
                    }
                    if let Entry::Vacant(entry) = named_operations.entry(name.clone()) {
                        errors.path.root = ExecutableDefinitionName::NamedOperation(
                            operation.operation_type,
                            name.clone(),
                        );
                        if let Some(op) = Operation::from_ast(schema, &mut errors, operation) {
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
                        let (key, _) = named_operations.get_key_value(name).unwrap();
                        errors.errors.push(
                            name.location(),
                            BuildError::OperationNameCollision {
                                name_at_previous_location: key.clone(),
                            },
                        );
                    }
                } else if let Some(previous) = &anonymous_operation {
                    if !multiple_anonymous {
                        multiple_anonymous = true;
                        errors
                            .errors
                            .push(previous.location(), BuildError::AmbiguousAnonymousOperation)
                    }
                    errors.errors.push(
                        operation.location(),
                        BuildError::AmbiguousAnonymousOperation,
                    )
                } else if !named_operations.is_empty() {
                    errors.errors.push(
                        operation.location(),
                        BuildError::AmbiguousAnonymousOperation,
                    )
                } else {
                    errors.path.root =
                        ExecutableDefinitionName::AnonymousOperation(operation.operation_type);
                    if let Some(op) = Operation::from_ast(schema, &mut errors, operation) {
                        anonymous_operation = Some(operation.same_location(op));
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
            ast::Definition::FragmentDefinition(fragment) => {
                if let Entry::Vacant(entry) = fragments.entry(fragment.name.clone()) {
                    errors.path.root = ExecutableDefinitionName::Fragment(fragment.name.clone());
                    if let Some(node) = Fragment::from_ast(schema, &mut errors, fragment) {
                        entry.insert(fragment.same_location(node));
                    }
                } else {
                    let (key, _) = fragments.get_key_value(&fragment.name).unwrap();
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
    ExecutableDocument {
        sources: document.sources.clone(),
        named_operations,
        anonymous_operation,
        fragments,
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
