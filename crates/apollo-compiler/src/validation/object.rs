use std::{collections::HashSet, sync::Arc};

use crate::{
    diagnostics::{MissingField, UniqueDefinition},
    hir,
    validation::{ast_type_definitions, ValidationSet},
    ApolloDiagnostic, ValidationDatabase,
};
use apollo_parser::ast;

pub fn validate_object_type_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().objects;
    for def in defs.values() {
        diagnostics.extend(db.validate_object_type_definition(def.clone()))
    }

    diagnostics
}

pub fn validate_object_type_definition(
    db: &dyn ValidationDatabase,
    object: Arc<hir::ObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(
        db.validate_directives(object.directives().to_vec(), hir::DirectiveLocation::Object),
    );

    // Object Type definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let hir = db.object_types();
    for (file_id, ast_def) in ast_type_definitions::<ast::ObjectTypeDefinition>(db) {
        if let Some(name) = ast_def.name() {
            let name = &*name.text();
            let hir_def = &hir[name];
            let ast_loc = (file_id, &ast_def).into();
            if *hir_def.loc() == ast_loc {
                // The HIR node was built from this AST node. This is fine.
            } else {
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    ty: "object type".into(),
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

    // Object Type field validations.
    diagnostics.extend(db.validate_field_definitions(object.fields_definition().to_vec()));

    // Implements Interfaceds validation.
    diagnostics.extend(db.validate_implements_interfaces(object.implements_interfaces().to_vec()));

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
            loc: *field.loc(),
        })
        .collect();
    for implements_interface in object.implements_interfaces().iter() {
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
                let current_offset = object.loc().offset();
                let current_len = object.loc().node_len();

                let super_offset = interface.loc().offset();
                let super_len = interface.loc().node_len();

                diagnostics.push(ApolloDiagnostic::MissingField(MissingField {
                    ty: missing_field.name.clone(),
                    src: db.source_code(object.loc().file_id()),
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
            println!("{diagnostic}")
        }
        assert_eq!(diagnostics.len(), 3);
    }
}
