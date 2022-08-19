#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn document(&self, name: String) -> String;
}
