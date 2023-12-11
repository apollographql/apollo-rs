use crate::ast;
use crate::schema;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::diagnostics::ValidationError;
use crate::Node;
use crate::ValidationDatabase;

pub(crate) fn validate_schema_definition(
    db: &dyn ValidationDatabase,
    schema_definition: ast::TypeWithExtensions<ast::SchemaDefinition>,
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let root_operations: Vec<_> = schema_definition.root_operations().cloned().collect();
    // A GraphQL schema must have a Query root operation.
    let has_query = root_operations
        .iter()
        .any(|op| op.0 == ast::OperationType::Query);
    if !has_query {
        let location = schema_definition.definition.location();
        diagnostics.push(ValidationError::new(
            location,
            DiagnosticData::QueryRootOperationType,
        ));
    }
    diagnostics.extend(validate_root_operation_definitions(db, &root_operations));

    diagnostics.extend(super::directive::validate_directives(
        db,
        schema_definition.directives(),
        ast::DirectiveLocation::Schema,
        // schemas don't use variables
        Default::default(),
    ));

    diagnostics
}

// All root operations in a schema definition must be unique.
//
// Return a Unique Operation Definition error in case of a duplicate name.
pub(crate) fn validate_root_operation_definitions(
    db: &dyn ValidationDatabase,
    root_op_defs: &[Node<(ast::OperationType, ast::NamedType)>],
) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    for op in root_op_defs {
        let (_op_type, name) = &**op;

        // Root Operation Named Type must be of Object Type.
        //
        // Return a Object Type error if it's any other type definition.
        let type_def = schema.types.get(name);
        if let Some(type_def) = type_def {
            if !matches!(type_def, schema::ExtendedType::Object(_)) {
                let op_loc = name.location();
                let kind = match type_def {
                    schema::ExtendedType::Scalar(_) => "scalar",
                    schema::ExtendedType::Union(_) => "union",
                    schema::ExtendedType::Enum(_) => "enum",
                    schema::ExtendedType::Interface(_) => "interface",
                    schema::ExtendedType::InputObject(_) => "input object",
                    schema::ExtendedType::Object(_) => unreachable!(),
                };
                diagnostics.push(ValidationError::new(
                    op_loc,
                    DiagnosticData::RootOperationObjectType {
                        name: name.to_string(),
                        ty: kind,
                    },
                ));
            }
        } else {
            let op_loc = name.location();
            diagnostics.push(ValidationError::new(
                op_loc,
                DiagnosticData::UndefinedDefinition {
                    name: name.to_string(),
                },
            ));
        }
    }

    diagnostics
}
