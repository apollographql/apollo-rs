use std::collections::HashSet;

use apollo_parser::ast::{self, ImplementsInterfaces};

pub fn check(doc: &ast::Document) -> (HashSet<String>, HashSet<String>) {
    let defined_interfaces: HashSet<String> = doc
        .definitions()
        .filter_map(|def| {
            if let ast::Definition::InterfaceTypeDefinition(interface_def) = def {
                interface_def.name()?.text().to_string();
            }
            None
        })
        .collect();

    let implements_interfaces: HashSet<String> =
        doc.definitions().fold(HashSet::new(), |mut set, def| {
            let extend = |set: &mut HashSet<String>, interface: Option<ImplementsInterfaces>| {
                if let Some(interface) = interface {
                    let named_types = interface.named_types();
                    let iter = named_types.filter_map(|n| Some(n.name()?.to_string()));
                    set.extend(iter);
                }
            };
            match def {
                ast::Definition::ObjectTypeDefinition(def) => {
                    extend(&mut set, def.implements_interfaces());
                }
                ast::Definition::ObjectTypeExtension(def) => {
                    extend(&mut set, def.implements_interfaces());
                }
                ast::Definition::InterfaceTypeExtension(def) => {
                    extend(&mut set, def.implements_interfaces());
                }
                ast::Definition::InterfaceTypeDefinition(def) => {
                    extend(&mut set, def.implements_interfaces());
                }
                _ => (),
            }
            set
        });

    (implements_interfaces, defined_interfaces)
}
