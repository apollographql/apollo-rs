use arbitrary::Result;

use crate::next::mutations::{all_mutations, Mutation};
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

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
        let was_applied1 = mutation.apply(u, &mut doc)?;
        let (valid_mutation, was_applied) = (mutation.is_valid(), was_applied1);
        if was_applied {
            match apollo_compiler::Schema::builder().add_ast(&doc).build() {
                Ok(new_schema) if valid_mutation => schema = new_schema,
                Ok(_new_schema) => {
                    panic!("valid schema returned from invalid mutation")
                }
                Err(_new_schema) if valid_mutation => {
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
