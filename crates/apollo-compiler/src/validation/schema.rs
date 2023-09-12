use crate::{
    ast,
    diagnostics::{DiagnosticData, Label},
    schema, ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::cst::{self, CstNode};
use std::collections::HashSet;

pub fn validate_schema_definition(
    db: &dyn ValidationDatabase,
    schema_definition: ast::TypeWithExtensions<ast::SchemaDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let root_operations: Vec<_> = schema_definition.root_operations().cloned().collect();
    // A GraphQL schema must have a Query root operation.
    let has_query = root_operations
        .iter()
        .any(|&(operation_type, _)| operation_type == ast::OperationType::Query);
    if !has_query {
        if let Some(&location) = schema_definition.definition.location() {
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

    diagnostics.extend(super::directive::validate_directives2(
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
    root_op_defs: &[(ast::OperationType, ast::NamedType)],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let mut seen: HashSet<ast::NamedType> = HashSet::new();
    let whole_op_def_location = |name: &ast::Name| {
        name.location().and_then(|loc| {
            super::lookup_cst_location(
                db.upcast(),
                *loc,
                |cst: cst::RootOperationTypeDefinition| Some(cst.syntax().text_range()),
            )
        })
    };

    for (_op_type, name) in root_op_defs {
        // All root operations in a schema definition must be unique.
        //
        // Return a Unique Operation Definition error in case of a duplicate name.
        if let Some(prev_def) = seen.get(name) {
            if let (Some(original_definition), Some(redefined_definition)) =
                (whole_op_def_location(prev_def), whole_op_def_location(name))
            {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        redefined_definition.into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "root operation type definition",
                            name: name.to_string(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{name}` redefined here")),
                    ])
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                );
            }
        } else {
            seen.insert(name.clone());
        }

        // Root Operation Named Type must be of Object Type.
        //
        // Return a Object Type error if it's any other type definition.
        let type_def = schema.types.get(name);
        if let Some(type_def) = type_def {
            if !matches!(type_def, schema::ExtendedType::Object(_)) {
                if let Some(op_loc) = whole_op_def_location(name) {
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
        } else if let Some(op_loc) = whole_op_def_location(name) {
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
