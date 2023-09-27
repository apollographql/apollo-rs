use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema, ValidationDatabase,
};
use std::collections::HashSet;

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for interface in db.ast_types().interfaces.values() {
        diagnostics.extend(db.validate_interface_definition(interface.clone()));
    }

    diagnostics
}

pub fn validate_interface_definition(
    db: &dyn ValidationDatabase,
    interface: ast::TypeWithExtensions<ast::InterfaceTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    diagnostics.extend(super::directive::validate_directives(
        db,
        interface.directives(),
        ast::DirectiveLocation::Interface,
        // interfaces don't use variables
        Default::default(),
    ));

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
    for implements_interface in interface.implements_interfaces() {
        if *implements_interface == interface.definition.name {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    (implements_interface.location().unwrap()).into(),
                    DiagnosticData::RecursiveInterfaceDefinition {
                        name: implements_interface.to_string(),
                    },
                )
                .label(Label::new(
                    implements_interface.location().unwrap(),
                    format!("interface {implements_interface} cannot implement itself"),
                )),
            );
        }
    }

    // Interface Type field validation.
    let field_definitions = interface.fields().cloned().collect();
    diagnostics.extend(db.validate_field_definitions(field_definitions));

    // Implements Interfaceds validation.
    let implements_interfaces: Vec<_> = interface.implements_interfaces().cloned().collect();
    diagnostics.extend(validate_implements_interfaces(
        db,
        &interface.definition.name,
        &interface.definition.clone().into(),
        &implements_interfaces,
    ));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    let field_names: HashSet<ast::Name> =
        interface.fields().map(|field| field.name.clone()).collect();
    for implements_interface in interface.implements_interfaces() {
        if let Some(schema::ExtendedType::Interface(super_interface)) =
            schema.types.get(implements_interface)
        {
            for super_field in super_interface.fields.values() {
                if field_names.contains(&super_field.name) {
                    continue;
                }
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        (interface.definition.location().unwrap()).into(),
                        DiagnosticData::MissingInterfaceField {
                            interface: implements_interface.to_string(),
                            field: super_field.name.to_string(),
                        },
                    )
                    .labels([
                        Label::new(
                            implements_interface.location().unwrap(),
                            format!(
                                "implementation of interface {implements_interface} declared here"
                            ),
                        ),
                        Label::new(
                            super_field.location().unwrap(),
                            format!(
                                "`{}` was originally defined by {} here",
                                super_field.name, implements_interface
                            ),
                        ),
                        Label::new(
                            interface.definition.location().unwrap(),
                            format!("add `{}` field to this interface", super_field.name),
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
    implementor_name: &ast::Name,
    implementor: &ast::Definition,
    implements_interfaces: &[ast::Name],
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let schema = db.schema();

    let interface_definitions = implements_interfaces
        .iter()
        .filter_map(|name| {
            schema
                .get_interface(name)
                .map(|interface| (name, interface))
        })
        .collect::<Vec<_>>();

    // Implements Interfaces must be defined.
    //
    // Returns Undefined Definition error.
    for interface_name in implements_interfaces {
        if schema.get_interface(interface_name).is_some() {
            continue;
        }

        // interface_name.loc should always be Some
        let loc = interface_name
            .location()
            .expect("missing implements interface location");
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                loc.into(),
                DiagnosticData::UndefinedDefinition {
                    name: interface_name.to_string(),
                },
            )
            .label(Label::new(loc, "not found in this scope")),
        );
    }

    // Transitively implemented interfaces must be defined on an implementing
    // type or interface.
    //
    // Returns Transitive Implemented Interfaces error.
    let transitive_interfaces = interface_definitions.iter().flat_map(|&(name, interface)| {
        interface
            .implements_interfaces
            .iter()
            .map(|component| &component.node)
            .zip(std::iter::repeat(name))
    });
    for (transitive_interface, via_interface) in transitive_interfaces {
        if implements_interfaces.contains(transitive_interface) {
            continue;
        }

        let definition_loc = implementor.location().expect("missing interface location");
        // let via_loc = via_interface
        //     .location()
        //     .expect("missing implements interface location");
        let transitive_loc = transitive_interface
            .location()
            .expect("missing implements interface location");
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                definition_loc.into(),
                DiagnosticData::TransitiveImplementedInterfaces {
                    missing_interface: transitive_interface.to_string(),
                },
            )
            .label(Label::new(
                transitive_loc,
                format!(
                    "implementation of {transitive_interface} declared by {via_interface} here"
                ),
            ))
            .label(Label::new(
                definition_loc,
                format!("{transitive_interface} must also be implemented here"),
            )),
        );
    }

    let mut seen: HashSet<&ast::Name> = HashSet::new();
    for name in implements_interfaces {
        if let Some(original) = seen.get(&name) {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    (name.location().unwrap()).into(),
                    DiagnosticData::DuplicateImplementsInterface {
                        ty: implementor_name.to_string(),
                        interface: name.to_string(),
                    },
                )
                .label(Label::new(
                    original.location().unwrap(),
                    format!("`{name}` interface implementation previously declared here"),
                ))
                .label(Label::new(
                    name.location().unwrap(),
                    format!("`{name}` interface implementation declared again here"),
                )),
            );
        } else {
            seen.insert(name);
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
