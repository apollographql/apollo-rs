use crate::DocumentBuilder;
use apollo_compiler::Name;
use arbitrary::Result as ArbitraryResult;
use std::fmt::Write as _;

// First char in a GraphQL name can't be a digit and we don't want it to be
// `_` either. Body chars can be letters, `_`, or digits.
const CHARSET_NAME_HEAD: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
const CHARSET_NAME_BODY: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789";
const RESERVED_KEYWORDS: &[&str] = &[
    "on",
    "Int",
    "Float",
    "String",
    "Boolean",
    "ID",
    "type",
    "enum",
    "union",
    "extend",
    "scalar",
    "directive",
    "query",
    "mutation",
    "subscription",
    "schema",
    "interface",
];

impl DocumentBuilder<'_> {
    /// Create an arbitrary `Name`
    pub fn name(&mut self) -> ArbitraryResult<Name> {
        Name::new(&self.limited_string(30)?).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    /// Create an arbitrary type `Name` that does not yet exist in the document.
    pub fn type_name(&mut self) -> ArbitraryResult<Name> {
        let base = self.limited_string(30)?;
        let mut suffix = 0usize;
        let mut new_name = base.clone();
        while self.used_type_names.contains(new_name.as_str()) {
            new_name.clear();
            let _ = write!(new_name, "{base}{suffix}");
            suffix += 1;
        }
        let name = Name::new(&new_name).map_err(|_| arbitrary::Error::IncorrectFormat)?;
        self.used_type_names.insert(new_name);
        Ok(name)
    }

    /// Create an arbitrary `Name` with an index included in the name (to avoid name conflict)
    pub fn name_with_index(&mut self, index: usize) -> ArbitraryResult<Name> {
        let mut name = self.limited_string(30)?;
        let _ = write!(name, "{index}");

        Name::new(&name).map_err(|_| arbitrary::Error::IncorrectFormat)
    }

    // Mirror what happens in `Arbitrary for String`, but do so with a clamped size.
    pub(crate) fn limited_string(&mut self, max_size: usize) -> ArbitraryResult<String> {
        loop {
            let size = self.u.int_in_range(1..=max_size)?;

            let gen_str = String::from_utf8(
                (0..size)
                    .map(|curr_idx| {
                        // GraphQL names can't start with a digit or `_`.
                        let charset = if curr_idx == 0 {
                            CHARSET_NAME_HEAD
                        } else {
                            CHARSET_NAME_BODY
                        };
                        Ok(*self.u.choose(charset)?)
                    })
                    .collect::<ArbitraryResult<Vec<u8>>>()?,
            )
            .unwrap();
            let new_gen = gen_str.trim_end_matches('_');
            if !new_gen.is_empty() && !RESERVED_KEYWORDS.contains(&new_gen) {
                break Ok(new_gen.to_string());
            }
        }
    }
}
