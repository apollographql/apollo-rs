use std::fmt::{self, Write};

fn write_character(c: char, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match c {
        // '"' => f.write_str(r#"\""#),
        '\n' => f.write_str(r#"\n"#),
        '\r' => f.write_str(r#"\r"#),
        '\t' => f.write_str(r#"\t"#),
        '\\' => f.write_str(r#"\\"#),
        c if c.is_control() => write!(f, "{}", c.escape_unicode()),
        c => write!(f, "{c}"),
    }
}

/// Format a string as a """block string""".
#[derive(Debug)]
struct BlockStringFormatter<'a> {
    string: &'a str,
    /// Indentation for the whole block string: it expects to be printed
    /// on its own line, and the caller is responsible for ensuring that.
    indent: usize,
}
impl fmt::Display for BlockStringFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:indent$}\"\"\"", "", indent = self.indent)?;

        let use_single_line =
            // Only one line of content
            self.string.lines().nth(1).is_none()
            // Should not end with a character that would change the meaning of the end quotes """
            && !self.string.ends_with(['"', '\\']);

        if use_single_line {
            for c in self.string.chars() {
                write_character(c, f)?;
            }
            f.write_str(r#"""""#)?;
        } else {
            for line in self.string.lines() {
                write!(f, "\n{:indent$}", "", indent = self.indent)?;
                for c in line.chars() {
                    write_character(c, f)?;
                }
            }
            write!(f, "\n{:indent$}\"\"\"", "", indent = self.indent)?;
        }
        Ok(())
    }
}

/// Format a string, handling escape sequences.
#[derive(Debug)]
struct StringFormatter<'a>(&'a str);
impl fmt::Display for StringFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for c in self.0.chars() {
            if matches!(c, '"' | '\\' | '\n' | '\t' | '\r') {
                f.write_char('\\')?;
            }
            f.write_char(c)?;
        }
        f.write_char('"')?;
        Ok(())
    }
}

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
        let source = match self {
            StringValue::Top { source }
            | StringValue::Field { source }
            | StringValue::Input { source } => source,
        };
        // TODO(@goto-bus-stop): We could instead pass in the indentation as a
        // fmt parameter whenever a StringValue is printed.
        let indent = match self {
            StringValue::Top { .. } => 0,
            StringValue::Field { .. } => 2,
            StringValue::Input { .. } => 4,
        };

        if should_use_block_string(source) {
            write!(
                f,
                "{}",
                BlockStringFormatter {
                    string: source,
                    indent,
                }
            )
        } else {
            // TODO(@goto-bus-stop) We should probably not prepend the indentation here
            // but let the caller handle it
            write!(
                f,
                "{:indent$}{string}",
                "",
                indent = indent,
                string = StringFormatter(source),
            )
        }
    }
}

/// For multi-line strings and strings containing ", use a block string.
fn should_use_block_string(s: &str) -> bool {
    s.contains(['"', '\n', '\r'])
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
            r#""Favourite cat nap spots include: plant corner, pile of clothes.""#
        );
    }

    #[test]
    fn it_encodes_description_with_quotations() {
        let desc = StringValue::Top {
            source: r#""Favourite "cat" nap spots include: plant corner, pile of clothes.""#
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""""
"Favourite "cat" nap spots include: plant corner, pile of clothes."
""""#
        );
    }

    #[test]
    fn it_encodes_description_with_other_languages() {
        let desc = StringValue::Top {
            source: r#"котя(猫, ねこ, قطة) любить дрімати в "кутку" з рослинами"#.to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#""""котя(猫, ねこ, قطة) любить дрімати в "кутку" з рослинами""""#
        );
    }

    #[test]
    fn it_encodes_description_with_new_line() {
        let desc = StringValue::Top {
            source: "Favourite cat nap spots include:\nplant corner, pile of clothes.".to_string(),
        };

        println!("{desc}");
        assert_eq!(
            desc.to_string(),
            r#""""
Favourite cat nap spots include:
plant corner, pile of clothes.
""""#
        );
    }

    #[test]
    fn it_encodes_description_with_carriage_return() {
        let desc = StringValue::Top {
            source: "Favourite cat nap spots include:\rplant corner,\rpile of clothes.".to_string(),
        };

        assert_eq!(
            desc.to_string(),
            "\"\"\"\nFavourite cat nap spots include:\rplant corner,\rpile of clothes.\n\"\"\""
        );
    }

    #[test]
    fn it_encodes_indented_desciption() {
        let desc = StringValue::Field {
            source: "Favourite cat nap spots include:\r  plant corner,\r  pile of clothes."
                .to_string(),
        };

        assert_eq!(
            dbg!(desc.to_string()),
            String::from(
                "  \"\"\"\n  Favourite cat nap spots include:\r  plant corner,\r  pile of clothes.\n  \"\"\""
            )
        );
    }

    #[test]
    fn it_encodes_ends_with_quote() {
        let source = r#"ends with ""#.to_string();

        let desc = StringValue::Top {
            source: source.clone(),
        };
        assert_eq!(
            desc.to_string(),
            r#""""
ends with "
""""#
        );

        let desc = StringValue::Field {
            source: source.clone(),
        };
        assert_eq!(
            desc.to_string(),
            r#"  """
  ends with "
  """"#
        );

        let desc = StringValue::Input { source };
        assert_eq!(
            desc.to_string(),
            r#"    """
    ends with "
    """"#
        );
    }
}
