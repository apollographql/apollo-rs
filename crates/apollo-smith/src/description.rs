use std::fmt::Write as _;

use arbitrary::{Arbitrary, Result as ArbitraryResult, Unstructured};

use crate::DocumentBuilder;

const CHARSET: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_\n\r\t/$#!.-+='";

/// The `__Description` type represents a description
///
/// *Description*:
///     "string"
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Descriptions).
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Description(String);

impl From<Description> for String {
    fn from(desc: Description) -> Self {
        desc.0
    }
}

impl From<Description> for apollo_compiler::NodeStr {
    fn from(desc: Description) -> Self {
        desc.0.into()
    }
}

impl From<apollo_parser::cst::Description> for Description {
    fn from(desc: apollo_parser::cst::Description) -> Self {
        Description(desc.string_value().map(|s| s.into()).unwrap_or_default())
    }
}

impl From<String> for Description {
    fn from(desc: String) -> Self {
        Description(desc)
    }
}

impl Arbitrary<'_> for Description {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> ArbitraryResult<Self> {
        let mut arbitrary_str = limited_string_desc(u, 100)?;
        if arbitrary_str.trim_matches('"').is_empty() {
            let _ = write!(arbitrary_str, "{}", u.arbitrary::<usize>()?);
        }
        Ok(Self(arbitrary_str))
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `Description`
    pub fn description(&mut self) -> ArbitraryResult<Description> {
        self.u.arbitrary()
    }
}

fn limited_string_desc(u: &mut Unstructured<'_>, max_size: usize) -> ArbitraryResult<String> {
    let size = u.int_in_range(0..=max_size)?;

    let gen_str = String::from_utf8(
        (0..size)
            .map(|_curr_idx| {
                let idx = u.arbitrary::<usize>()?;

                let idx = idx % CHARSET.len();

                Ok(CHARSET[idx])
            })
            .collect::<ArbitraryResult<Vec<u8>>>()?,
    )
    .unwrap();

    Ok(gen_str)
}

#[cfg(test)]
mod tests {

    #[test]
    fn convert_description_from_parser() {
        use crate::description::Description;
        use apollo_parser::cst::Definition;
        use apollo_parser::Parser;

        let schema = r#"
"Description for the schema"
schema {}
        "#;
        let parser = Parser::new(schema);
        let cst = parser.parse();
        let document = cst.document();
        if let Definition::SchemaDefinition(def) = document.definitions().next().unwrap() {
            let parser_description = def.description().unwrap();
            let smith_description = Description::from(parser_description);
            assert_eq!(
                smith_description,
                Description::from("Description for the schema".to_string())
            );
        } else {
            unreachable!();
        }
    }
}
