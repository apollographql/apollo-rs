use std::fmt::Write as _;

use arbitrary::{Arbitrary, Result, Unstructured};

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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Arbitrary)]
pub struct Description(StringValue);

impl From<Description> for String {
    fn from(desc: Description) -> Self {
        desc.0.into()
    }
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::Description> for Description {
    fn from(desc: apollo_parser::ast::Description) -> Self {
        Description(
            desc.string_value()
                .map(|s| s.into())
                .unwrap_or_else(|| StringValue::Line(Default::default())),
        )
    }
}

impl From<String> for Description {
    fn from(desc: String) -> Self {
        Description(StringValue::from(desc))
    }
}

/// The `__StringValue` type represents a sequence of characters
///
/// *StringValue*:
///     "string" | """string"""
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Descriptions).
///
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StringValue {
    /// Represents a string value between """
    Block(String),
    /// Represents a one line string value between "
    Line(String),
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::StringValue> for StringValue {
    fn from(val: apollo_parser::ast::StringValue) -> Self {
        Self::from(Into::<String>::into(val))
    }
}

impl From<StringValue> for String {
    fn from(str_value: StringValue) -> Self {
        match str_value {
            StringValue::Block(str_val) => format!(r#""""{str_val}""""#),
            StringValue::Line(str_val) => format!(r#"{str_val}""#),
        }
    }
}

impl From<String> for StringValue {
    fn from(str_value: String) -> Self {
        // TODO check
        if str_value.contains(['"', '\t', '\r', '\n']) {
            return StringValue::Block(str_value);
        }
        StringValue::Line(str_value)
    }
}

impl Arbitrary<'_> for StringValue {
    fn arbitrary(u: &mut arbitrary::Unstructured<'_>) -> Result<Self> {
        let mut arbitrary_str = limited_string_desc(u, 100)?;
        if arbitrary_str.trim_matches('"').is_empty() {
            let _ = write!(arbitrary_str, "{}", u.arbitrary::<usize>()?);
        }
        let variant_idx = u.int_in_range(0..=1usize)?;
        let str_value = match variant_idx {
            0 => Self::Block(arbitrary_str),
            1 => Self::Line(arbitrary_str),
            _ => unreachable!(),
        };

        Ok(str_value)
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `Description`
    pub fn description(&mut self) -> Result<Description> {
        self.u.arbitrary()
    }
}

fn limited_string_desc(u: &mut Unstructured<'_>, max_size: usize) -> Result<String> {
    let size = u.int_in_range(0..=max_size)?;

    let gen_str = String::from_utf8(
        (0..size)
            .map(|_curr_idx| {
                let idx = u.arbitrary::<usize>()?;

                let idx = idx % CHARSET.len();

                Ok(CHARSET[idx])
            })
            .collect::<Result<Vec<u8>>>()?,
    )
    .unwrap();

    Ok(gen_str)
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "parser-impl")]
    #[test]
    fn convert_description_from_parser() {
        use crate::description::Description;
        use apollo_parser::ast::Definition;
        use apollo_parser::Parser;

        let schema = r#"
"Description for the schema"
schema {}
        "#;
        let parser = Parser::new(schema);
        let ast = parser.parse();
        let document = ast.document();
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
