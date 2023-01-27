use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
/// Convenience enum to create a Description. Can be a `Top` level, a `Field`
/// level or an `Input` level. The variants are distinguished by the way they
/// get displayed, e.g. number of leading spaces.
pub enum StringValue {
    /// Top-level description.
    Top {
        /// Description.
        source: String,
    },
    /// Field-level description.
    /// This description gets additional leading spaces.
    Field {
        /// Description.
        source: String,
    },
    /// Input-level description.
    /// This description get an additional space at the end.
    Input {
        /// Description.
        source: String,
    },
}

impl fmt::Display for StringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringValue::Top { source } => {
                if is_block_string_character(source) {
                    writeln!(f, "\"\"\"\n{}\n\"\"\"", trim_double_quotes(source))?
                } else {
                    writeln!(f, "\"{source}\"")?
                }
            }
            StringValue::Field { source } => {
                if is_block_string_character(source) {
                    write!(f, "  \"\"\"")?;
                    let desc = trim_double_quotes(source);
                    for line in desc.lines() {
                        write!(f, "\n  {line}")?;
                    }
                    writeln!(f, "\n  \"\"\"")?;
                } else {
                    writeln!(f, "  \"{source}\"")?
                }
            }
            StringValue::Input { source } => {
                if is_block_string_character(source) {
                    write!(f, "\"\"\"")?;
                    let desc = trim_double_quotes(source);
                    for line in desc.lines() {
                        write!(f, "\n    {line}")?;
                    }
                    write!(f, "\n    \"\"\"")?
                } else {
                    write!(f, "\"{source}\"")?
                }
            }
        }
        write!(f, "")
    }
}

#[allow(clippy::nonminimal_bool)]
fn trim_double_quotes(description: &str) -> String {
    let desc_len = description.len();
    if desc_len < 2 {
        return description.to_string();
    }

    if !description.starts_with('\"') || !description.ends_with('\"') {
        return description.to_string();
    }

    description
        .chars()
        .enumerate()
        .filter_map(|(i, c)| {
            if (i == 0 && c == '"') || (i == desc_len - 1 && c == '"') {
                None
            } else {
                Some(c)
            }
        })
        .collect()
}

fn is_block_string_character(s: &str) -> bool {
    s.contains('\n') || s.contains('"') || s.contains('\r')
}
#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_description_without_block_string_character() {
        let desc = StringValue::Top {
            source: "Favourite cat nap spots include: plant corner, pile of clothes.".to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""Favourite cat nap spots include: plant corner, pile of clothes."
"#
        );
    }

    #[test]
    fn it_encodes_description_with_quotations() {
        let desc = StringValue::Top {
            source: "\"Favourite \"cat\" nap spots include: plant corner, pile of clothes.\""
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""""
Favourite "cat" nap spots include: plant corner, pile of clothes.
"""
"#
        );
    }

    #[test]
    fn it_encodes_description_with_other_languages() {
        let desc = StringValue::Top {
            source: "котя(猫, ねこ, قطة) любить дрімати в \"кутку\" з рослинами".to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""""
котя(猫, ねこ, قطة) любить дрімати в "кутку" з рослинами
"""
"#
        );
    }

    #[test]
    fn it_encodes_description_with_new_line() {
        let desc = StringValue::Top {
            source: "Favourite cat nap spots include:\nplant corner, pile of clothes.".to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""""
Favourite cat nap spots include:
plant corner, pile of clothes.
"""
"#
        );
    }

    #[test]
    fn it_encodes_description_with_carriage_return() {
        let desc = StringValue::Top {
            source: "Favourite cat nap spots include:\rplant corner,\rpile of clothes.".to_string(),
        };

        assert_eq!(
            desc.to_string(),
            String::from(
                "\"\"\"\nFavourite cat nap spots include:\rplant corner,\rpile of clothes.\n\"\"\"\n"
            )
        );
    }

    #[test]
    fn it_encodes_indented_desciption() {
        let desc = StringValue::Field {
            source: "Favourite cat nap spots include:\r  plant corner,\r  pile of clothes."
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            String::from(
                "  \"\"\"\n  Favourite cat nap spots include:\r  plant corner,\r  pile of clothes.\n  \"\"\"\n"
            )
        );
    }
}
