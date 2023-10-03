use crate::{
    ast,
    diagnostics::{DiagnosticData, Label},
    schema, ApolloDiagnostic, Node, ValidationDatabase,
};

pub fn validate_schema_definition(
    db: &dyn ValidationDatabase,
    schema_definition: ast::TypeWithExtensions<ast::SchemaDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let root_operations: Vec<_> = schema_definition.root_operations().cloned().collect();
    // A GraphQL schema must have a Query root operation.
    let has_query = root_operations
        .iter()
        .any(|op| op.0 == ast::OperationType::Query);
    if !has_query {
        if let Some(location) = schema_definition.definition.location() {
            diagnostics.push(
                ApolloDiagnostic::new(db, location.into(), DiagnosticData::QueryRootOperationType)
                    .label(Label::new(
                        location,
                        "`query` root operation type must be defined here",
                    )),
            );
        }
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
pub fn validate_root_operation_definitions(
    db: &dyn ValidationDatabase,
    root_op_defs: &[Node<(ast::OperationType, ast::NamedType)>],
) -> Vec<ApolloDiagnostic> {
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
                if let Some(op_loc) = op.location() {
                    let (particle, kind) = match type_def {
                        schema::ExtendedType::Scalar(_) => ("an", "scalar"),
                        schema::ExtendedType::Union(_) => ("an", "union"),
                        schema::ExtendedType::Enum(_) => ("an", "enum"),
                        schema::ExtendedType::Interface(_) => ("an", "interface"),
                        schema::ExtendedType::InputObject(_) => ("an", "input object"),
                        schema::ExtendedType::Object(_) => unreachable!(),
                    };
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            op_loc.into(),
                            DiagnosticData::ObjectType {
                                name: name.to_string(),
                                ty: kind,
                            },
                        )
                        .label(Label::new(op_loc, format!("This is {particle} {kind}")))
                        .help("root operation type must be an object type"),
                    );
                }
            }
        } else if let Some(op_loc) = op.location() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    op_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: name.to_string(),
                    },
                )
                .label(Label::new(op_loc, "not found in this scope")),
            );
        }
    }

    diagnostics
}
