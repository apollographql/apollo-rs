use super::*;

pub(super) fn document_from_ast(
    schema: &Schema,
    document: &ast::Document,
) -> (ExecutableDocument, Result<(), FromAstError>) {
    let mut named_operations = IndexMap::new();
    let mut anonymous_operation = None;
    let mut fragments = IndexMap::new();
    let mut errors = FromAstError {
        undefined_root_operation: Vec::new(),
        undefined_type: Vec::new(),
        undefined_field: Vec::new(),
    };
    for definition in &document.definitions {
        match definition {
            ast::Definition::OperationDefinition(operation) => {
                if let Some(name) = &operation.name {
                    if let Entry::Vacant(entry) = named_operations.entry(name.clone()) {
                        if let Some(op) = Operation::from_ast(schema, &mut errors, operation) {
                            entry.insert(operation.same_location(op));
                        } else {
                            errors.undefined_root_operation.push(operation.clone())
                        }
                    }
                } else if anonymous_operation.is_none() {
                    if let Some(op) = Operation::from_ast(schema, &mut errors, operation) {
                        anonymous_operation = Some(operation.same_location(op));
                    }
                }
            }
            ast::Definition::FragmentDefinition(fragment) => {
                if let Entry::Vacant(entry) = fragments.entry(fragment.name.clone()) {
                    entry.insert(fragment.same_location(Fragment::from_ast(
                        schema,
                        &mut errors,
                        fragment,
                    )));
                }
            }
            _ => {}
        }
    }
    let doc = ExecutableDocument {
        named_operations,
        anonymous_operation,
        fragments,
    };
    if errors.undefined_root_operation.is_empty()
        && errors.undefined_type.is_empty()
        && errors.undefined_field.is_empty()
    {
        (doc, Ok(()))
    } else {
        (doc, Err(errors))
    }
}

impl Operation {
    fn from_ast(
        schema: &Schema,
        errors: &mut FromAstError,
        ast: &ast::OperationDefinition,
    ) -> Option<Self> {
        let ty = schema.root_operation(ast.operation_type)?.node.clone();
        let mut selection_set = SelectionSet::new(ty);
        selection_set.extend_from_ast(schema, errors, &ast.selection_set);
        Some(Self {
            selection_set,
            operation_type: ast.operation_type,
            variables: ast.variables.clone(),
            directives: ast.directives.clone(),
        })
    }
}

impl Fragment {
    fn from_ast(schema: &Schema, errors: &mut FromAstError, ast: &ast::FragmentDefinition) -> Self {
        let mut selection_set = SelectionSet::new(ast.type_condition.clone());
        selection_set.extend_from_ast(schema, errors, &ast.selection_set);
        Self {
            selection_set,
            directives: ast.directives.clone(),
        }
    }
}

impl SelectionSet {
    pub(super) fn extend_from_ast(
        &mut self,
        schema: &Schema,
        errors: &mut FromAstError,
        ast_selections: &[ast::Selection],
    ) {
        for selection in ast_selections {
            match selection {
                ast::Selection::Field(ast) => match self.new_field(schema, ast.name.clone()) {
                    Ok(field) => self.push(
                        ast.same_location(
                            field
                                .with_alias(ast.alias.clone())
                                .with_arguments(ast.arguments.iter().cloned())
                                .with_directives(ast.directives.iter().cloned())
                                .with_ast_selections(schema, errors, &ast.selection_set),
                        ),
                    ),
                    Err(FieldLookupError::NoSuchField) => errors.undefined_field.push(ast.clone()),
                    Err(FieldLookupError::NoSuchType) => {
                        errors.undefined_type.push((self.ty.clone(), ast.clone()))
                    }
                },
                ast::Selection::FragmentSpread(ast) => self.push(
                    ast.same_location(
                        self.new_fragment_spread(ast.fragment_name.clone())
                            .with_directives(ast.directives.iter().cloned()),
                    ),
                ),
                ast::Selection::InlineFragment(ast) => self.push(
                    ast.same_location(
                        self.new_inline_fragment(ast.type_condition.clone())
                            .with_directives(ast.directives.iter().cloned())
                            .with_ast_selections(schema, errors, &ast.selection_set),
                    ),
                ),
            }
        }
    }
}

impl Field {
    fn with_ast_selections(
        mut self,
        schema: &Schema,
        errors: &mut FromAstError,
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
        schema: &Schema,
        errors: &mut FromAstError,
        ast_selections: &[ast::Selection],
    ) -> Self {
        self.selection_set
            .extend_from_ast(schema, errors, ast_selections);
        self
    }
}
