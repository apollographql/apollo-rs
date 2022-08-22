#[salsa::query_group(InputsStorage)]
pub trait Inputs {
    #[salsa::input]
    fn input(&self, name: String) -> String;
}
