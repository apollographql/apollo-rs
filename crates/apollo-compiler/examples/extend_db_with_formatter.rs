use std::collections::HashMap;

use apollo_compiler::{
    database::{AstStorage, DocumentStorage, HirStorage, InputStorage},
    hir::*,
    AstDatabase, DocumentDatabase, HirDatabase, InputDatabase,
};

pub struct GraphQLFmt {
    pub db: FormatterDatabase,
}

impl GraphQLFmt {
    pub fn new(input: &str) -> Self {
        let mut db = FormatterDatabase::default();
        let input = input.to_string();
        db.set_input(input);
        Self { db }
    }

    pub fn fmt(&self) -> String {
        self.db.fmt()
    }
}

// Includes all the necessary database's storage units that will now be
// accessible from FormatterDatabase.
#[salsa::database(DocumentStorage, InputStorage, AstStorage, HirStorage, FmtStorage)]
#[derive(Default)]
pub struct FormatterDatabase {
    pub storage: salsa::Storage<FormatterDatabase>,
}

impl salsa::Database for FormatterDatabase {}

// This is important if your FormatterDatabase storage needs to be accessed from in
// a multi-threaded environment. You can drop this otherwise.
impl salsa::ParallelDatabase for FormatterDatabase {
    fn snapshot(&self) -> salsa::Snapshot<FormatterDatabase> {
        salsa::Snapshot::new(FormatterDatabase {
            storage: self.storage.snapshot(),
        })
    }
}

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

impl Upcast<dyn DocumentDatabase> for FormatterDatabase {
    fn upcast(&self) -> &(dyn DocumentDatabase + 'static) {
        self
    }
}

#[salsa::query_group(FmtStorage)]
pub trait Formatter:
    Upcast<dyn DocumentDatabase> + InputDatabase + AstDatabase + HirDatabase
{
    // Define any queries that should be part of this database.
    fn fmt(&self) -> String;
    fn fmt_fragment(&self) -> String;
}

fn fmt(db: &dyn Formatter) -> String {
    db.fmt_fragment()
}

fn fmt_fragment(db: &dyn Formatter) -> String {
    let mut seen: HashMap<&[Selection], String> = HashMap::new();
    let mut doc = apollo_encoder::Document::new();

    let ops = db.operations();
    for op in ops.iter() {
        op.selection_set().selection().iter().for_each(|sel| {
            if let Selection::Field(f) = sel {
                let selection = f.selection_set().selection();
                let ty = f.ty(db.upcast()).unwrap().name();
                if !seen.contains_key(selection) {
                    seen.insert(selection, ty);
                }
            }
        })
    }

    if !seen.is_empty() {
        let fragment_name = "newFragment".to_string();
        for fragment in seen.clone() {
            let sel_set = sel_set(fragment.0);
            let frag = apollo_encoder::FragmentDefinition::new(
                fragment_name.clone(),
                apollo_encoder::TypeCondition::new(fragment.1),
                sel_set,
            );
            doc.fragment(frag)
        }
        for operation in ops.iter() {
            let selections: Vec<apollo_encoder::Selection> = operation
                .selection_set()
                .selection()
                .iter()
                .filter_map(|sel| match sel {
                    Selection::Field(f) => {
                        let selection = f.selection_set().selection();
                        seen.get(&selection).map(|_t| {
                            let mut field = apollo_encoder::Field::new(f.name().to_string());
                            let fragment_spread = apollo_encoder::Selection::FragmentSpread(
                                apollo_encoder::FragmentSpread::new(fragment_name.clone()),
                            );
                            field.selection_set(Some(
                                apollo_encoder::SelectionSet::with_selections(vec![
                                    fragment_spread,
                                ]),
                            ));
                            apollo_encoder::Selection::Field(field)
                        })
                    }
                    _ => None,
                })
                .collect();

            let mut op = apollo_encoder::OperationDefinition::new(
                op_type(operation.operation_ty()),
                apollo_encoder::SelectionSet::with_selections(selections),
            );
            op.name(operation.name().map(str::to_string));

            doc.operation(op)
        }
    }

    doc.to_string()
}

fn op_type(ty: &apollo_compiler::hir::OperationType) -> apollo_encoder::OperationType {
    match ty {
        apollo_compiler::hir::OperationType::Query => apollo_encoder::OperationType::Query,
        apollo_compiler::hir::OperationType::Mutation => apollo_encoder::OperationType::Mutation,
        apollo_compiler::hir::OperationType::Subscription => {
            apollo_encoder::OperationType::Subscription
        }
    }
}

fn sel_set(sel: &[Selection]) -> apollo_encoder::SelectionSet {
    let selections = sel
        .iter()
        .filter_map(|sel| match sel {
            Selection::Field(f) => Some(apollo_encoder::Selection::Field(
                apollo_encoder::Field::new(f.name().to_string()),
            )),
            Selection::FragmentSpread(f) => Some(apollo_encoder::Selection::FragmentSpread(
                apollo_encoder::FragmentSpread::new(f.name().to_string()),
            )),
            _ => None,
        })
        .collect();
    apollo_encoder::SelectionSet::with_selections(selections)
}

fn main() {
    let input = r#"
query FrontPage {
  AW2022 {
  	name
    reviews
  }
  SS2023 {
  	name
    reviews
  }
}

query DropDown {
  jackets {
  	name
    reviews
  }
  jeans {
  	name
    reviews
  }
}

type Query {
  products: [Product]
  AW2022: [Product]
  SS2023: [Product]
  jackets: [Product]
  jeans: [Product]
}

type Product {
  name: String
  reviews: [String]
  price: Int
}
    "#;

    let fmt = GraphQLFmt::new(input);
    let formatted = fmt.fmt();

    println!("{}", formatted);
}
