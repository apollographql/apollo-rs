use std::collections::HashSet;

use crate::{diagnostics::ErrorDiagnostic, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut errors = Vec::new();

    // Interface definitions must have unique names.
    //
    // Return a Unique Definition error in case of a duplicate name.
    let mut seen = HashSet::new();
    for interface in db.interfaces().iter() {
        let name = interface.name();
        if seen.contains(name) {
            errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueDefinition {
                message: "Operation Definitions must have unique names".into(),
                definition: name.to_string(),
            }));
        } else {
            seen.insert(name);
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
                if name == &(*interface.name()) {
                    errors.push(ApolloDiagnostic::Error(
                        ErrorDiagnostic::RecursiveDefinition {
                            message: "Interface cannot implement itself".into(),
                            definition: (*interface.name()).to_string(),
                        },
                    ));
                }
            }
        }
    }

    // Fields in an Interface definition must be unique
    for interface_def in db.interfaces().iter() {
        let mut seen = HashSet::new();

        let fields = interface_def.fields_definition();

        for field in fields {
            let field_name = field.name();
            if seen.contains(&field_name) {
                errors.push(ApolloDiagnostic::Error(ErrorDiagnostic::UniqueValue {
                    message: "Fields must be unique".into(),
                    value: field_name.into(),
                }));
            } else {
                seen.insert(field_name);
            }
        }
    }

    let interfaces = db.interfaces();
    let defined_interfaces: HashSet<String> = interfaces
        .iter()
        .map(|interface| interface.name().into())
        .collect();
    for interface_def in interfaces.iter() {
        // Implements Interfaces must be defined.
        //
        // Returns Undefined Definition error.
        let implements_interfaces: HashSet<String> = interface_def
            .implements_interfaces()
            .iter()
            .map(|interface| interface.interface().into())
            .collect();
        let diff = implements_interfaces.difference(&defined_interfaces);
        for undefined_interface in diff {
            errors.push(ApolloDiagnostic::Error(
                ErrorDiagnostic::UndefinedDefinition {
                    message: "Implements Interface must be defined".into(),
                    definition: undefined_interface.into(),
                },
            ))
        }

        // Transitively implemented interfaces must be defined on an implementing
        // type or interface.
        //
        // Returns Transitive Implemented Interfaces error.
        let transitive_interfaces: HashSet<String> = interface_def
            .implements_interfaces()
            .iter()
            .filter_map(|implements_interface| {
                if let Some(interface) = implements_interface.interface_definition(db) {
                    let child_interfaces: HashSet<String> = interface
                        .implements_interfaces()
                        .iter()
                        .map(|interface| interface.interface().into())
                        .collect();
                    Some(child_interfaces)
                } else {
                    None
                }
            })
            .flatten()
            .collect();
        let transitive_diff = transitive_interfaces.difference(&implements_interfaces);
        for undefined_interface in transitive_diff {
            errors.push(ApolloDiagnostic::Error(
                ErrorDiagnostic::TransitiveImplementedInterfaces {
                    message: "Transitively implemented interfaces must also be defined on an implementing interface.".into(),
                    interface: interface_def.name().into(),
                    missing_implemented_interface: undefined_interface.into(),
                },
            ))
        }
    }

    errors
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_cyclic_implements_interfaces() {
        let input = r#"
type Query implements NamedEntity {
  name: String
}

interface NamedEntity implements NamedEntity {
  name: String
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_duplicate_fields() {
        let input = r#"
type Query implements NamedEntity {
  name: String
}

interface NamedEntity {
  name: String
  name: String
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_transitive_implemented_interfaces() {
        let input = r#"
type Query implements Node {
    ud: ID!
}

interface Node {
  id: ID!
}

interface Resource implements Node {
  id: ID!
  url: String
}

interface Image implements Resource {
  id: ID!
  url: String
  thumbnail: String
}
"#;
        let ctx = ApolloCompiler::new(input);
        let errors = ctx.validate();
        assert_eq!(errors.len(), 1);
    }
}
