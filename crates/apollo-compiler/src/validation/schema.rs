use crate::ast;
use crate::schema;

use crate::validation::diagnostics::DiagnosticData;
use crate::validation::diagnostics::ValidationError;

pub(crate) fn validate_schema_definition(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    // A GraphQL schema must have a Query root operation.
    if schema.schema_definition.query.is_none() {
        let location = schema.schema_definition.location();
        diagnostics.push(ValidationError::new(
            location,
            DiagnosticData::QueryRootOperationType,
        ));
    }
    diagnostics.extend(validate_root_operation_definitions(schema));

    diagnostics.extend(super::directive::validate_directives(
        Some(schema),
        schema.schema_definition.directives.iter_ast(),
        ast::DirectiveLocation::Schema,
        // schemas don't use variables
        Default::default(),
    ));

    diagnostics
}

// All root operations in a schema definition must be unique.
//
// Return a Unique Operation Definition error in case of a duplicate name.
pub(crate) fn validate_root_operation_definitions(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for op in [
        &schema.schema_definition.query,
        &schema.schema_definition.mutation,
        &schema.schema_definition.subscription,
    ] {
        let Some(name) = op else { continue };

        // Root Operation Named Type must be of Object Type.
        //
        // Return a Object Type error if it's any other type definition.
        let type_def = schema.types.get(name.as_ref());
        if let Some(type_def) = type_def {
            if !matches!(type_def, schema::ExtendedType::Object(_)) {
                let op_loc = name.location();
                diagnostics.push(ValidationError::new(
                    op_loc,
                    DiagnosticData::RootOperationObjectType {
                        name: name.name.clone(),
                        describe_type: type_def.describe(),
                    },
                ));
            }
        } else {
            let op_loc = name.location();
            diagnostics.push(ValidationError::new(
                op_loc,
                DiagnosticData::UndefinedDefinition {
                    name: name.name.clone(),
                },
            ));
        }
    }

    diagnostics
}
