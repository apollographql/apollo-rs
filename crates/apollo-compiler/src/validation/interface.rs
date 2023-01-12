use std::collections::{HashMap, HashSet};

use crate::{
    diagnostics::{Diagnostic2, DiagnosticData, Label},
    hir::FieldDefinition,
    validation::{ast_type_definitions, ValidationSet},
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Interface definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.interfaces();
    for (file_id, ast_def) in ast_type_definitions::<ast::InterfaceTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let original_definition = hir_def.name_src().loc().unwrap_or(hir_def.loc());
            let redefined_definition = ast_def
                .name()
                .map(|name| (file_id, &name).into())
                .unwrap_or((file_id, &ast_def).into());
            if original_definition == redefined_definition {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        (db, redefined_definition).into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "interface",
                            name: name.into(),
                            original_definition: (db, original_definition).into(),
                            redefined_definition: (db, redefined_definition).into(),
                        },
                    )
                    .labels([
                        Label::new(
                            (db, original_definition),
                            format!("previous definition of `{}` here", name),
                        ),
                        Label::new(
                            (db, redefined_definition),
                            format!("`{}` redefined here", name),
                        ),
                    ])
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                ));
            }
        }
    }

    // Interface must not implement itself.
    //
    // Return Recursive Definition error.
    //
    // NOTE(@lrlna): we should also check for more sneaky cyclic references for interfaces like this, for example:
    //
    // interface Node implements Named & Node {
    //   id: ID!
    //   name: String
    // }
    //
    // interface Named implements Node & Named {
    //   id: ID!
    //   name: String
    // }
    for (name, interface_def) in db.interfaces().iter() {
        for implements_interface in interface_def.implements_interfaces() {
            if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
                let super_name = interface.name();
                if name == super_name {
                    diagnostics.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
                            (db, implements_interface.loc()).into(),
                            DiagnosticData::RecursiveInterfaceDefinition {
                                name: super_name.into(),
                            },
                        )
                        .label(Label::new(
                            (db, implements_interface.loc()),
                            format!("interface {} cannot implement itself", super_name),
                        )),
                    ));
                }
            }
        }
    }

    // Interface Type field validations.
    for interface_def in db.interfaces().values() {
        let mut seen: HashMap<&str, &FieldDefinition> = HashMap::new();

        let fields = interface_def.fields_definition();

        for field in fields {
            // Fields in an Interface definition must be unique
            let field_name = field.name();
            let redefined_definition = field.loc();

            if let Some(prev_field) = seen.get(&field_name) {
                let original_definition = prev_field.loc();

                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        (db, redefined_definition).into(),
                        DiagnosticData::UniqueField {
                            field: field_name.into(),
                            original_definition: (db, original_definition).into(),
                            redefined_definition: (db, redefined_definition).into(),
                        }
                    )
                    .labels([
                        Label::new((db, original_definition), format!("previous definition of `{field_name}` here")),
                        Label::new((db, redefined_definition), format!("`{field_name}` redefined here")),
                    ])
                    .help(format!("`{field_name}` field must only be defined once in this interface definition."))
                ));
            } else {
                seen.insert(field_name, field);
            }

            // Field types in interface types must be of output type
            if let Some(field_ty) = field.ty().type_def(db.upcast()) {
                if !field.ty().is_output_type(db.upcast()) {
                    diagnostics.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new((db, field.loc()).into(), DiagnosticData::OutputType {
                            name: field.name().into(),
                            ty: field_ty.kind(),
                        })
                        .label(Label::new((db, field.loc()), format!("this is of `{}` type", field_ty.kind())))
                        .help(format!("Scalars, Objects, Interfaces, Unions and Enums are output types. Change `{}` field to return one of these output types.", field.name())),
                    ));
                }
            } else if let Some(field_ty_loc) = field.ty().loc() {
                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        (db, field_ty_loc).into(),
                        DiagnosticData::UndefinedDefinition {
                            name: field.name().into(),
                        },
                    )
                    .label(Label::new((db, field_ty_loc), "not found in this scope")),
                ));
            } else {
                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
                        (db, field.loc()).into(),
                        DiagnosticData::UndefinedDefinition {
                            name: field.ty().name().into(),
                        },
                    )
                    .label(Label::new((db, field.loc()), "not found in this scope")),
                ));
            }
        }
    }

    let interfaces = db.interfaces();
    let defined_interfaces: HashSet<ValidationSet> = interfaces
        .iter()
        .map(|(name, interface)| ValidationSet {
            name: name.to_owned(),
            loc: interface.loc(),
        })
        .collect();
    for interface_def in interfaces.values() {
        // Implements Interfaces must be defined.
        //
        // Returns Undefined Definition error.
        let implements_interfaces: HashSet<ValidationSet> = interface_def
            .implements_interfaces()
            .iter()
            .map(|interface| ValidationSet {
                name: interface.interface().to_owned(),
                loc: interface.loc(),
            })
            .collect();
        let diff = implements_interfaces.difference(&defined_interfaces);
        for undefined in diff {
            diagnostics.push(ApolloDiagnostic::Diagnostic2(
                Diagnostic2::new(
                    (db, undefined.loc).into(),
                    DiagnosticData::UndefinedDefinition {
                        name: undefined.name.clone(),
                    },
                )
                .label(Label::new((db, undefined.loc), "not found in this scope")),
            ));
        }

        // Transitively implemented interfaces must be defined on an implementing
        // type or interface.
        //
        // Returns Transitive Implemented Interfaces error.
        let transitive_interfaces: HashSet<ValidationSet> = interface_def
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
            diagnostics.push(ApolloDiagnostic::Diagnostic2(
                Diagnostic2::new(
                    (db, undefined.loc).into(),
                    DiagnosticData::TransitiveImplementedInterfaces {
                        missing_interface: undefined.name.clone(),
                    },
                )
                .label(Label::new(
                    (db, undefined.loc),
                    format!("{} must also be implemented here", undefined.name),
                )),
            ));
        }

        // When defining an interface that implements another interface, the
        // implementing interface must define each field that is specified by
        // the implemented interface.
        //
        // Returns a Missing Field error.
        let fields: HashSet<ValidationSet> = interface_def
            .fields_definition()
            .iter()
            .map(|field| ValidationSet {
                name: field.name().into(),
                loc: field.loc(),
            })
            .collect();
        for implements_interface in interface_def.implements_interfaces().iter() {
            if let Some(super_interface) = implements_interface.interface_definition(db.upcast()) {
                let implements_interface_fields: HashSet<ValidationSet> = super_interface
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
                    diagnostics.push(ApolloDiagnostic::Diagnostic2(
                        Diagnostic2::new(
                            (db, interface_def.loc()).into(),
                            DiagnosticData::MissingField {
                                field: name.clone(),
                            },
                        )
                        .labels([
                            Label::new(
                                (db, super_interface.loc()),
                                format!("`{name}` was originally defined here"),
                            ),
                            Label::new(
                                (db, interface_def.loc()),
                                format!("add `{name}` field to this interface"),
                            ),
                        ])
                        .help("An interface must be a super-set of all interfaces it implements"),
                    ));
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
    fn it_fails_validation_with_duplicate_operation_fields() {
        let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
  name: String
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_duplicate_interface_definitions() {
        let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_recursive_interface_definition() {
        let input = r#"
type Query implements NamedEntity {
  imgSize: Int
  name: String
  image: URL
  results: [Int]
}

interface NamedEntity implements NamedEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_undefined_interface_definition() {
        let input = r#"
interface NamedEntity implements NewEntity {
  name: String
  image: URL
  results: [Int]
}

scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_missing_transitive_interface() {
        let input = r#"
type Query implements Node {
  id: ID!
}

interface Node {
  id: ID!
}

interface Resource implements Node {
  id: ID!
  width: Int
}

interface Image implements Resource & Node {
  id: ID!
  thumbnail: String
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_generates_diagnostics_for_non_output_field_types() {
        let input = r#"
query mainPage {
  name
}

type Query {
  name: mainInterface
}

interface mainInterface {
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
