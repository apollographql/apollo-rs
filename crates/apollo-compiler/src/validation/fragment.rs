use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, DirectiveLocation},
    FileId, ValidationDatabase,
};
use std::sync::Arc;

use super::scalar::BUILT_IN_SCALARS;

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
        diagnostics.extend(db.validate_selection_set(def.selection_set().clone()));
        diagnostics.extend(db.validate_fragment_spread_type_existence(file_id));
        diagnostics.extend(db.validate_fragment_on_composite_types(file_id));
        diagnostics.extend(db.validate_fragment_used(file_id));
    }

    diagnostics
}

pub fn validate_fragment_spread_type_existence(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let schema_types = db.types_definitions_by_name();

    let diagnostics: Vec<ApolloDiagnostic> = db
        .fragments(file_id)
        .values()
        .flat_map(|def| {
            let type_cond = def.type_condition();

            let mut fragment_diagnostics = Vec::new();

            if !schema_types.contains_key(type_cond) {
                fragment_diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        def.loc().into(),
                        DiagnosticData::UndefinedDefinition {
                            name: type_cond.into(),
                        },
                    )
                    .label(Label::new(
                        def.loc(),
                        format!("`{type_cond}` is defined here but not declared in the schema"),
                    ))
                    .help(format!(
                        "Fragments must be specified on types that exist in the schema. Consider defining `{type_cond}` in the schema."
                    )),
                );
            }

            for inline_def in def.selection_set().inline_fragments().iter() {
                let inline_type_cond = inline_def.type_condition().unwrap_or_default();

                if !schema_types.contains_key(inline_type_cond) {
                    fragment_diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            inline_def.loc().into(),
                            DiagnosticData::UndefinedDefinition {
                                name: inline_type_cond.into(),
                            },
                        )
                        .label(Label::new(
                            inline_def.loc(),
                            format!(
                                "`{inline_type_cond}` is defined here but not declared in the schema"
                            ),
                        ))
                        .help(format!(
                            "Fragments must be specified on types that exist in the schema. Consider defining `{inline_type_cond}` in the schema."
                        )),
                    );
                }
            }

            fragment_diagnostics
        })
        .collect();

    let operation_inline_fragment_diagnostics = db
        .operations(file_id)
        .iter()
        .flat_map(|operation| {
            if operation.name().is_some() {
                let fields = operation.fields(db.upcast());
                let inline_fragments = fields
                    .iter()
                    .flat_map(|field| field.selection_set().inline_fragments());

                inline_fragments.filter_map(|inline_fragment| {
                    if let Some(inline_type_cond) = inline_fragment.type_condition() {
                        if !schema_types.contains_key(inline_type_cond) {
                            Some(
                                ApolloDiagnostic::new(
                                    db,
                                    inline_fragment.loc().into(),
                                    DiagnosticData::UndefinedDefinition {
                                        name: inline_type_cond.into(),
                                    },
                                )
                                .label(Label::new(
                                    inline_fragment.loc(),
                                    format!("`{inline_type_cond}` is defined here but not declared in the schema"),
                                ))
                                .help(format!("Fragments must be specified on types that exist in the schema. Consider defining `{inline_type_cond}` in the schema."))
                            )
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).collect::<Vec<_>>()
            } else {
                std::iter::empty().collect::<Vec<_>>()
            }
        })
        .collect::<Vec<_>>();

    let mut combined_diagnostics =
        Vec::with_capacity(diagnostics.len() + operation_inline_fragment_diagnostics.len());
    combined_diagnostics.extend(diagnostics);
    combined_diagnostics.extend(operation_inline_fragment_diagnostics);
    combined_diagnostics
}

pub fn validate_fragment_on_composite_types(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut invalid_fragments = std::collections::HashSet::new();
    let mut invalid_inline_fragments = std::collections::HashSet::new();

    let diagnostics: Vec<ApolloDiagnostic> = db
        .fragments(file_id)
        .values()
        .flat_map(|def| {
            let type_cond = def.type_condition();
            let ty_def = db.find_type_definition_by_name(type_cond.to_owned());
            let is_scalar = ty_def.map_or(false, |ty| ty.is_scalar_type_definition());

            let mut fragment_diagnostics = Vec::new();

            if is_scalar {
                let name = def.name();
                if invalid_fragments.insert((name.to_string(), type_cond.to_owned())) {
                    fragment_diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            def.loc().into(),
                            DiagnosticData::InvalidFragmentType {
                                name: name.into(),
                                ty: type_cond.into(),
                            },
                        )
                        .label(Label::new(
                            def.loc(),
                            format!("`{type_cond}` is defined here"),
                        )),
                    );
                }
            }

            for inline_def in def.selection_set().inline_fragments().iter() {
                let inline_type_cond = inline_def.type_condition().unwrap_or_default();
                let inline_ty_def = db.find_type_definition_by_name(inline_type_cond.to_owned());
                let inline_is_scalar =
                    inline_ty_def.map_or(false, |ty| ty.is_scalar_type_definition());

                if inline_is_scalar {
                    let inline_name = def.name().to_string();
                    if invalid_inline_fragments
                        .insert((inline_name.clone(), inline_type_cond.to_owned()))
                    {
                        fragment_diagnostics.push(
                            ApolloDiagnostic::new(
                                db,
                                inline_def.loc().into(),
                                DiagnosticData::InvalidFragmentType {
                                    name: inline_name,
                                    ty: inline_type_cond.into(),
                                },
                            )
                            .label(Label::new(
                                inline_def.loc(),
                                format!("`{inline_type_cond}` is defined here"),
                            )),
                        );
                    }
                }
            }

            fragment_diagnostics
        })
        .collect();

    diagnostics
}

