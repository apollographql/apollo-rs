use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, DirectiveLocation},
    FileId, ValidationDatabase,
};
use std::{collections::HashSet, sync::Arc};

/// Given a type definition, find all the types that can be used for fragment spreading.
///
/// Spec: https://spec.graphql.org/October2021/#GetPossibleTypes()
pub fn get_possible_types(
    db: &dyn ValidationDatabase,
    ty: hir::TypeDefinition,
) -> Vec<hir::TypeDefinition> {
    match &ty {
        // 1. If `type` is an object type, return a set containing `type`.
        hir::TypeDefinition::ObjectTypeDefinition(_) => vec![ty],
        // 2. If `type` is an interface type, return the set of types implementing `type`.
        hir::TypeDefinition::InterfaceTypeDefinition(intf) => {
            let mut implementors = db
                .subtype_map()
                .get(intf.name())
                .map(|names| {
                    names
                        .iter()
                        .filter_map(|name| db.find_type_definition_by_name(name.to_string()))
                        .flat_map(|ty| get_possible_types(db, ty))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            implementors.push(ty);
            implementors
        }
        // 3. If `type` is a union type, return the set of possible types of `type`.
        hir::TypeDefinition::UnionTypeDefinition(union_) => {
            let mut members = db
                .subtype_map()
                .get(union_.name())
                .map(|names| {
                    names
                        .iter()
                        .filter_map(|name| db.find_type_definition_by_name(name.to_string()))
                        .flat_map(|ty| get_possible_types(db, ty))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            members.push(ty);
            members
        }
        _ => vec![],
    }
}

pub fn validate_fragment_spread(
    db: &dyn ValidationDatabase,
    spread: Arc<hir::FragmentSpread>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    diagnostics.extend(db.validate_directives(
        spread.directives().to_vec(),
        DirectiveLocation::FragmentSpread,
    ));

    if let Some(fragment) = spread.fragment(db.upcast()) {
        let cond = fragment.type_condition();
        let parent_type = spread.parent_type(db.upcast());
        let cond_type = db.find_type_definition_by_name(cond.to_string());
        if let (Some(parent_type), Some(cond_type)) = (parent_type, cond_type) {
            let concrete_parent_types = db
                .get_possible_types(parent_type.clone())
                .into_iter()
                .map(|ty| ty.name().to_string())
                .collect::<HashSet<_>>();
            let concrete_condition_types = db
                .get_possible_types(cond_type.clone())
                .into_iter()
                .map(|ty| ty.name().to_string())
                .collect::<HashSet<_>>();

            let mut applicable_types =
                concrete_parent_types.intersection(&concrete_condition_types);
            if applicable_types.next().is_none() {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        spread.loc().into(),
                        DiagnosticData::InvalidFragmentSpread {
                            name: Some(spread.name().to_string()),
                            type_name: parent_type.name().to_string(),
                        },
                    )
                    .label(Label::new(
                        spread.loc(),
                        format!("fragment `{}` cannot be applied", spread.name()),
                    ))
                    .label(Label::new(
                        fragment.loc(),
                        format!("fragment declared with type condition `{cond}` here"),
                    ))
                    .label(Label::new(
                        parent_type.loc(),
                        format!("type condition `{cond}` is not assignable to this type"),
                    )),
                );
            }
        }
    } else {
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
