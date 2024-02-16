use crate::ast;
use crate::schema;
use crate::schema::SchemaDefinition;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::diagnostics::ValidationError;
use crate::ValidationDatabase;

pub(crate) fn validate_schema_definition(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();
    let schema_definition = &schema.schema_definition;
    // A GraphQL schema must have a Query root operation.
    if schema_definition.query.is_none() {
        let location = schema_definition.location();
        diagnostics.push(ValidationError::new(
            location,
            DiagnosticData::QueryRootOperationType,
        ));
    }
    diagnostics.extend(validate_root_operation_definitions(db, schema_definition));

    let has_schema = true;
    diagnostics.extend(super::directive::validate_directives(
        db,
        schema_definition.directives.iter_ast(),
        ast::DirectiveLocation::Schema,
        // schemas don't use variables
        Default::default(),
        has_schema,
    ));

    diagnostics
}

// All root operations in a schema definition must be unique.
//
// Return a Unique Operation Definition error in case of a duplicate name.
pub(crate) fn validate_root_operation_definitions(
    db: &dyn ValidationDatabase,
    schema_definition: &SchemaDefinition,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    for op in [
        &schema_definition.query,
        &schema_definition.mutation,
        &schema_definition.subscription,
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
