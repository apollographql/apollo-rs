use crate::DocumentBuilder;
use arbitrary::Result as ArbitraryResult;
use arbitrary::Unstructured;
use std::fmt::Write as _;

const CHARSET: &[u8] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_\n\r\t/$#!.-+='";

impl DocumentBuilder<'_> {
    /// Create an arbitrary description.
    pub fn description(&mut self) -> ArbitraryResult<apollo_compiler::Node<str>> {
        let mut description = limited_string_desc(self.u, 100)?;
        if description.trim_matches('"').is_empty() {
            let _ = write!(description, "{}", self.u.arbitrary::<usize>()?);
        }
        Ok(description.into())
    }

    pub(crate) fn optional_description(
        &mut self,
    ) -> ArbitraryResult<Option<apollo_compiler::Node<str>>> {
        if self.u.ratio(7, 8)? {
            return Ok(None);
        }
        Ok(Some(self.description()?))
    }
}

fn limited_string_desc(u: &mut Unstructured<'_>, max_size: usize) -> ArbitraryResult<String> {
    let size = u.int_in_range(0..=max_size)?;

    let gen_str = String::from_utf8(
        (0..size)
            .map(|_curr_idx| Ok(*u.choose(CHARSET)?))
            .collect::<ArbitraryResult<Vec<u8>>>()?,
    )
    .unwrap();

    Ok(gen_str)
}
