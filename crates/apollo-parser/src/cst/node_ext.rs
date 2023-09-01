use rowan::{GreenToken, SyntaxKind};

use crate::{cst, cst::CstNode, SyntaxNode, TokenText};
use std::num::{ParseFloatError, ParseIntError};

impl cst::Name {
    pub fn text(&self) -> TokenText {
        text_of_first_token(self.syntax())
    }
}

impl cst::Variable {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get variable's NAME token")
            .text()
    }
}

impl cst::EnumValue {
    pub fn text(&self) -> TokenText {
        self.name()
            .expect("Cannot get enum value's NAME token")
            .text()
    }
}

impl cst::DirectiveLocation {
    pub fn text(self) -> Option<TokenText> {
        let txt = if self.query_token().is_some() {
            Some("QUERY")
        } else if self.mutation_token().is_some() {
            Some("MUTATION")
        } else if self.subscription_token().is_some() {
            Some("SUBSCRIPTION")
        } else if self.field_token().is_some() {
            Some("FIELD")
        } else if self.fragment_definition_token().is_some() {
            Some("FRAGMENT_DEFINITION")
        } else if self.fragment_spread_token().is_some() {
            Some("FRAGMENT_SPREAD")
        } else if self.inline_fragment_token().is_some() {
            Some("INLINE_FRAGMENT")
        } else if self.variable_definition_token().is_some() {
            Some("VARIABLE_DEFINITION")
        } else if self.schema_token().is_some() {
            Some("SCHEMA")
        } else if self.scalar_token().is_some() {
            Some("SCALAR")
        } else if self.object_token().is_some() {
            Some("OBJECT")
        } else if self.field_definition_token().is_some() {
            Some("FIELD_DEFINITION")
        } else if self.argument_definition_token().is_some() {
            Some("ARGUMENT_DEFINITION")
        } else if self.interface_token().is_some() {
            Some("INTERFACE")
        } else if self.union_token().is_some() {
            Some("UNION")
        } else if self.enum_token().is_some() {
            Some("ENUM")
        } else if self.enum_value_token().is_some() {
            Some("ENUM_VALUE")
        } else if self.input_object_token().is_some() {
            Some("INPUT_OBJECT")
        } else if self.input_field_definition_token().is_some() {
            Some("INPUT_FIELD_DEFINITION")
        } else {
            None
        };

        txt.map(|txt| {
            TokenText(GreenToken::new(
                SyntaxKind(crate::SyntaxKind::DIRECTIVE_LOCATION as u16),
                txt,
            ))
        })
    }
}

impl cst::Definition {
    /// Return the name of this definition, if any. Schema definitions are unnamed and always
    /// return `None`.
    pub fn name(&self) -> Option<cst::Name> {
        match self {
            Self::OperationDefinition(it) => it.name(),
            Self::FragmentDefinition(it) => it.fragment_name()?.name(),
            Self::DirectiveDefinition(it) => it.name(),
            Self::SchemaDefinition(_) => None,
            Self::ScalarTypeDefinition(it) => it.name(),
            Self::ObjectTypeDefinition(it) => it.name(),
            Self::InterfaceTypeDefinition(it) => it.name(),
            Self::UnionTypeDefinition(it) => it.name(),
            Self::EnumTypeDefinition(it) => it.name(),
            Self::InputObjectTypeDefinition(it) => it.name(),
            Self::SchemaExtension(_) => None,
            Self::ScalarTypeExtension(it) => it.name(),
            Self::ObjectTypeExtension(it) => it.name(),
            Self::InterfaceTypeExtension(it) => it.name(),
            Self::UnionTypeExtension(it) => it.name(),
            Self::EnumTypeExtension(it) => it.name(),
            Self::InputObjectTypeExtension(it) => it.name(),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            cst::Definition::OperationDefinition(_) => "OperationDefinition",
            cst::Definition::FragmentDefinition(_) => "FragmentDefinition",
            cst::Definition::DirectiveDefinition(_) => "DirectiveDefinition",
            cst::Definition::ScalarTypeDefinition(_) => "ScalarTypeDefinition",
            cst::Definition::ObjectTypeDefinition(_) => "ObjectTypeDefinition",
            cst::Definition::InterfaceTypeDefinition(_) => "InterfaceTypeDefinition",
            cst::Definition::UnionTypeDefinition(_) => "UnionTypeDefinition",
            cst::Definition::EnumTypeDefinition(_) => "EnumTypeDefinition",
            cst::Definition::InputObjectTypeDefinition(_) => "InputObjectTypeDefinition",
            cst::Definition::SchemaDefinition(_) => "SchemaDefinition",
            cst::Definition::SchemaExtension(_) => "SchemaExtension",
            cst::Definition::ScalarTypeExtension(_) => "ScalarTypeExtension",
            cst::Definition::ObjectTypeExtension(_) => "ObjectTypeExtension",
            cst::Definition::InterfaceTypeExtension(_) => "InterfaceTypeExtension",
            cst::Definition::UnionTypeExtension(_) => "UnionTypeExtension",
            cst::Definition::EnumTypeExtension(_) => "EnumTypeExtension",
            cst::Definition::InputObjectTypeExtension(_) => "InputObjectTypeExtension",
        }
    }

