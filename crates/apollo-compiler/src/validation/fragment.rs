use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{DirectiveLocation, FragmentDefinition, HirNodeLocation, TypeDefinition},
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
        diagnostics.extend(db.validate_fragment_spreads(
            Some(def.type_condition().to_string()),
            def.loc(),
            def.type_def(db.upcast()).clone(),
        ));
        diagnostics.extend(db.validate_selection_set(def.selection_set().clone()));
        diagnostics.extend(db.validate_fragment_used(def.as_ref().clone(), file_id));
    }

    diagnostics
}

pub fn validate_fragment_spreads(
    db: &dyn ValidationDatabase,
    type_cond: Option<String>,
    loc: HirNodeLocation,
    type_def: Option<TypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    if let Err(diagnostic) = db.validate_fragment_spread_type_existence(type_cond.clone(), loc) {
        diagnostics.push(diagnostic)
    };
    if let Err(diagnostic) = db.validate_fragment_on_composite_types(type_cond, loc, type_def) {
        diagnostics.push(diagnostic)
    };

    diagnostics
}

pub fn validate_fragment_spread_type_existence(
    db: &dyn ValidationDatabase,
    type_cond: Option<String>,
    loc: HirNodeLocation,
) -> Result<(), ApolloDiagnostic> {
    let schema_types = db.types_definitions_by_name();

    match type_cond {
        Some(type_cond) => {
            if !schema_types.contains_key(&type_cond) {
                return Err(ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::InvalidFragment {
                        ty: type_cond.clone().into(),
                    },
                )
                .label(Label::new(
                    loc,
                    format!(
                        "`{}` is defined here but not declared in the schema",
                        &type_cond
                    ),
                ))
                .help("fragments must be specified on types that exist in the schema".to_string())
                .help(format!("consider defining `{}` in the schema", &type_cond)));
            }
        }
        None => {
            return Err(ApolloDiagnostic::new(
                db,
                loc.into(),
                DiagnosticData::InvalidFragment { ty: None },
            )
            .label(Label::new(
                loc,
                "fragment target is defined here but not declared in the schema".to_string(),
            )));
        }
    }
    Ok(())
}

pub fn validate_fragment_on_composite_types(
    db: &dyn ValidationDatabase,
    type_cond: Option<String>,
    loc: HirNodeLocation,
    type_def: Option<TypeDefinition>,
) -> Result<(), ApolloDiagnostic> {
    let is_scalar = type_def.map_or(false, |ty| ty.is_scalar_type_definition());

    match type_cond {
        Some(type_cond) => {
            if is_scalar {
                return Err(ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::InvalidFragmentTarget {
                        ty: type_cond.clone(),
                    },
                )
                .label(Label::new(loc, format!("`{type_cond}` is defined here"))));
            }
            Ok(())
        }
        None => Ok(()),
    }
}

pub fn validate_fragment_used(
    db: &dyn ValidationDatabase,
    def: FragmentDefinition,
    file_id: FileId,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let operations = db.operations(file_id);
    let name = def.name();

    // Fragments must be used within the schema
    //
    // Returns Unused Fragment error.
    if operations.len() < 1 {
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                def.loc().into(),
                DiagnosticData::UnusedFragment { name: name.into() },
            )
            .label(Label::new(def.loc(), format!("`{name}` is defined here"))),
        )
    }

    for op in operations.iter() {
        let fields = op.fields(db.upcast());
        let fragment_spreads: Vec<String> = fields
            .iter()
            .flat_map(|f| f.selection_set().fragment_spreads())
            .map(|f| f.name().to_owned())
            .collect();

        if !fragment_spreads.contains(&name.to_string()) {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    def.loc().into(),
                    DiagnosticData::UnusedFragment { name: name.into() },
                )
                .label(Label::new(def.loc(), format!("`{name}` is defined here")))
                .help(format!("fragment `{name}` must be used in an operation")),
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
            a
            ... on MissingType {
                a
                ... on AnotherType {
                    a
                }
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

    #[test]
    fn it_validates_fragment_is_used_in_nested_fragments() {
        let input = r#"
query IntrospectionQuery {
  foo {
    ...Bar
    baz {
      ...Quux
    }
  }
}

fragment Bar on Foo {
  baz {
    ...Quux
  }
}

fragment Quux on Baz {
  id
}

type Query {
  foo: Foo
}

type Foo {
  baz: Baz
}

type Baz {
  id: ID
}
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{diagnostic}");
        }

        assert!(diagnostics.is_empty())
    }
}
