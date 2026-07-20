use crate::DocumentBuilder;
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

/// Name is useful to name different elements.
///
/// GraphQL Documents are full of named things: operations, fields, arguments, types, directives, fragments, and variables.
/// All names must follow the same grammatical form.
/// Names in GraphQL are case-sensitive. That is to say name, Name, and NAME all refer to different names.
/// Underscores are significant, which means other_name and othername are two different names
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#Name).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Name {
    pub(crate) name: String,
}

impl From<Name> for String {
    fn from(val: Name) -> Self {
        val.name
    }
}

impl From<Name> for apollo_compiler::Name {
    fn from(value: Name) -> Self {
        (&value).into()
    }
}

impl From<&'_ Name> for apollo_compiler::Name {
    fn from(value: &'_ Name) -> Self {
        // FIXME: falliable instead of unwrap?
        // Names from `DocumentBuilder` do have valid syntax,
        // but the `new` constructor accepts any string
        apollo_compiler::Name::new(&value.name).unwrap()
    }
}

impl From<apollo_parser::cst::Name> for Name {
    fn from(name: apollo_parser::cst::Name) -> Self {
        Self {
            name: name.ident_token().unwrap().to_string(),
        }
    }
}

impl Name {
    pub const fn new(name: String) -> Self {
        Self { name }
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `Name`
    pub fn name(&mut self) -> ArbitraryResult<Name> {
        Ok(Name::new(self.limited_string(30)?))
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
        self.used_type_names.insert(new_name.clone());
        Ok(Name::new(new_name))
    }

    /// Create an arbitrary `Name` with an index included in the name (to avoid name conflict)
    pub fn name_with_index(&mut self, index: usize) -> ArbitraryResult<Name> {
        let mut name = self.limited_string(30)?;
        let _ = write!(name, "{index}");

        Ok(Name::new(name))
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