    pub fn is_executable_definition(&self) -> bool {
        matches!(
            self,
            Self::OperationDefinition(_) | Self::FragmentDefinition(_)
        )
    }

    pub fn is_extension_definition(&self) -> bool {
        matches!(
            self,
            Self::SchemaExtension(_)
                | Self::ScalarTypeExtension(_)
                | Self::ObjectTypeExtension(_)
                | Self::InterfaceTypeExtension(_)
                | Self::UnionTypeExtension(_)
                | Self::EnumTypeExtension(_)
                | Self::InputObjectTypeExtension(_)
        )
    }
}

impl From<cst::StringValue> for String {
    fn from(val: cst::StringValue) -> Self {
        Self::from(&val)
    }
}

/// Handle escaped characters in a StringValue.
///
/// Panics on invalid escape sequences. Those should be rejected in the lexer already.
fn unescape_string(input: &str) -> String {
    let mut output = String::with_capacity(input.len());

    let mut iter = input.chars();
    while let Some(c) = iter.next() {
        match c {
            '\\' => {
                let Some(c2) = iter.next() else {
                    output.push(c);
                    break;
                };

                let mut unicode = || {
                    // 1. Let value be the 16-bit hexadecimal value represented
                    // by the sequence of hexadecimal digits within EscapedUnicode.
                    let value = iter.by_ref().take(4).fold(0, |acc, c| {
                        let digit = c.to_digit(16).unwrap();
                        (acc << 4) + digit
                    });
                    // 2. Return the code point value.
                    char::from_u32(value).unwrap()
                };

                match c2 {
                    '"' | '\\' | '/' => output.push(c2),
                    'b' => output.push('\u{0008}'),
                    'f' => output.push('\u{000c}'),
                    'n' => output.push('\n'),
                    'r' => output.push('\r'),
                    't' => output.push('\t'),
                    'u' => output.push(unicode()),
                    _ => (),
                }
            }
            _ => output.push(c),
        }
    }

    output
}

const ESCAPED_TRIPLE_QUOTE: &str = r#"\""""#;
const TRIPLE_QUOTE: &str = r#"""""#;

fn is_block_string(input: &str) -> bool {
    input.starts_with(TRIPLE_QUOTE)
}

/// Iterator over the lines in a GraphQL string, using GraphQL's definition of newlines
/// (\r\n, \n, or just \r).
struct GraphQLLines<'a> {
    input: &'a str,
    finished: bool,
}

impl<'a> GraphQLLines<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            finished: false,
        }
    }
}

impl<'a> Iterator for GraphQLLines<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        // Can't just check for the input string being empty, as an empty string should still
        // produce one line.
        if self.finished {
            return None;
        }

        let Some(index) = memchr::memchr2(b'\r', b'\n', self.input.as_bytes()) else {
            self.finished = true;
            return Some(self.input);
        };
        let line = &self.input[..index];
        let rest = match self.input.get(index..=index + 1) {
            Some("\r\n") => &self.input[index + 2..],
            _ => &self.input[index + 1..],
        };
        self.input = rest;
        Some(line)
    }
}

/// Split lines on \n, \r\n, and just \r
fn split_lines(input: &str) -> impl Iterator<Item = &str> {
    GraphQLLines::new(input)
}