pub fn validate_fragment_used(
    db: &dyn ValidationDatabase,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let fragments = db.fragments(file_id);

    let operations = db.operations(file_id);
    let op_len = operations.len();

    // Fragments must be used within the schema
    //
    // Returns Unused Fragment error.
    if op_len > 0 {
        for op in operations.iter() {
            let fields = op.fields(db.upcast());
            let fragment_spreads = fields
                .iter()
                .flat_map(|f| f.selection_set().fragment_spreads())
                .map(|f| f.name().to_owned())
                .collect::<Vec<_>>();

            for fragment in fragments.values() {
                let name = fragment.name();
                if !fragment_spreads.contains(&name.to_owned()) {
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            fragment.loc().into(),
                            DiagnosticData::UnusedFragment { name: name.into() },
                        )
                        .label(Label::new(
                            fragment.loc(),
                            format!("`{name}` is defined here"),
                        ))
                        .help(format!("Fragment `{name}` must be used in schema")),
                    )
                }
            }
        }
    } else {
        for fragment in fragments.values() {
            let name = fragment.name();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    fragment.loc().into(),
                    DiagnosticData::UnusedFragment { name: name.into() },
                )
                .label(Label::new(
                    fragment.loc(),
                    format!("`{name}` is defined here"),
                ))
                .help(format!("Fragment `{name}` must be used in schema")),
            )
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

        assert_eq!(diagnostics.len(), 2)
    }

    #[test]
    fn it_fails_validation_with_missing_fragment_spread_type() {
        let input = r#"
        query Query {
            interface {
                a
              ...invalidFragment
            }
          }
          type Query {
            interface: Interface
          }
          interface Interface {
            a: String
          }

          fragment invalidFragment on MissingType {
            a
          }
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 2)
    }

    #[test]
    fn it_fails_validation_with_missing_inline_fragment_spread_type() {
        let input = r#"
        query Query {
            interface {
              ...invalidInlineFragment  
              ... on MissingSecondType {
                a
              }
            }
          }
          type Query {
            interface: Interface
          }
          interface Interface {
            a: String
          }

          fragment invalidInlineFragment on Interface {
            ... on MissingType {
                a
            }
          }
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 2)
    }

    #[test]
    fn it_validates_fragments_on_composite_types() {
        let input = r#"
        query Query {
            products {
                inStock
                name
                ...fragOnScalar
                ...inlineFragOnScalar
                ... on Int {
                    name
                }
            }
        }
        type Query {
            products: Product
        }
        type Product {
            inStock: Boolean
            name: String
        }

        fragment fragOnScalar on Int {
            name
        }

        fragment inlineFragOnScalar on Product {
            ... on Int {
                name
            }
            name
        }
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert_eq!(diagnostics.len(), 3)
    }

    #[test]
    fn it_validates_fragment_is_used() {
        let input = r#"
        query Query {
            products {
                inStock
                name
            }
        }
        type Query {
            products: Product
        }
        type Product {
            inStock: Boolean
            name: String
        }
        fragment nameFragment on Product {
            name
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
