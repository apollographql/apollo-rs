use std::collections::{HashMap, HashSet};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::FieldDefinition,
    validation::{ast_type_definitions, ValidationSet},
    ValidationDatabase,
};
use apollo_parser::ast;

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Object Type definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.object_types();
    for (file_id, ast_def) in ast_type_definitions::<ast::ObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let original_definition = hir[name].loc();
            let redefined_definition = (file_id, &ast_def).into();
            if original_definition == redefined_definition {
                // The HIR node was built from this AST node. This is fine.
            } else {
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
        }
    }

    // Object Type field validations.
    for object in db.object_types().values() {
        let mut seen: HashMap<&str, &FieldDefinition> = HashMap::new();

        let fields = object.fields_definition();
        for field in fields {
            // Fields in an Object Type definition must be unique
            //
            // Returns Unique Value error.
            let field_name = field.name();
            let redefined_definition = field.loc();

            if let Some(prev_field) = seen.get(&field_name) {
                let original_definition = prev_field.loc();
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db, original_definition.into(),
                        DiagnosticData::UniqueField {
                            field: field_name.into(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        }
                    )
                    .labels([
                        Label::new(original_definition, format!("previous definition of `{field_name}` here")),
                        Label::new(redefined_definition, format!("`{field_name}` redefined here")),
                    ])
                    .help(format!("`{field_name}` field must only be defined once in this object type definition."))
                );
            } else {
                seen.insert(field_name, field);
            }

            // Field types in Object Types must be of output type
            if let Some(field_ty) = field.ty().type_def(db.upcast()) {
                if !field.ty().is_output_type(db.upcast()) {
                    diagnostics.push(
                        ApolloDiagnostic::new(db, field.loc().into(), DiagnosticData::OutputType {
                            name: field.name().into(),
                            ty: field_ty.kind(),
                        })
                        .label(Label::new(field.loc(), format!("this is of `{}` type", field_ty.kind())))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name())),
                    );
                }
            } else if let Some(field_ty_loc) = field.ty().loc() {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        field_ty_loc.into(),
                        DiagnosticData::UndefinedDefinition {
                            name: field.name().into(),
                        },
                    )
                    .label(Label::new(field_ty_loc, "not found in this scope")),
                );
            } else {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        field.loc().into(),
                        DiagnosticData::UndefinedDefinition {
                            name: field.ty().name(),
                        },
                    )
                    .label(Label::new(field.loc(), "not found in this scope")),
                );
            }
        }
    }

    let objects = db.object_types();
    let defined_interfaces: HashSet<ValidationSet> = db
        .interfaces()
        .iter()
        .map(|(name, interface)| ValidationSet {
            name: name.to_owned(),
            loc: interface.loc(),
        })
        .collect();
    for object in objects.values() {
        // Implements Interfaces must be defined.
        //
        // Returns Undefined Definition error.
        let implements_interfaces: HashSet<ValidationSet> = object
            .implements_interfaces()
            .iter()
            .map(|interface| ValidationSet {
                name: interface.interface().to_owned(),
                loc: interface.loc(),
            })
            .collect();
        let diff = implements_interfaces.difference(&defined_interfaces);
        for undefined in diff {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    undefined.loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: undefined.name.clone(),
                    },
                )
                .label(Label::new(undefined.loc, "not found in this scope")),
            );
        }

        // Transitively implemented interfaces must be defined on an implementing
        // type or interface.
        //
        // Returns Transitive Implemented Interfaces error.
        let transitive_interfaces: HashSet<ValidationSet> = object
            .implements_interfaces()
            .iter()
            .filter_map(|implements_interface| {
                if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
                    let child_interfaces: HashSet<ValidationSet> = interface
                        .implements_interfaces()
                        .iter()
                        .map(|interface| ValidationSet {
                            name: interface.interface().to_owned(),
                            loc: implements_interface.loc(),
                        })
                        .collect();
                    Some(child_interfaces)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        let transitive_diff = transitive_interfaces.difference(&implements_interfaces);
        for undefined in transitive_diff {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    undefined.loc.into(),
                    DiagnosticData::TransitiveImplementedInterfaces {
                        missing_interface: undefined.name.clone(),
                    },
                )
                .label(Label::new(
                    undefined.loc,
                    format!("{} must also be implemented here", undefined.name),
                )),
            );
        }

        // When defining an interface that implements another interface, the
        // implementing interface must define each field that is specified by
        // the implemented interface.
        //
        // Returns a Missing Field error.
        let fields: HashSet<ValidationSet> = object
            .fields_definition()
            .iter()
            .map(|field| ValidationSet {
                name: field.name().into(),
                loc: field.loc(),
            })
            .collect();
        for implements_interface in object.implements_interfaces().iter() {
            if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
                let implements_interface_fields: HashSet<ValidationSet> = interface
                    .fields_definition()
                    .iter()
                    .map(|field| ValidationSet {
                        name: field.name().into(),
                        loc: field.loc(),
                    })
                    .collect();

                let field_diff = implements_interface_fields.difference(&fields);

                for missing_field in field_diff {
                    let name = &missing_field.name;
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            object.loc().into(),
                            DiagnosticData::MissingField {
                                field: missing_field.name.to_string(),
                            },
                        )
                        .labels([
                            Label::new(
                                missing_field.loc,
                                format!("`{name}` was originally defined here"),
                            ),
                            Label::new(
                                object.loc(),
                                format!("add `{name}` field to this object"),
                            ),
                        ])
                        .help("An object must provide all fields required by the interfaces it implements"),
                    );
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_generates_diagnostics_for_non_output_field_types() {
        let input = r#"
query mainPage {
  width
  result
  entity
}

type Query {
  width: Int
  img: Url
  relationship: Person
  entity: NamedEntity
  depth: Number
  result: SearchResult
  permissions: Auth
  coordinates: Point2D
  main: mainPage
}

type Person {
  name: String
  age: Int
}

type Photo {
  size: Int
  type: String
}

interface NamedEntity {
  name: String
}

enum Number {
  INT
  FLOAT
}

union SearchResult = Photo | Person

directive @Auth(username: String!) repeatable on OBJECT | INTERFACE

input Point2D {
  x: Float
  y: Float
}

scalar Url @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 3);
    }
}