/// Replace a literal pattern in a string but push the output to an existing string.
///
/// Like `str::replace`, but doesn't allocate if there's enough space in the provided output.
fn replace_into(input: &str, pattern: &str, replace: &str, output: &mut String) {
    let mut last_index = 0;
    for index in memchr::memmem::find_iter(input.as_bytes(), pattern.as_bytes()) {
        output.push_str(&input[last_index..index]);
        output.push_str(replace);
        last_index = index + pattern.len();
    }
    if last_index < input.len() {
        output.push_str(&input[last_index..]);
    }
}

/// Implementation of the spec function `BlockStringValue(rawValue)`. In addition to handling
/// indents and newline normalization, this also handles escape sequences (strictly not part of
/// BlockStringValue in the spec, but more efficient to do it at the same time).
///
/// Spec: https://spec.graphql.org/October2021/#BlockStringValue()
fn unescape_block_string(raw_value: &str) -> String {
    /// WhiteSpace :: Horizontal Tab (U+0009) Space (U+0020)
    fn is_whitespace(c: char) -> bool {
        matches!(c, ' ' | '\t')
    }
    /// Check if a string is all WhiteSpace. This expects a single line of input.
    fn is_whitespace_line(line: &str) -> bool {
        line.chars().all(is_whitespace)
    }
    /// Count the indentation of a single line (how many WhiteSpace characters are at the start).
    fn count_indent(line: &str) -> usize {
        line.chars().take_while(|&c| is_whitespace(c)).count()
    }

    // 1. Let lines be the result of splitting rawValue by LineTerminator.
    // 2. Let commonIndent be null.
    // 3. For each line in lines:
    let common_indent = split_lines(raw_value)
        // 3.a. If line is the first item in lines, continue to the next line.
        .skip(1)
        .filter_map(|line| {
            // 3.b. Let length be the number of characters in line.
            // We will compare this byte length to a character length below, but
            // `count_indent` only ever counts one-byte characters, so it's equivalent.
            let length = line.len();
            // 3.c. Let indent be the number of leading consecutive WhiteSpace characters in line.
            let indent = count_indent(line);
            // 3.d. If indent is less than length:
            (indent < length).then_some(indent)
        })
        .min()
        .unwrap_or(0);

    let mut lines = split_lines(raw_value)
        .enumerate()
        // 4.a. For each line in lines:
        .map(|(index, line)| {
            // 4.a.i. If line is the first item in lines, continue to the next line.
            if index == 0 {
                line
            } else {
                // 4.a.ii. Remove commonIndent characters from the beginning of line.
                &line[common_indent.min(line.len())..]
            }
        })
        // 5. While the first item line in lines contains only WhiteSpace:
        // 5.a. Remove the first item from lines.
        .skip_while(|line| is_whitespace_line(line));

    // (Step 6 is done at the end so we don't need an intermediate allocation.)

    // 7. Let formatted be the empty character sequence.
    let mut formatted = String::with_capacity(raw_value.len());

    // 8. For each line in lines:
    // 8.a. If line is the first item in lines:
    if let Some(line) = lines.next() {
        // 8.a.i. Append formatted with line.
        replace_into(line, ESCAPED_TRIPLE_QUOTE, TRIPLE_QUOTE, &mut formatted);
    };

    let mut final_char_index = formatted.len();

    // 8.b. Otherwise:
    for line in lines {
        // 8.b.i. Append formatted with a line feed character (U+000A).
        formatted.push('\n');
        // 8.b.ii. Append formatted with line.
        replace_into(line, ESCAPED_TRIPLE_QUOTE, TRIPLE_QUOTE, &mut formatted);

        // Track the last non-whitespace line for implementing step 6 in the spec.
        if !is_whitespace_line(line) {
            final_char_index = formatted.len();
        }
    }

    // 6. Implemented differently: remove WhiteSpace-only lines from the end.
    formatted.truncate(final_char_index);

    // 9. Return formatted.
    formatted
}

// TODO(@goto-bus-stop) As this handles escaping, which can fail in theory, it should be TryFrom
impl From<&'_ cst::StringValue> for String {
    fn from(val: &'_ cst::StringValue) -> Self {
        let text = text_of_first_token(val.syntax());
        // These slices would panic if the contents are invalid, but the lexer already guarantees that the
        // string is valid.
        if is_block_string(&text) {
            unescape_block_string(&text[3..text.len() - 3])
        } else {
            unescape_string(&text[1..text.len() - 1])
        }
    }
}

