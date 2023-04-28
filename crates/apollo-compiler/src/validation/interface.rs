use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, ImplementsInterface},
    validation::ValidationSet,
    ValidationDatabase,
};

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().interfaces;
    for def in defs.values() {
        diagnostics.extend(db.validate_directives(
            def.directives().cloned().collect(),
            hir::DirectiveLocation::Interface,
        ));
        diagnostics.extend(db.validate_interface_definition(def.clone()));
    }

    diagnostics
}

fn collect_nodes<'a, Item: Clone, Ext>(
    base: &'a [Item],
    extensions: &'a [Arc<Ext>],
    method: impl Fn(&'a Ext) -> &'a [Item],
) -> Vec<Item> {
    let mut nodes = base.to_vec();
    for ext in extensions {
        nodes.extend(method(ext).iter().cloned());
    }
    nodes
}

pub fn validate_interface_definition(
    db: &dyn ValidationDatabase,
    interface_def: Arc<hir::InterfaceTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

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
        for implements_interface in interface_def.self_implements_interfaces() {
            if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
                let super_name = interface.name();
                if name == super_name {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            implements_interface.loc().into(),
                            DiagnosticData::RecursiveInterfaceDefinition {
                                name: super_name.into(),
                            },
                        )
                        .label(Label::new(
                            implements_interface.loc(),
                            format!("interface {super_name} cannot implement itself"),
                        )),
                    );
                }
            }
        }
    }

    // Interface Type field validation.
    let field_definitions = collect_nodes(
        interface_def.self_fields(),
        interface_def.extensions(),
        hir::InterfaceTypeExtension::fields,
    );
    diagnostics.extend(db.validate_field_definitions(field_definitions));

    // Implements Interfaceds validation.
    let implements_interfaces = collect_nodes(
        interface_def.self_implements_interfaces(),
        interface_def.extensions(),
        hir::InterfaceTypeExtension::implements_interfaces,
    );
    diagnostics.extend(
        db.validate_implements_interfaces(interface_def.name().to_string(), implements_interfaces),
    );

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    let fields: HashSet<ValidationSet> = interface_def
        .fields()
        .map(|field| ValidationSet {
            name: field.name().into(),
            loc: field.loc(),
        })
        .collect();
    for implements_interface in interface_def.implements_interfaces() {
        if let Some(super_interface) = implements_interface.interface_definition(db.upcast()) {
            let implements_interface_fields: HashSet<ValidationSet> = super_interface
                .fields()
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
                        interface_def.loc().into(),
                        DiagnosticData::MissingField {
                            field: name.clone(),
                        },
                    )
                    .labels([
                        Label::new(
                            super_interface.loc(),
                            format!("`{name}` was originally defined here"),
                        ),
                        Label::new(
                            interface_def.loc(),
                            format!("add `{name}` field to this interface"),
                        ),
                    ])
                    .help("An interface must be a super-set of all interfaces it implements"),
                );
            }
        }
    }

    diagnostics
}

pub fn validate_implements_interfaces(
    db: &dyn ValidationDatabase,
    implementor_name: String,
    impl_interfaces: Vec<ImplementsInterface>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let interfaces = db.interfaces();
    let defined_interfaces: HashSet<ValidationSet> = interfaces
        .iter()
        .map(|(name, interface)| ValidationSet {
            name: name.to_owned(),
            loc: Some(interface.loc()),
        })
        .collect();

    // Implements Interfaces must be defined.
    //
    // Returns Undefined Definition error.
    let implements_interfaces: HashSet<ValidationSet> = impl_interfaces
        .iter()
        .map(|interface| ValidationSet {
            name: interface.interface().to_owned(),
            loc: Some(interface.loc()),
        })
        .collect();
    let diff = implements_interfaces.difference(&defined_interfaces);
    for undefined in diff {
        // undefined.loc should always be Some
        let loc = undefined
            .loc
            .expect("missing implements interface location");
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                loc.into(),
                DiagnosticData::UndefinedDefinition {
                    name: undefined.name.clone(),
                },
            )
            .label(Label::new(loc, "not found in this scope")),
        );
    }

    // Transitively implemented interfaces must be defined on an implementing
    // type or interface.
    //
    // Returns Transitive Implemented Interfaces error.
    let transitive_interfaces: HashSet<ValidationSet> = impl_interfaces
        .iter()
        .filter_map(|implements_interface| {
            if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
                let child_interfaces: HashSet<ValidationSet> = interface
                    .self_implements_interfaces()
                    .iter()
                    .map(|interface| ValidationSet {
                        name: interface.interface().to_owned(),
                        loc: Some(implements_interface.loc()),
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
        // undefined.loc is always be Some
        let loc = undefined
            .loc
            .expect("missing implements interface location");
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                loc.into(),
                DiagnosticData::TransitiveImplementedInterfaces {
                    missing_interface: undefined.name.clone(),
                },
            )
            .label(Label::new(
                loc,
                format!("{} must also be implemented here", undefined.name),
            )),
        );
    }

    let mut seen = HashMap::<&str, &ImplementsInterface>::new();
    for impl_interface in &impl_interfaces {
        let name = impl_interface.interface();
        if let Some(original) = seen.get(&name) {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    impl_interface.loc().into(),
                    DiagnosticData::DuplicateImplementsInterface {
                        ty: implementor_name.clone(),
                        interface: name.to_string(),
                    },
                )
                .label(Label::new(
                    original.loc(),
                    format!("`{name}` interface implementation previously declared here"),
                ))
                .label(Label::new(
                    impl_interface.loc(),
                    format!("`{name}` interface implementation declared again here"),
                )),
            );
        } else {
            seen.insert(name, impl_interface);
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
            println!("{diagnostic}")
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
            println!("{diagnostic}")
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
            println!("{diagnostic}")
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
            println!("{diagnostic}")
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
            println!("{diagnostic}")
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_generates_diagnostics_for_non_output_field_types() {
        let input = r#"
query mainPage {
  name {
    width
  }
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
            println!("{diagnostic}")
        }
        assert_eq!(diagnostics.len(), 3);
    }
}
