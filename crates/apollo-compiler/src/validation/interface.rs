use std::collections::{HashMap, HashSet};

use crate::{
    diagnostics::{
        MissingField, OutputType, RecursiveDefinition, TransitiveImplementedInterfaces,
        UndefinedDefinition, UniqueDefinition, UniqueField,
    },
    hir::{self, FieldDefinition},
    validation::{ast_type_definitions, ValidationSet},
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

pub fn validate_interface_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().interfaces;
    for def in defs.values() {
        diagnostics.extend(
            db.validate_directives(def.directives().to_vec(), hir::DirectiveLocation::Interface),
        );
        db.validate_interface_definition(def.as_ref().clone());
    }

    diagnostics
}

pub fn validate_interface_definition(
    db: &dyn ValidationDatabase,
    interface_def: hir::InterfaceTypeDefinition,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for field in interface_def.fields_definition() {
        diagnostics.extend(db.validate_field_definition(field.clone()));
    }

    // Interface definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.interfaces();
    for (file_id, ast_def) in ast_type_definitions::<ast::InterfaceTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let ast_loc = (file_id, &ast_def).into();
            if *hir_def.loc() == ast_loc {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "interface".into(),
                    name: name.to_owned(),
                    src: db.source_code(hir_def.loc().file_id()),
                    original_definition: hir_def.loc().into(),
                    redefined_definition: ast_loc.into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
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
                let i_name = interface.name();
                if name == i_name {
                    let offset = implements_interface.loc().offset();
                    let len = implements_interface.loc().node_len();
                    diagnostics.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} interface cannot implement itself", i_name),
                        definition: (offset, len).into(),
                        src: db.source_code(implements_interface.loc().file_id()),
                        definition_label: "recursive implements interfaces".into(),
                    }));
                }
            }
        }
    }

    // Interface Type field validations.
    let mut seen: HashMap<&str, &FieldDefinition> = HashMap::new();

    let fields = interface_def.fields_definition();

    for field in fields {
        // Fields in an Interface definition must be unique
        //
        // Returns Unique Value error.
        let field_name = field.name();
        let offset = field.loc().offset();
        let len = field.loc().node_len();

        if let Some(prev_field) = seen.get(&field_name) {
            let prev_offset = prev_field.loc().offset();
            let prev_node_len = prev_field.loc().node_len();

            diagnostics.push(ApolloDiagnostic::UniqueField(UniqueField {
                field: field_name.into(),
                src: db.source_code(prev_field.loc().file_id()),
                original_field: (prev_offset, prev_node_len).into(),
                redefined_field: (offset, len).into(),
                help: Some(format!(
                    "`{field_name}` field must only be defined once in this interface definition."
                )),
            }));
        } else {
            seen.insert(field_name, field);
        }

        // Field types in interface types must be of output type
        if let Some(field_ty) = field.ty().type_def(db.upcast()) {
            if !field.ty().is_output_type(db.upcast()) {
                diagnostics.push(ApolloDiagnostic::OutputType(OutputType {
                    name: field.name().into(),
                    ty: field_ty.kind(),
                    src: db.source_code(field.loc().file_id()),
                    definition: (offset, len).into(),
                }))
            }
        } else if let Some(loc) = field.ty().loc() {
            let field_ty_offset = loc.offset();
            let field_ty_len = loc.node_len();
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: field.ty().name(),
                src: db.source_code(loc.file_id()),
                definition: (field_ty_offset, field_ty_len).into(),
            }))
        } else {
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: field.ty().name(),
                src: db.source_code(field.loc().file_id()),
                definition: (offset, len).into(),
            }))
        }
    }

    let interfaces = db.interfaces();
    let defined_interfaces: HashSet<ValidationSet> = interfaces
        .iter()
        .map(|(name, interface)| ValidationSet {
            name: name.to_owned(),
            loc: *interface.loc(),
        })
        .collect();
    // Implements Interfaces must be defined.
    //
    // Returns Undefined Definition error.
    let implements_interfaces: HashSet<ValidationSet> = interface_def
        .implements_interfaces()
        .iter()
        .map(|interface| ValidationSet {
            name: interface.interface().to_owned(),
            loc: *interface.loc(),
        })
        .collect();
    let diff = implements_interfaces.difference(&defined_interfaces);
    for undefined in diff {
        let offset = undefined.loc.offset();
        let len: usize = undefined.loc.node_len();
        diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
            ty: undefined.name.clone(),
            src: db.source_code(undefined.loc.file_id()),
            definition: (offset, len).into(),
        }))
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
                        loc: *implements_interface.loc(),
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
        let offset = undefined.loc.offset();
        let len = undefined.loc.node_len();
        diagnostics.push(ApolloDiagnostic::TransitiveImplementedInterfaces(
            TransitiveImplementedInterfaces {
                missing_interface: undefined.name.clone(),
                src: db.source_code(undefined.loc.file_id()),
                definition: (offset, len).into(),
            },
        ))
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
            loc: *field.loc(),
        })
        .collect();
    for implements_interface in interface_def.implements_interfaces().iter() {
        if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
            let implements_interface_fields: HashSet<ValidationSet> = interface
                .fields_definition()
                .iter()
                .map(|field| ValidationSet {
                    name: field.name().into(),
                    loc: *field.loc(),
                })
                .collect();

            let field_diff = implements_interface_fields.difference(&fields);

            for missing_field in field_diff {
                let current_offset = interface_def.loc().offset();
                let current_len = interface_def.loc().node_len();

                let super_offset = interface.loc().offset();
                let super_len = interface.loc().node_len();

                diagnostics.push(ApolloDiagnostic::MissingField(MissingField {
                    ty: missing_field.name.clone(),
                    src: db.source_code(interface_def.loc.file_id()),
                    current_definition: (current_offset, current_len).into(),
                    super_definition: (super_offset, super_len).into(),
                    help: Some(
                        "An interface must be a super-set of all interfaces it implement".into(),
                    ),
                }))
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