impl TryFrom<cst::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: cst::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::IntValue> for i32 {
    type Error = ParseIntError;

    fn try_from(val: &'_ cst::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: cst::IntValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::IntValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ cst::IntValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: cst::FloatValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::FloatValue> for f64 {
    type Error = ParseFloatError;

    fn try_from(val: &'_ cst::FloatValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

impl TryFrom<cst::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: cst::BooleanValue) -> Result<Self, Self::Error> {
        Self::try_from(&val)
    }
}

impl TryFrom<&'_ cst::BooleanValue> for bool {
    type Error = std::str::ParseBoolError;

    fn try_from(val: &'_ cst::BooleanValue) -> Result<Self, Self::Error> {
        let text = text_of_first_token(val.syntax());
        text.parse()
    }
}

fn text_of_first_token(node: &SyntaxNode) -> TokenText {
    let first_token = node
        .green()
        .children()
        .next()
        .and_then(|it| it.into_token())
        .unwrap()
        .to_owned();

    TokenText(first_token)
}

#[cfg(test)]
mod string_tests {
    use super::unescape_string;

    #[test]
    fn it_parses_strings() {
        assert_eq!(unescape_string(r"simple"), "simple");
        assert_eq!(unescape_string(r" white space "), " white space ");
    }

    #[test]
    fn it_unescapes_strings() {
        assert_eq!(unescape_string(r#"quote \""#), "quote \"");
        assert_eq!(
            unescape_string(r"escaped \n\r\b\t\f"),
            "escaped \n\r\u{0008}\t\u{000c}"
        );
        assert_eq!(unescape_string(r"slashes \\ \/"), r"slashes \ /");
        assert_eq!(
            unescape_string("unescaped unicode outside BMP 😀"),
            "unescaped unicode outside BMP 😀"
        );
        assert_eq!(
            unescape_string(r"unicode \u1234\u5678\u90AB\uCDEF"),
            "unicode \u{1234}\u{5678}\u{90AB}\u{CDEF}"
        );
    }
}

#[cfg(test)]
mod block_string_tests {
    use super::{split_lines, unescape_block_string};

    #[test]
    fn it_splits_lines_by_graphql_newline_definition() {
        let plain_newlines: Vec<_> = split_lines(
            r#"source text
    with some
    new


    lines
        "#,
        )
        .collect();

        assert_eq!(
            plain_newlines,
            [
                "source text",
                "    with some",
                "    new",
                "",
                "",
                "    lines",
                "        ",
            ]
        );

        let different_endings: Vec<_> =
            split_lines("with\nand\r\nand\rall in the same\r\nstring").collect();
        assert_eq!(
            different_endings,
            ["with", "and", "and", "all in the same", "string",]
        );

        let empty_string: Vec<_> = split_lines("").collect();
        assert_eq!(empty_string, [""]);

        let empty_line: Vec<_> = split_lines("\n\r\r\n").collect();
        assert_eq!(empty_line, ["", "", "", ""]);
    }

    #[test]
    fn it_normalizes_block_string_newlines() {
        assert_eq!(unescape_block_string("multi\nline"), "multi\nline");
        assert_eq!(unescape_block_string("multi\r\nline"), "multi\nline");
        assert_eq!(unescape_block_string("multi\rline"), "multi\nline");
    }

    #[test]
    fn it_does_not_unescape_block_strings() {
        assert_eq!(
            unescape_block_string(r"escaped \n\r\b\t\f"),
            r"escaped \n\r\b\t\f"
        );
        assert_eq!(unescape_block_string(r"slashes \\ \/"), r"slashes \\ \/");
        assert_eq!(
            unescape_block_string("unescaped unicode outside BMP \u{1f600}"),
            "unescaped unicode outside BMP \u{1f600}"
        );
    }

    #[test]
    fn it_dedents_block_strings() {
        assert_eq!(
            unescape_block_string("  intact whitespace with one line  "),
            "  intact whitespace with one line  "
        );

        assert_eq!(
            unescape_block_string(
                r"
            This is
            indented
            quite a lot
    "
            ),
            r"This is
indented
quite a lot"
        );

        assert_eq!(
            unescape_block_string(
                r"

        spans
          multiple
            lines

    "
            ),
            r"spans
  multiple
    lines"
        );
    }
}
