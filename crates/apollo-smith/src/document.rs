use apollo_compiler::ast;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;

/// A compiler document generated from arbitrary bytes.
#[derive(Debug, Clone)]
pub struct ArbitraryDocument(pub ast::Document);

impl<'a> Arbitrary<'a> for ArbitraryDocument {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let rest = u.bytes(u.len())?;
        Self::arbitrary_take_rest(Unstructured::new(rest))
    }

    fn arbitrary_take_rest(u: Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut u = Unstructured::new(u.take_rest());
        crate::DocumentBuilder::new(&mut u).build().map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::input;
    use crate::DocumentBuilder;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn arbitrary_documents_match_build_and_use_all_input(data in input()) {
            let built = DocumentBuilder::new(&mut Unstructured::new(&data)).build().ok().map(|doc| doc.to_string());
            let mut u = Unstructured::new(&data);
            let arbitrary = ArbitraryDocument::arbitrary(&mut u).ok().map(|doc| doc.0.to_string());
            prop_assert_eq!(built, arbitrary);
            prop_assert!(u.is_empty());
        }
    }
}
