use std::sync::Arc;

use crate::diagnostics::{DiagnosticData, Label};
use crate::{hir, validation::ValidationDatabase, ApolloDiagnostic};

pub fn validate_selection(
    db: &dyn ValidationDatabase,
    selection: Arc<Vec<hir::Selection>>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for sel in selection.iter() {
        match sel {
            hir::Selection::Field(field) => {
                validate_selection_field(db, field.clone(), &mut diagnostics);
            }

            // TODO handle fragment spreads on invalid parent types
            hir::Selection::FragmentSpread(frag) => diagnostics.extend(db.validate_directives(
                frag.directives().to_vec(),
                hir::DirectiveLocation::FragmentSpread,
            )),
            hir::Selection::InlineFragment(inline) => diagnostics.extend(db.validate_directives(
                inline.directives().to_vec(),
                hir::DirectiveLocation::InlineFragment,
            )),
        }
    }

    diagnostics
}

fn validate_selection_field(
    db: &dyn ValidationDatabase,
    field: Arc<hir::Field>,
    diagnostics: &mut Vec<ApolloDiagnostic>,
) {
    let leaf = !field.selection_set.selection.is_empty();
    let mut leaf_validation_error = false;
    if leaf {
        let field_type = field.ty(db.upcast());
        if let Some(field_type) = field_type {
            let type_name = field_type.name();
            let type_def = db.find_type_definition_by_name(type_name.clone());
            if let Some(type_def) = type_def {
                if type_def.is_enum_type_definition() || type_def.is_scalar_type_definition() {
                    leaf_validation_error = true;
                    let name = field.name.src.clone();
                    let label_text = if type_def.is_enum_type_definition() {
                        "This field is an enum and cannot select any fields"
                    } else {
                        "This field is a scalar and cannot select any fields"
                    };
                    let diagnostic = ApolloDiagnostic::new(
                        db,
                        field.loc.into(),
                        DiagnosticData::NoSubselectionAllowed {
                            name,
                            ty: type_name.clone(),
                        },
                    )
                    .label(Label::new(field.loc, label_text));
                    let diagnostic = if let Some(type_def_loc) = type_def.loc() {
                        diagnostic.label(Label::new(
                            type_def_loc,
                            format!("`{}` declared here", &type_name),
                        ))
                    } else {
                        diagnostic
                    };
                    diagnostics.push(diagnostic);
                }
            }
        }
    }

    if !leaf_validation_error {
        diagnostics.extend(db.validate_field(field));
    }
}

pub fn validate_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // TODO handle Unions

    diagnostics.extend(db.validate_selection(selection_set.selection));

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_validates_subselections_of_enums() {
        let input = r#"
query SelectionOfEnum {
  pet {
    name
  }
}

type Query {
  pet: Pet,
}

enum Pet {
    CAT
    DOG
    FOX
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
