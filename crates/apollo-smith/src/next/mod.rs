use arbitrary::Result;

use crate::next::mutations::{all_mutations, Mutation};
use crate::next::unstructured::Unstructured;
use apollo_compiler::ast::Document;

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

    let mut schema = apollo_compiler::Schema::builder()
        .add_ast(&doc)
        .build()
        .expect("initial document must be valid");
    for _ in 0..1000 {
        let (valid_mutation, was_applied) =
            apply_mutation(&mut Unstructured::new(u, &schema), &mut doc, &mut mutations)?;
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

fn apply_mutation(
    u: &mut Unstructured,
    doc: &mut Document,
    mutations: &mut Vec<Box<dyn Mutation>>,
) -> Result<(bool, bool)> {
    let mutation = &mutations[u.int_in_range(0..=mutations.len() - 1)?];
    let was_applied = mutation.apply(u, doc)?;
    Ok((mutation.is_valid(), was_applied))
}
