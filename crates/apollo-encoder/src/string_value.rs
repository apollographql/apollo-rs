use std::fmt;

/// Top-level description.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct TopStringValue {
    /// Description.
    source: Option<String>,
}

/// Field-level description.
/// This description gets additional leading spaces.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct FieldStringValue {
    /// Description.
    source: Option<String>,
}

/// Input-level description.
/// This description get an additional space at the end.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct InputStringValue {
    /// Description.
    source: Option<String>,
}

/// Reason-level description.
/// Like `Input` variant, but without the space.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct ReasonStringValue {
    /// Description.
    source: Option<String>,
}

macro_rules! impl_helpers {
    ($($struct_name: ident,)+) => {
        $(
            impl $struct_name {
                /// Create new StringValue
                pub fn new(source: Option<String>) -> Self {
                    Self {
                        source,
                    }
                }

                /// Return true if source is some
                pub fn is_empty(&self) -> bool {
                    self.source.is_some()
                }
            }
        )+
    };
}

impl_helpers!(TopStringValue, FieldStringValue, InputStringValue, ReasonStringValue,);

impl fmt::Display for TopStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.source {
            if is_block_string_character(&description) {
                writeln!(f, "\"\"\"\n{}\n\"\"\"", description)
            } else {
                writeln!(f, "\"{}\"", description)
            }
        } else {
            write!(f, "")
        }
    }
}

impl fmt::Display for FieldStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.source {
            if is_block_string_character(&description) {
                write!(f, "  \"\"\"")?;
                for line in description.lines() {
                    write!(f, "\n  {}", line)?;
                }
                writeln!(f, "\n  \"\"\"")
            } else {
                writeln!(f, "  \"{}\"", description)
            }
        } else {
            write!(f, "")
        }
    }
}

impl fmt::Display for InputStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.source {
            if is_block_string_character(&description) {
                write!(f, "\"\"\"\n{}\n\"\"\" ", description)
            } else {
                write!(f, "\"{}\" ", description)
            }
        } else {
            write!(f, "")
        }
    }
}

impl fmt::Display for ReasonStringValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(description) = &self.source {
            if is_block_string_character(&description) {
                write!(f, "\n  \"\"\"")?;
                for line in description.lines() {
                    write!(f, "\n  {}", line)?;
                }
                write!(f, "\n  \"\"\"\n  ")
            } else {
                write!(f, " \"{}\"", description)
            }
        } else {
            write!(f, "")
        }
    }
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
        let desc = TopStringValue {
            source: Some(
                "Favourite cat nap spots include: plant corner, pile of clothes.".to_string(),
            ),
        };

        assert_eq!(
            desc.to_string(),
            r#""Favourite cat nap spots include: plant corner, pile of clothes."
"#
        );
    }

    #[test]
    fn it_encodes_description_with_quotations() {
        let desc = TopStringValue {
            source: Some(
                "Favourite \"cat\" nap spots include: plant corner, pile of clothes.".to_string(),
            ),
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
    fn it_encodes_description_with_new_line() {
        let desc = TopStringValue {
            source: Some(
                "Favourite cat nap spots include:\nplant corner, pile of clothes.".to_string(),
            ),
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
        let desc = TopStringValue {
            source: Some(
                "Favourite cat nap spots include:\rplant corner,\rpile of clothes.".to_string(),
            ),
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
        let desc = FieldStringValue {
            source: Some(
                "Favourite cat nap spots include:\r  plant corner,\r  pile of clothes.".to_string(),
            ),
        };

        assert_eq!(
            desc.to_string(),
            String::from(
                "  \"\"\"\n  Favourite cat nap spots include:\r  plant corner,\r  pile of clothes.\n  \"\"\"\n"
            )
        );
    }
}
