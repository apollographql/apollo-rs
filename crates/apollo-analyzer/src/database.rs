use std::sync::Arc;

#[salsa::query_group(ASTStorage)]
pub trait DatabaseTrait: salsa::Database {
    #[salsa::input]
    fn input_string(&self, key: ()) -> Arc<String>;

    fn length(&self, key: ()) -> usize;
}

fn length(db: &dyn DatabaseTrait, (): ()) -> usize {
    // Read the input string:
    let input_string = db.input_string(());

    // Return its length:
    input_string.len()
}

#[salsa::database(ASTStorage)]
#[derive(Default)]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}
