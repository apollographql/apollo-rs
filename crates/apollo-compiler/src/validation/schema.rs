use crate::ast;
use crate::schema;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::DiagnosticList;

pub(crate) fn validate_schema_definition(diagnostics: &mut DiagnosticList, schema: &crate::Schema) {
    // A GraphQL schema must have a Query root operation.
    if schema.schema_definition.query.is_none() {
        let location = schema.schema_definition.location();
        diagnostics.push(location, DiagnosticData::QueryRootOperationType);
    }
    validate_root_operation_definitions(diagnostics, schema);

    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        schema.schema_definition.directives.iter_ast(),
        ast::DirectiveLocation::Schema,
        // schemas don't use variables
        Default::default(),
    );
}

// The query, mutation, and subscription root types must all be different
// types if provided.
//
// Return a Duplicate Root Operation Type error in case of a duplicate name.
pub(crate) fn validate_root_operation_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
) {
    let mut seen: Vec<(ast::OperationType, &crate::Name)> = Vec::new();

    for (op, name) in schema.schema_definition.iter_root_operations() {
        // Root Operation Named Type must be of Object Type.
        //
        // Return a Object Type error if it's any other type definition.
        let type_def = schema.types.get(name.as_ref());
        if let Some(type_def) = type_def {
            if !matches!(type_def, schema::ExtendedType::Object(_)) {
                diagnostics.push(
                    name.location(),
                    DiagnosticData::RootOperationObjectType {
                        name: name.name.clone(),
                        describe_type: type_def.describe(),
                    },
                );
            }
        } else {
            diagnostics.push(
                name.location(),
                DiagnosticData::UndefinedDefinition {
                    name: name.name.clone(),
                },
            );
        }

        if let Some((original_op, original_name)) = seen
            .iter()
            .find(|(_, seen_name)| *seen_name == name.as_ref())
        {
            diagnostics.push(
                name.location(),
                DiagnosticData::DuplicateRootOperationType {
                    name: name.name.clone(),
                    original_operation: *original_op,
                    original_definition: original_name.location(),
                    redefined_operation: op,
                    redefined_definition: name.location(),
                },
            );
        } else {
            seen.push((op, name));
        }
    }
}
