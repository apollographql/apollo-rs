use arbitrary::Result;

use crate::next::mutations::all_mutations;
use crate::next::unstructured::Unstructured;

mod document;
mod existing;
mod invalid;
mod mutations;
mod unstructured;
mod valid;

pub fn build_document(u: &mut arbitrary::Unstructured) -> Result<()> {
    let mut doc = apollo_compiler::ast::Document::new();

    let mut mutations = all_mutations();
    mutations.retain(|_| {
        u.arbitrary()
            .expect("fuzzer must be able to generate a bool")
    });
    if mutations.is_empty() {
        return Ok(());
    }

    let mut schema = apollo_compiler::Schema::builder()
        .add_ast(&doc)
        .build()
        .expect("initial document must be valid");
    for _ in 0..1000 {
        let u = &mut Unstructured::new(u, &schema);
        let mutation = u.choose(&mut mutations)?;
        if mutation.apply(u, &mut doc).is_ok() {
            match apollo_compiler::Schema::builder().add_ast(&doc).build() {
                Ok(new_schema) if mutation.is_valid() => schema = new_schema,
                Ok(_new_schema) => {
                    panic!("valid schema returned from invalid mutation")
                }
                Err(_new_schema) if mutation.is_valid() => {
                    panic!("invalid schema returned from valid mutation")
                }
                Err(_new_schema) => {
                    break;
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let mut u = arbitrary::Unstructured::new(&[0; 32]);
        if let Err(e) = super::build_document(&mut u) {
            panic!("error: {:?}", e);
        };
    }
}
