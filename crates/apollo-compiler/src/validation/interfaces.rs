use std::collections::{HashMap, HashSet};

use crate::{
    diagnostics::{
        MissingField, RecursiveDefinition, TransitiveImplementedInterfaces, UndefinedDefinition,
        UniqueDefinition, UniqueField,
    },
    validation::ValidationSet,
    values::{FieldDefinition, InterfaceDefinition},
    ApolloDiagnostic, SourceDatabase,
};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Interface definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen: HashMap<&str, &InterfaceDefinition> = HashMap::new();
    for interface in db.interfaces().iter() {
        let name = interface.name();
        if let Some(prev_def) = seen.get(&name) {
            let prev_offset: usize = prev_def.ast_node(db).text_range().start().into();
            let prev_node_len: usize = prev_def.ast_node(db).text_range().len().into();

            let current_offset: usize = interface.ast_node(db).text_range().start().into();
            let current_node_len: usize = interface.ast_node(db).text_range().len().into();
            diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                ty: "interface".into(),
                name: name.into(),
                src: db.input_string(()).to_string(),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!(
                    "`{name}` must only be defined once in this document."
                )),
            }));
        } else {
            seen.insert(name, interface);
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
    for interface_def in db.interfaces().iter() {
        let name = interface_def.name();
        for implements_interface in interface_def.implements_interfaces() {
            if let Some(interface) = implements_interface.interface_definition(db) {
                let i_name = (*interface.name()).to_string();
                if name == i_name {
                    let offset = implements_interface
                        .ast_node(db)
                        .text_range()
                        .start()
                        .into();
                    let len: usize = implements_interface.ast_node(db).text_range().len().into();
                    diagnostics.push(ApolloDiagnostic::RecursiveDefinition(RecursiveDefinition {
                        message: format!("{} interface cannot implement itself", i_name),
                        definition: (offset, len).into(),
                        src: db.input_string(()).to_string(),
                        definition_label: "recursive implements interfaces".into(),
                    }));
                }
            }
        }
    }

    // Fields in an Interface definition must be unique
    //
    // Returns Unique Value error.
    for interface_def in db.interfaces().iter() {
        let mut seen: HashMap<&str, &FieldDefinition> = HashMap::new();

        let fields = interface_def.fields_definition();

        for field in fields {
            let field_name = field.name();
            if let Some(prev_field) = seen.get(&field_name) {
                let prev_offset: usize = prev_field.ast_node(db).text_range().start().into();
                let prev_node_len: usize = prev_field.ast_node(db).text_range().len().into();

                let current_offset: usize = field.ast_node(db).text_range().start().into();
                let current_node_len: usize = field.ast_node(db).text_range().len().into();

                diagnostics.push(ApolloDiagnostic::UniqueField(UniqueField {
                    field: field_name.into(),
                    src: db.input_string(()).to_string(),
                    original_field: (prev_offset, prev_node_len).into(),
                    redefined_field: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "{field_name} field must only be defined once in this interface definition."
                    )),
                }));
            } else {
                seen.insert(field_name, field);
            }
        }
    }

    let interfaces = db.interfaces();
    let defined_interfaces: HashSet<ValidationSet> = interfaces
        .iter()
        .map(|interface| ValidationSet {
            name: interface.name().to_owned(),
            node: interface.ast_node(db),
        })
        .collect();
    for interface_def in interfaces.iter() {
        // Implements Interfaces must be defined.
        //
        // Returns Undefined Definition error.
        let implements_interfaces: HashSet<ValidationSet> = interface_def
            .implements_interfaces()
            .iter()
            .map(|interface| ValidationSet {
                name: interface.interface().to_owned(),
                node: interface.ast_node(db),
            })
            .collect();
        let diff = implements_interfaces.difference(&defined_interfaces);
        for undefined in diff {
            let offset = undefined.node.text_range().start().into();
            let len: usize = undefined.node.text_range().len().into();
            diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                ty: undefined.name.clone(),
                src: db.input_string(()).to_string(),
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
                if let Some(interface) = implements_interface.interface_definition(db) {
                    let child_interfaces: HashSet<ValidationSet> = interface
                        .implements_interfaces()
                        .iter()
                        .map(|interface| ValidationSet {
                            name: interface.interface().to_owned(),
                            node: implements_interface.ast_node(db),
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
            let offset = undefined.node.text_range().start().into();
            let len: usize = undefined.node.text_range().len().into();
            diagnostics.push(ApolloDiagnostic::TransitiveImplementedInterfaces(
                TransitiveImplementedInterfaces {
                    missing_interface: undefined.name.clone(),
                    src: db.input_string(()).to_string(),
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
                node: field.ast_node(db),
            })
            .collect();
        for implements_interface in interface_def.implements_interfaces().iter() {
            if let Some(interface) = implements_interface.interface_definition(db) {
                let implements_interface_fields: HashSet<ValidationSet> = interface
                    .fields_definition()
                    .iter()
                    .map(|field| ValidationSet {
                        name: field.name().into(),
                        node: field.ast_node(db),
                    })
                    .collect();

                let field_diff = implements_interface_fields.difference(&fields);

                for missing_field in field_diff {
                    let current_offset: usize =
                        interface_def.ast_node(db).text_range().start().into();
                    let current_len = interface_def.ast_node(db).text_range().len().into();

                    let super_offset = interface.ast_node(db).text_range().start().into();
                    let super_len: usize = interface.ast_node(db).text_range().len().into();

                    diagnostics.push(ApolloDiagnostic::MissingField(MissingField {
                        ty: missing_field.name.clone(),
                        src: db.input_string(()).to_string(),
                        current_definition: (current_offset, current_len).into(),
                        super_definition: (super_offset, super_len).into(),
                        help: Some(
                            "An interface must be a super-set of all interfaces it implement"
                                .into(),
                        ),
                    }))
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
  name: String
}

interface NamedEntity {
  name: String
  image: URL
  results: [Int]
  name: String
}

"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        assert_eq!(diagnostics.len(), 1);

        for diagnostic in diagnostics {
            println!("{}", diagnostic)
        }
    }

    #[test]
    fn it_fails_validation_with_duplicate_interface_definitions() {
        let input = r#"
type Query implements NamedEntity {
  name: String
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
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        assert_eq!(diagnostics.len(), 1);

        for diagnostic in diagnostics {
            println!("{}", diagnostic)
        }
    }

    #[test]
    fn it_fails_validation_with_recursive_interface_definition() {
        let input = r#"
type Query implements NamedEntity {
  name: String
}

interface NamedEntity implements NamedEntity {
  name: String
  image: URL
  results: [Int]
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        assert_eq!(diagnostics.len(), 1);

        for diagnostic in diagnostics {
            println!("{}", diagnostic)
        }
    }

    #[test]
    fn it_fails_validation_with_undefined_interface_definition() {
        let input = r#"
interface NamedEntity implements NewEntity {
  name: String
  image: URL
  results: [Int]
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostic = ctx.validate();
        assert_eq!(diagnostic.len(), 1);

        for diagnostic in diagnostic {
            println!("{}", diagnostic)
        }
    }

    #[test]
    fn it_fails_validation_with_missing_transitive_interface() {
        let input = r#"
type Query implements Node {
    ud: ID!
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
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        assert_eq!(diagnostics.len(), 1);

        for diagnostic in diagnostics {
            println!("{}", diagnostic)
        }
    }
}
