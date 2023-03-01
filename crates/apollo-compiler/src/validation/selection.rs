use std::sync::Arc;

use crate::diagnostics::{DiagnosticData, Label};
use crate::hir::TypeDefinition;
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
    let leaf = field.selection_set.selection.is_empty();
    let mut leaf_validation_error: Option<&str> = None;
    let field_type = field.ty(db.upcast());
    if let Some(field_type) = field_type {
        let type_name = field_type.name();
        let type_def = db.find_type_definition_by_name(type_name.clone());
        if let Some(type_def) = type_def {
            match type_def {
                TypeDefinition::EnumTypeDefinition(_) if !leaf => {
                    leaf_validation_error =
                        Some("This field is an enum and cannot select any fields");
                }
                TypeDefinition::ScalarTypeDefinition(_) if !leaf => {
                    leaf_validation_error =
                        Some("This field is a scalar and cannot select any fields");
                }
                TypeDefinition::ObjectTypeDefinition(_) if leaf => {
                    leaf_validation_error = Some("This field is an object and must select fields");
                }
                TypeDefinition::InterfaceTypeDefinition(_) if leaf => {
                    leaf_validation_error =
                        Some("This field is an interface and must select fields");
                }
                TypeDefinition::UnionTypeDefinition(_) if leaf => {
                    leaf_validation_error = Some("This field is an union and must select fields");
                }
                _ => {}
            };
            if let Some(error) = leaf_validation_error {
                let name = field.name.src.clone();
                let diagnostic_data = if leaf {
                    DiagnosticData::MandatorySubselection {
                        name,
                        ty: type_name.clone(),
                    }
                } else {
                    DiagnosticData::NoSubselectionAllowed {
                        name,
                        ty: type_name.clone(),
                    }
                };
                let diagnostic = ApolloDiagnostic::new(db, field.loc.into(), diagnostic_data)
                    .label(Label::new(field.loc, error));
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

    if leaf_validation_error.is_none() {
        diagnostics.extend(db.validate_field(field));
    }
}

pub fn validate_selection_set(
    db: &dyn ValidationDatabase,
    selection_set: hir::SelectionSet,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

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

    #[test]
    fn it_validates_subselections_of_union() {
        let input = r#"
query SelectionOfEnum {
  animal
}

type Query {
  animal: CatOrDog,
}

type Cat { id: String! }
type Dog { id: String! }
union CatOrDog = Cat | Dog
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
