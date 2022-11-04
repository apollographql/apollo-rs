use apollo_parser::{ast, Parser};

use anyhow::Result;

// This example merges the two operation definitions into a single one.
fn merge_queries() -> Result<apollo_encoder::Document> {
    let query = r"#
    query LaunchSite {
      launches {
        launches {
          id
          site
        }
      }
    }

    query AstronautInfo {
      user
      me
    }
    #";

    let parser = Parser::new(query);
    let ast = parser.parse();
    assert_eq!(ast.errors().len(), 0);

    let doc = ast.document();

    let mut new_query = apollo_encoder::Document::new();
    let mut sel_set = Vec::new();
    for def in doc.definitions() {
        // We want to combine all of our operations into a single one.
        if let ast::Definition::OperationDefinition(op) = def {
            let selections: Vec<apollo_encoder::Selection> = op
                .selection_set()
                .unwrap()
                .selections()
                .map(|sel| sel.try_into())
                .collect::<Result<Vec<apollo_encoder::Selection>, _>>()?;
            sel_set.extend(selections)
        }
    }

    let op_def = apollo_encoder::OperationDefinition::new(
        apollo_encoder::OperationType::Query,
        apollo_encoder::SelectionSet::with_selections(sel_set),
    );
    new_query.operation(op_def);

    Ok(new_query)
}

// This example only includes fields without the `@omitted` directive.
fn omitted_fields() -> Result<apollo_encoder::Document> {
    let query = r"#
    query Products {
      isbn @omitted
      title
      year @omitted
      metadata @omitted
      reviews
      ...details
    }

    fragment details on ProductDetails {
        country
    }

    #";

    let parser = Parser::new(query);
    let ast = parser.parse();
    assert_eq!(ast.errors().len(), 0);

    let doc = ast.document();

    let mut new_query = apollo_encoder::Document::new();
    for def in doc.definitions() {
        if let ast::Definition::OperationDefinition(op) = def {
            let mut selection_set = apollo_encoder::SelectionSet::new();
            for selection in op.selection_set().unwrap().selections() {
                if let ast::Selection::Field(field) = selection {
                    if let Some(dir) = field.directives() {
                        let omit = dir
                            .directives()
                            .any(|dir| dir.name().unwrap().text() == "omitted");
                        if !omit {
                            selection_set.selection(apollo_encoder::Selection::Field(
                                field.clone().try_into()?,
                            ))
                        }
                    } else {
                        selection_set.selection(apollo_encoder::Selection::Field(field.try_into()?))
                    }
                } else {
                    selection_set.selection(selection.try_into()?)
                }
            }
            let op_def = apollo_encoder::OperationDefinition::new(
                op.operation_type().unwrap().try_into()?,
                selection_set,
            );
            new_query.operation(op_def)
        } else if let ast::Definition::FragmentDefinition(fragment) = def {
            new_query.fragment(fragment.try_into()?);
        }
    }

    Ok(new_query)
}

fn main() -> Result<()> {
    let merged = merge_queries()?;
    println!("{}", merged);

    let omitted_fields = omitted_fields()?;
    println!("{}", omitted_fields);

    Ok(())
}
