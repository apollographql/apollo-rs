use std::{collections::HashSet, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::ValidationSet,
    ValidationDatabase,
};

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

    diagnostics.extend(db.validate_directives(
        object.directives().cloned().collect(),
        hir::DirectiveLocation::Object,
    ));

    // Object Type field validations.
    diagnostics.extend(db.validate_field_definitions(object.self_fields().to_vec()));

    // Implements Interfaces validation.
    diagnostics
        .extend(db.validate_implements_interfaces(object.self_implements_interfaces().to_vec()));

    // When defining an interface that implements another interface, the
    // implementing interface must define each field that is specified by
    // the implemented interface.
    //
    // Returns a Missing Field error.
    let fields: HashSet<ValidationSet> = object
        .self_fields()
        .iter()
        .map(|field| ValidationSet {
            name: field.name().into(),
            loc: field.loc(),
        })
        .collect();
    for implements_interface in object.self_implements_interfaces().iter() {
        if let Some(interface) = implements_interface.interface_definition(db.upcast()) {
            let implements_interface_fields: HashSet<ValidationSet> = interface
                .self_fields()
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
                            field: name.to_string(),
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
                    )
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
  result {
    ... on Person {
      name
    }
  }
  entity {
    name
  }
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
