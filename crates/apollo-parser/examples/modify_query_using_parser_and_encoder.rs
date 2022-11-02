use apollo_encoder;
use apollo_parser::{
    ast::{self, AstNode},
    Parser,
};

use anyhow::Result;

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

    let new_query = apollo_encoder::Document::new();
    for def in doc.definitions() {
        // We want to combine all of our operations into a single one.
        if let ast::Definition::OperationDefinition(op) = def {
            let sel_set: ast::SelectionSet = op.selection_set().unwrap();
            let selection_set: apollo_encoder::SelectionSet = sel_set.try_into()?;
        }
    }

    Ok(new_query)
}

fn omitted_fields() -> Result<apollo_encoder::Document> {
    let query = r"#
    query Products{
      isbn @omitted
      title
      year @omitted
      metadata @omitted
      reviews
    }

    #";

    let parser = Parser::new(query);
    let ast = parser.parse();
    assert_eq!(ast.errors().len(), 0);

    let doc = ast.document();

    let new_query = apollo_encoder::Document::new();
    for def in doc.definitions() {
        // We want to combine all of our operations into a single one.
        if let ast::Definition::OperationDefinition(op) = def {
            let selection_set = apollo_encoder::SelectionSet::new();
            for selection in op.selection_set().unwrap().selections() {
                if let ast::Selection::Field(field) = selection {
                    let incl = field
                        .directives()
                        .unwrap()
                        .directives()
                        .into_iter()
                        .filter(|d| d.name().unwrap().source_string() != "omitted");
                    incl.for_each(|f| {
                        selection_set.selection(apollo_encoder::Selection::Field(field.try_into()?))
                    });
                } else {
                    selection_set.selection(selection.try_into()?)
                }
            }
            let op_def = apollo_encoder::OperationDefinition::new(
                op.operation_type().unwrap().try_into()?,
                selection_set,
            );
        }
    }

    Ok(new_query)
}

fn main() -> Result<()> {
    let merged = merge_queries()?;
    println!("{}", merged);

    let omitted_fields = omitted_fields()?;

    Ok(())
}
