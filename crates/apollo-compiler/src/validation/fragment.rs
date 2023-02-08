use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, DirectiveLocation},
    FileId, ValidationDatabase,
};
use std::sync::Arc;

pub fn validate_fragment_spread(
    db: &dyn ValidationDatabase,
    spread: Arc<hir::FragmentSpread>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_directives(
        spread.directives().to_vec(),
        DirectiveLocation::FragmentSpread,
    ));

    if spread.fragment(db.upcast()).is_none() {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                spread.loc().into(),
                DiagnosticData::UndefinedFragment {
                    name: spread.name().to_string(),
                },
            )
            .labels(vec![Label::new(
                spread.loc(),
                format!("fragment `{}` is not defined", spread.name()),
            )]),
        );
    }

    diagnostics
}

pub fn validate_fragment_definitions(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    for def in db.fragments(file_id).values() {
        diagnostics.extend(db.validate_directives(
            def.directives().to_vec(),
            DirectiveLocation::FragmentDefinition,
        ));

        let fragment_type_def = db.find_type_definition_by_name(def.type_condition().to_string());
        // Make sure the fragment type exists in the schema
        if fragment_type_def.is_some() {
            // TODO handle cases where the type does not support fragments (Enum, Scalar...)
            diagnostics.extend(db.validate_selection_set(def.selection_set().clone()));
        }
    }

    diagnostics
}

// Validate fragment spread type existence
pub fn validate_fragment_spread_type_existence(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // Store existing types in the schema
    let schema_types = db.object_types();

    for def in db.fragments(file_id).values() {
        let type_def = def.type_condition();

        if !schema_types.contains_key(type_def) {}
    }

    diagnostics
}

// Validate fragments on composite types
// pub fn validate_fragments_on_composite_types(

// ) -> Vec<ApolloDiagnostic> {
//     let mut diagnostics = Vec::new();

//     diagnostics
// }

// // Validate fragment is used
// pub fn validate_fragment_is_used(

// ) -> Vec<ApolloDiagnostic> {
//     let mut diagnostics = Vec::new();

//     diagnostics
// }
#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_validates_fields_in_fragment_definitions() {
        let input = r#"
type Query {
  name: String
  topProducts: Product
}

type Product {
  inStock: Boolean
  name: String
}

fragment XY on Product {
  notExistingField
}
"#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 1)
    }

    #[test]
    fn it_fails_validation_with_invalid_fragment_spread_type() {
        let input = r#"
        query ExampleQuery {
          topProducts {
            name
          }
        
          ... on User {
            id
            name
            status(membership: $goldStatus)
          }
        
          ... fragmentOne
          ... invalidInlineFragment
        }
        
        fragment fragmentOne on Query {
            profilePic(size: $dimensions)
        }
        
        fragment invalidInlineFragment on Dog {
          ... on NotInSchema {
            name
          }
        }
        
        type Query {
          topProducts: Product
        }
        
        type Product {
          name: String
          price(setPrice: Int): Int
        }
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 1)
    }
}
