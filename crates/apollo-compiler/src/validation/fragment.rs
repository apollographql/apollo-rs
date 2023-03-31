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
}
