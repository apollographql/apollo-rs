use super::*;

pub(super) fn document_from_ast(
    schema: &Schema,
    document: &ast::Document,
) -> Result<ExecutableDocument, TypeError> {
    let mut named_operations = IndexMap::new();
    let mut anonymous_operation = None;
    let mut fragments = IndexMap::new();
    for definition in &document.definitions {
        match definition {
            ast::Definition::OperationDefinition(operation) => {
                if let Some(name) = &operation.name {
                    if let Entry::Vacant(entry) = named_operations.entry(name.clone()) {
                        entry.insert(
                            operation.same_location(Operation::from_ast(schema, operation)?),
                        );
                    }
                } else if anonymous_operation.is_none() {
                    anonymous_operation =
                        Some(operation.same_location(Operation::from_ast(schema, operation)?));
                }
            }
            ast::Definition::FragmentDefinition(fragment) => {
                if let Entry::Vacant(entry) = fragments.entry(fragment.name.clone()) {
                    entry.insert(fragment.same_location(Fragment::from_ast(schema, fragment)?));
                }
            }
            _ => {}
        }
    }
    Ok(ExecutableDocument {
        named_operations,
        anonymous_operation,
        fragments,
    })
}

impl Operation {
    fn from_ast(schema: &Schema, ast: &ast::OperationDefinition) -> Result<Self, TypeError> {
        let mut selection_set = SelectionSet::for_operation(schema, ast.operation_type)?;
        selection_set.extend_from_ast(schema, &ast.selection_set)?;
        Ok(Self {
            selection_set,
            operation_type: ast.operation_type,
            variables: ast.variables.clone(),
            directives: ast.directives.clone(),
        })
    }
}

impl Fragment {
    fn from_ast(schema: &Schema, ast: &ast::FragmentDefinition) -> Result<Self, TypeError> {
        let mut selection_set = SelectionSet::new(schema, ast.type_condition.clone())?;
        selection_set.extend_from_ast(schema, &ast.selection_set)?;
        Ok(Self {
            selection_set,
            directives: ast.directives.clone(),
        })
    }
}

impl SelectionSet {
    pub(super) fn extend_from_ast(
        &mut self,
        schema: &Schema,
        ast_selections: &[ast::Selection],
    ) -> Result<(), TypeError> {
        for selection in ast_selections {
            match selection {
                ast::Selection::Field(ast) => self.push(
                    ast.same_location(
                        self.new_field(schema, ast.name.clone())?
                            .with_alias(ast.alias.clone())
                            .with_arguments(ast.arguments.iter().cloned())
                            .with_directives(ast.directives.iter().cloned())
                            .with_ast_selections(schema, &ast.selection_set)?,
                    ),
                ),
                ast::Selection::FragmentSpread(ast) => self.push(
                    ast.same_location(
                        self.new_fragment_spread(ast.fragment_name.clone())
                            .with_directives(ast.directives.iter().cloned()),
                    ),
                ),
                ast::Selection::InlineFragment(ast) => self.push(
                    ast.same_location(
                        self.new_inline_fragment(schema, ast.type_condition.clone())?
                            .with_directives(ast.directives.iter().cloned())
                            .with_ast_selections(schema, &ast.selection_set)?,
                    ),
                ),
            }
        }
        Ok(())
    }
}
