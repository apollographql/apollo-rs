// NOTE @lrlna: uncomment this once we add interfaces to database.
//
// use std::collections::HashSet;
//
// use apollo_parser::ast::{self, ImplementsInterfaces};
//
// pub fn check(doc: &ast::Document) -> (HashSet<ast::Name>, HashSet<ast::Name>) {
//     let defined_interfaces: HashSet<ast::Name> = doc
//         .definitions()
//         .filter_map(|def| {
//             if let ast::Definition::InterfaceTypeDefinition(interface_def) = def {
//                 interface_def.name()?;
//             }
//             None
//         })
//         .collect();
//
//     let implements_interfaces: HashSet<ast::Name> =
//         doc.definitions().fold(HashSet::new(), |mut set, def| {
//             let extend = |set: &mut HashSet<ast::Name>, interface: Option<ImplementsInterfaces>| {
//                 if let Some(interface) = interface {
//                     let named_types = interface.named_types();
//                     let iter = named_types.filter_map(|n| n.name());
//                     set.extend(iter);
//                 }
//             };
//             match def {
//                 ast::Definition::ObjectTypeDefinition(def) => {
//                     extend(&mut set, def.implements_interfaces());
//                 }
//                 ast::Definition::ObjectTypeExtension(def) => {
//                     extend(&mut set, def.implements_interfaces());
//                 }
//                 ast::Definition::InterfaceTypeExtension(def) => {
//                     extend(&mut set, def.implements_interfaces());
//                 }
//                 ast::Definition::InterfaceTypeDefinition(def) => {
//                     extend(&mut set, def.implements_interfaces());
//                 }
//                 _ => (),
//             }
//             set
//         });
//
//     (implements_interfaces, defined_interfaces)
// }
//
