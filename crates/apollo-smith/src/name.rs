use std::fmt::Write as _;

use arbitrary::Result;

use crate::DocumentBuilder;

const CHARSET_LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_";
const CHARSET_NUMBERS: &[u8] = b"0123456789";
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

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::Name> for Name {
    fn from(name: apollo_parser::ast::Name) -> Self {
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

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `Name`
    pub fn name(&mut self) -> Result<Name> {
        Ok(Name::new(self.limited_string(30)?))
    }

    /// Create an arbitrary type `Name`
    pub fn type_name(&mut self) -> Result<Name> {
        let mut new_name = self.limited_string(30)?;
        if self.list_existing_type_names().any(|n| n.name == new_name) {
            let _ = write!(
                new_name,
                "{}",
                self.object_type_defs.len() + self.enum_type_defs.len() + self.directive_defs.len()
            );
        }
        Ok(Name::new(new_name))
    }

    /// Create an arbitrary `Name` with an index included in the name (to avoid name conflict)
    pub fn name_with_index(&mut self, index: usize) -> Result<Name> {
        let mut name = self.limited_string(30)?;
        let _ = write!(name, "{}", index);

        Ok(Name::new(name))
    }

    // Mirror what happens in `Arbitrary for String`, but do so with a clamped size.
    pub(crate) fn limited_string(&mut self, max_size: usize) -> Result<String> {
        loop {
            let size = self.u.int_in_range(0..=max_size)?;

            let gen_str = String::from_utf8(
                (0..size)
                    .map(|curr_idx| {
                        let idx = self.u.arbitrary::<usize>()?;

                        // Cannot start with a number
                        let ch = if curr_idx == 0 {
                            // len - 1 to not have a _ at the begining
                            CHARSET_LETTERS[idx % (CHARSET_LETTERS.len() - 1)]
                        } else {
                            let idx = idx % (CHARSET_LETTERS.len() + CHARSET_NUMBERS.len());
                            if idx < CHARSET_LETTERS.len() {
                                CHARSET_LETTERS[idx]
                            } else {
                                CHARSET_NUMBERS[idx - CHARSET_LETTERS.len()]
                            }
                        };

                        Ok(ch)
                    })
                    .collect::<Result<Vec<u8>>>()?,
            )
            .unwrap();
            let new_gen = gen_str.trim_end_matches('_');
            if !new_gen.is_empty() && !RESERVED_KEYWORDS.contains(&new_gen) {
                break Ok(new_gen.to_string());
            }
        }
    }

    fn list_existing_type_names(&self) -> impl Iterator<Item = &Name> {
        self.object_type_defs
            .iter()
            .map(|o| &o.name)
            .chain(self.interface_type_defs.iter().map(|itf| &itf.name))
            .chain(self.enum_type_defs.iter().map(|itf| &itf.name))
            .chain(self.directive_defs.iter().map(|itf| &itf.name))
            .chain(self.union_type_defs.iter().map(|itf| &itf.name))
            .chain(self.input_object_type_defs.iter().map(|itf| &itf.name))
            .chain(self.scalar_type_defs.iter().map(|itf| &itf.name))
            .chain(self.directive_defs.iter().map(|itf| &itf.name))
            .chain(self.fragment_defs.iter().map(|itf| &itf.name))
            .chain(self.operation_defs.iter().filter_map(|op| op.name.as_ref()))
    }
}
