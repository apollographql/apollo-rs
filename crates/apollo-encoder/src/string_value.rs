use std::fmt::{self, Write};

/// Write and optionally escape a character inside a GraphQL string value.
fn write_character(c: char, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match c {
        '"' => f.write_str(r#"\""#),
        '\u{0008}' => f.write_str(r#"\b"#),
        '\u{000c}' => f.write_str(r#"\f"#),
        '\n' => f.write_str(r#"\n"#),
        '\r' => f.write_str(r#"\r"#),
        '\t' => f.write_str(r#"\t"#),
        '\\' => f.write_str(r#"\\"#),
        c if c.is_control() => write!(f, "\\u{:04x}", c as u32),
        // Other unicode chars are written as is
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

/// Write one line of a block string value, escaping characters as necessary.
fn write_block_string_line(line: &'_ str, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut char_iter = line.char_indices();

    while let Some((pos, c)) = char_iter.next() {
        // Output """ as \"""
        if c == '"' && line.get(pos..pos + 3) == Some("\"\"\"") {
            // We know there will be two more " characters.
            // Skip them so we can output """""" as \"""\""" instead of as \"\"\"\"""
            char_iter.next();
            char_iter.next();

            f.write_str("\\\"\"\"")?;
            continue;
        }

        f.write_char(c)?;
    }

    Ok(())
}

impl fmt::Display for BlockStringFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent = " ".repeat(self.indent);

        write!(f, "{indent}\"\"\"")?;
        for line in self.string.lines() {
            write!(f, "\n{indent}")?;
            write_block_string_line(line, f)?;
        }
        write!(f, "\n{indent}\"\"\"")?;

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
            write_character(c, f)?;
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

/// For multi-line strings and strings containing ", try to use a block string.
/// It's not possible to use a block string if characters would need to be escaped.
fn should_use_block_string(s: &str) -> bool {
    s.contains(['"', '\n']) && s.lines().all(|line| !line.contains(char::is_control))
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
            r#""""
котя(猫, ねこ, قطة) любить дрімати в "кутку" з рослинами
""""#
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
            r#""Favourite cat nap spots include:\rplant corner,\rpile of clothes.""#,
        );
    }

    #[test]
    fn it_encodes_indented_desciption() {
        let desc = StringValue::Field {
            source: "Favourite cat nap spots include:\n  plant corner,\n  pile of clothes."
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#"  """
  Favourite cat nap spots include:
    plant corner,
    pile of clothes.
  """"#,
        );

        let desc = StringValue::Field {
            source: "Favourite cat nap spots include:\r\n  plant corner,\r\n  pile of clothes."
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#"  """
  Favourite cat nap spots include:
    plant corner,
    pile of clothes.
  """"#,
        );

        let desc = StringValue::Field {
            source: "Favourite cat nap spots include:\r  plant corner,\r  pile of clothes."
                .to_string(),
        };

        assert_eq!(
            desc.to_string(),
            r#"  "Favourite cat nap spots include:\r  plant corner,\r  pile of clothes.""#,
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

    #[test]
    fn it_encodes_triple_quotes() {
        let source = r#"this """ has """ triple """ quotes"#.to_string();

        let desc = StringValue::Top { source };
        assert_eq!(
            desc.to_string(),
            r#""""
this \""" has \""" triple \""" quotes
""""#
        );

        let source = r#"this """ has """" many """"""" quotes"#.to_string();

        let desc = StringValue::Top { source };
        println!("{desc}");
        assert_eq!(
            desc.to_string(),
            r#""""
this \""" has \"""" many \"""\"""" quotes
""""#
        );
    }

    #[test]
    fn it_encodes_control_characters() {
        let source = "control \u{009c} character".to_string();

        let desc = StringValue::Top { source };
        assert_eq!(desc.to_string(), r#""control \u009c character""#);

        let source = "multi-line\nwith control \u{009c} character\n :)".to_string();

        let desc = StringValue::Top { source };
        println!("{desc}");
        assert_eq!(
            desc.to_string(),
            r#""multi-line\nwith control \u009c character\n :)""#
        );
    }
}
