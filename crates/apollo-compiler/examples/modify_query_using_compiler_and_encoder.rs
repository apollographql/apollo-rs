// use anyhow::Result;
// use apollo_compiler::{ApolloCompiler, HirDatabase};
// use pretty_assertions::assert_eq;
//
// // This example merges the two operation definitions into a single one.
// fn merge_queries() -> Result<apollo_encoder::Document> {
//     let ts = r"#
//     type Query {
//       user: String
//       me: String
//       launches: [Launch]
//     }
//
//     type Launch {
//         launches: LaunchInfo
//     }
//
//     type LaunchInfo {
//         id: Int
//         site: String
//     }
//     #";
//     let executable = r"#
//     query LaunchSite {
//       launches {
//         launches {
//           id
//           site
//         }
//       }
//     }
//
//     query AstronautInfo {
//       user
//       me
//     }
//     #";
//
//     let mut compiler = ApolloCompiler::new();
//     compiler.add_type_system(ts, "ts.graphql");
//     let file_id = compiler.add_executable(executable, "operation.graphql");
//     let diagnostics = compiler.validate();
//     assert_eq!(diagnostics.len(), 0);
//
//     let mut new_query = apollo_encoder::Document::new();
//     let mut sel_set = Vec::new();
//
//     for op in compiler.db.operations(file_id).iter() {
//         let selections: Vec<apollo_encoder::Selection> = op
//             .selection_set()
//             .selection()
//             .iter()
//             .map(|sel| sel.try_into())
//             .collect::<Result<Vec<apollo_encoder::Selection>, _>>()?;
//         sel_set.extend(selections);
//     }
//
//     let op_def = apollo_encoder::OperationDefinition::new(
//         apollo_encoder::OperationType::Query,
//         apollo_encoder::SelectionSet::with_selections(sel_set),
//     );
//
//     new_query.operation(op_def);
//
//     Ok(new_query)
// }
//
// // This example only includes fields without the `@omitted` directive.
// fn omitted_fields() -> Result<apollo_encoder::Document> {
//     let ts = r"#
//     type Query {
//         isbn: String
//         title: String
//         year: String
//         metadata: String
//         reviews: [String]
//         details: ProductDetails
//     }
//
//     type ProductDetails {
//         country: String
//     }
//
//     directive @omitted on FIELD
//     #";
//     let executable = r"#
//     query Products {
//       isbn @omitted
//       title
//       year @omitted
//       metadata @omitted
//       reviews
//       details {
//         ...details
//       }
//     }
//
//     fragment details on ProductDetails {
//         country
//     }
//     #";
//
//     let mut compiler = ApolloCompiler::new();
//     compiler.add_type_system(ts, "ts.graphql");
//     let file_id = compiler.add_executable(executable, "operation.graphql");
//     let diagnostics = compiler.validate();
//     for diag in &diagnostics {
//         println!("{diag}");
//     }
//
//     assert_eq!(diagnostics.len(), 0);
//
//     let mut new_query = apollo_encoder::Document::new();
//
//     for op in compiler.db.operations(file_id).iter() {
//         let mut selection_set = apollo_encoder::SelectionSet::new();
//         for sel in op.selection_set().selection().iter() {
//             match sel {
//                 apollo_compiler::hir::Selection::Field(field) => {
//                     let omit = field.directives().iter().any(|dir| dir.name() == "omitted");
//                     if !omit {
//                         selection_set.selection(apollo_encoder::Selection::Field(
//                             field.as_ref().try_into()?,
//                         ));
//                     }
//                 }
//                 _ => selection_set.selection(sel.try_into()?),
//             }
//         }
//
//         let op_def =
//             apollo_encoder::OperationDefinition::new(op.operation_ty().try_into()?, selection_set);
//         new_query.operation(op_def);
//     }
//
//     for fragment in compiler.db.fragments(file_id).values() {
//         new_query.fragment(fragment.as_ref().try_into()?)
//     }
//     Ok(new_query)
// }
//
// fn main() -> Result<()> {
//     let merged = merge_queries()?;
//     assert_eq!(
//         merged.to_string(),
//         r#"query {
//   launches {
//     launches {
//       id
//       site
//     }
//   }
//   user
//   me
// }
// "#
//     );
//
//     let omitted_fields = omitted_fields()?;
//     assert_eq!(
//         omitted_fields.to_string(),
//         r#"query {
//   title
//   reviews
//   details {
//     ...details
//   }
// }
// fragment details on ProductDetails {
//   country
// }
// "#
//     );
//
//     Ok(())
// }
//

fn main() {
    todo!()
}
