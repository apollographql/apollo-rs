//! Serialization that matches the output of graphql-js `print()`.
//!
//! This is a port of `packages/graphql/src/language/printer.ts`,
//! `printString.ts` and `blockString.ts` from graphql-js 17.0.2.
//! Every function here mirrors its graphql-js counterpart,
//! including quirks such as `{  }` for an empty object value,
//! so that the output is byte-for-byte identical for the same AST.
//!
//! Line lengths are measured in UTF-16 code units like JavaScript's `String.length`.

use super::*;

const MAX_LINE_LENGTH: usize = 80;

/// The length of a string as JavaScript's `String.prototype.length` would report it
fn js_len(string: &str) -> usize {
    string.encode_utf16().count()
}

/// Like graphql-js `join`: concatenates non-empty items with `separator`
fn join<'a>(items: impl IntoIterator<Item = &'a str>, separator: &str) -> String {
    let mut result = String::new();
    for item in items {
        if item.is_empty() {
            continue;
        }
        if !result.is_empty() {
            result.push_str(separator);
        }
        result.push_str(item);
    }
    result
}

fn join_owned(items: &[String], separator: &str) -> String {
    join(items.iter().map(String::as_str), separator)
}

/// Like graphql-js `wrap`: `start + string + end` if `string` is non-empty, otherwise empty
fn wrap(start: &str, string: &str, end: &str) -> String {
    if string.is_empty() {
        String::new()
    } else {
        format!("{start}{string}{end}")
    }
}

/// Like graphql-js `indent`: indents every line by two spaces (including empty lines)
fn indent(string: &str) -> String {
    wrap("  ", &string.replace('\n', "\n  "), "")
}

/// Like graphql-js `block`: `{ … }` with one item per indented line
fn block(items: &[String]) -> String {
    wrap("{\n", &indent(&join_owned(items, "\n")), "\n}")
}

fn has_multiline_items(items: &[String]) -> bool {
    items.iter().any(|item| item.contains('\n'))
}

/// Like graphql-js `wrappedLineAndArgs`
fn wrapped_line_and_args(prefix: &str, args: &[String]) -> String {
    let args_line = format!("{prefix}{}", wrap("(", &join_owned(args, ", "), ")"));
    if js_len(&args_line) > MAX_LINE_LENGTH {
        format!(
            "{prefix}{}",
            wrap("(\n", &indent(&join_owned(args, "\n")), "\n)")
        )
    } else {
        args_line
    }
}

/// Like graphql-js `printString`
pub(crate) fn print_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for c in value.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{8}' => result.push_str("\\b"),
            '\t' => result.push_str("\\t"),
            '\n' => result.push_str("\\n"),
            '\u{C}' => result.push_str("\\f"),
            '\r' => result.push_str("\\r"),
            '\0'..='\u{1F}' | '\u{7F}'..='\u{9F}' => {
                result.push_str(&format!("\\u{:04X}", c as u32))
            }
            _ => result.push(c),
        }
    }
    result.push('"');
    result
}

/// The `WhiteSpace` production: space or horizontal tab
fn is_white_space(c: char) -> bool {
    c == ' ' || c == '\t'
}

/// Like JavaScript `string.split(/\r\n|[\n\r]/g)`
fn split_lines(mut string: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    while let Some(i) = string.find(['\n', '\r']) {
        lines.push(&string[..i]);
        let after = &string[i..];
        string = after.strip_prefix("\r\n").unwrap_or(&after[1..]);
    }
    lines.push(string);
    lines
}

/// Like graphql-js `isPrintableAsBlockString`
pub(crate) fn is_printable_as_block_string(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    let mut is_empty_line = true;
    let mut has_indent = false;
    let mut has_common_indent = true;
    let mut seen_non_empty_line = false;
    for c in value.chars() {
        match c {
            '\0'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}' | '\u{F}' | '\r' => return false,
            '\n' => {
                if is_empty_line && !seen_non_empty_line {
                    // Has leading new line
                    return false;
                }
                seen_non_empty_line = true;
                is_empty_line = true;
                has_indent = false;
            }
            ' ' | '\t' => has_indent |= is_empty_line,
            _ => {
                has_common_indent &= has_indent;
                is_empty_line = false;
            }
        }
    }
    if is_empty_line {
        // Has trailing empty lines
        return false;
    }
    if has_common_indent && seen_non_empty_line {
        // Has internal indent
        return false;
    }
    true
}

/// Like graphql-js `printBlockString` (without the `minimize` option)
pub(crate) fn print_block_string(value: &str) -> String {
    let escaped_value = value.replace("\"\"\"", "\\\"\"\"");
    // Expand a block string's raw value into independent lines
    let lines = split_lines(&escaped_value);
    let is_single_line = lines.len() == 1;

    // If common indentation is found we can fix some of those cases by adding a leading new line
    let force_leading_new_line = lines.len() > 1
        && lines[1..]
            .iter()
            .all(|line| line.is_empty() || line.starts_with(is_white_space));

    // Trailing triple quotes just looks confusing but doesn't force trailing new line
    let has_trailing_triple_quotes = escaped_value.ends_with("\\\"\"\"");

    // Trailing quote (single or double) or slash forces trailing new line
    let has_trailing_quote = value.ends_with('"') && !has_trailing_triple_quotes;
    let has_trailing_slash = value.ends_with('\\');
    let force_trailing_newline = has_trailing_quote || has_trailing_slash;

    let print_as_multiple_lines = !is_single_line
        || js_len(value) > 70
        || force_trailing_newline
        || force_leading_new_line
        || has_trailing_triple_quotes;

    let mut result = String::new();
    // Format a multi-line block quote to account for leading space
    let skip_leading_new_line = is_single_line && value.starts_with(is_white_space);
    if (print_as_multiple_lines && !skip_leading_new_line) || force_leading_new_line {
        result.push('\n');
    }
    result.push_str(&escaped_value);
    if print_as_multiple_lines || force_trailing_newline {
        result.push('\n');
    }
    format!("\"\"\"{result}\"\"\"")
}

/// Descriptions print as block strings when graphql-js would consider that printable.
///
/// The AST does not record whether the source used block-string syntax
/// (<https://github.com/apollographql/apollo-rs/issues/1120>),
/// so this uses the same heuristic as graphql-js `printSchema`.
fn print_description(description: &Option<Node<str>>) -> String {
    match description {
        Some(description) => {
            let printed = if is_printable_as_block_string(description) {
                print_block_string(description)
            } else {
                print_string(description)
            };
            wrap("", &printed, "\n")
        }
        None => String::new(),
    }
}

fn print_directives(directives: &DirectiveList) -> String {
    let printed: Vec<String> = directives
        .iter()
        .map(|dir| dir.print_graphql_js())
        .collect();
    join_owned(&printed, " ")
}

pub(crate) fn print_selection_set(selection_set: &[Selection]) -> String {
    let printed: Vec<String> = selection_set
        .iter()
        .map(|selection| selection.print_graphql_js())
        .collect();
    block(&printed)
}

fn print_arguments(arguments: &[Node<Argument>]) -> Vec<String> {
    arguments.iter().map(|arg| arg.print_graphql_js()).collect()
}

fn print_implements(interfaces: &[Name]) -> String {
    wrap(
        "implements ",
        &join(interfaces.iter().map(Name::as_str), " & "),
        "",
    )
}

fn print_fields(fields: &[Node<FieldDefinition>]) -> String {
    let printed: Vec<String> = fields
        .iter()
        .map(|field| field.print_graphql_js())
        .collect();
    block(&printed)
}

fn print_input_fields(fields: &[Node<InputValueDefinition>]) -> String {
    let printed: Vec<String> = fields
        .iter()
        .map(|field| field.print_graphql_js())
        .collect();
    block(&printed)
}

fn print_enum_values(values: &[Node<EnumValueDefinition>]) -> String {
    let printed: Vec<String> = values
        .iter()
        .map(|value| value.print_graphql_js())
        .collect();
    block(&printed)
}

fn print_root_operations(root_operations: &[Node<(OperationType, NamedType)>]) -> String {
    let printed: Vec<String> = root_operations
        .iter()
        .map(|node| {
            let (operation_type, type_name) = &**node;
            format!("{}: {type_name}", operation_type.name())
        })
        .collect();
    block(&printed)
}

fn print_union_members(members: &[NamedType]) -> String {
    wrap("= ", &join(members.iter().map(Name::as_str), " | "), "")
}

/// Like graphql-js `FieldDefinition` and `DirectiveDefinition` argument printing
fn print_arguments_definition(arguments: &[Node<InputValueDefinition>]) -> String {
    let printed: Vec<String> = arguments.iter().map(|arg| arg.print_graphql_js()).collect();
    if has_multiline_items(&printed) {
        wrap("(\n", &indent(&join_owned(&printed, "\n")), "\n)")
    } else {
        wrap("(", &join_owned(&printed, ", "), ")")
    }
}

impl Document {
    pub(crate) fn print_graphql_js(&self) -> String {
        let printed: Vec<String> = self
            .definitions
            .iter()
            .map(|def| def.print_graphql_js())
            .collect();
        join_owned(&printed, "\n\n")
    }
}

impl Definition {
    pub(crate) fn print_graphql_js(&self) -> String {
        match self {
            Definition::OperationDefinition(def) => def.print_graphql_js(),
            Definition::FragmentDefinition(def) => def.print_graphql_js(),
            Definition::DirectiveDefinition(def) => def.print_graphql_js(),
            Definition::SchemaDefinition(def) => def.print_graphql_js(),
            Definition::ScalarTypeDefinition(def) => def.print_graphql_js(),
            Definition::ObjectTypeDefinition(def) => def.print_graphql_js(),
            Definition::InterfaceTypeDefinition(def) => def.print_graphql_js(),
            Definition::UnionTypeDefinition(def) => def.print_graphql_js(),
            Definition::EnumTypeDefinition(def) => def.print_graphql_js(),
            Definition::InputObjectTypeDefinition(def) => def.print_graphql_js(),
            Definition::SchemaExtension(def) => def.print_graphql_js(),
            Definition::ScalarTypeExtension(def) => def.print_graphql_js(),
            Definition::ObjectTypeExtension(def) => def.print_graphql_js(),
            Definition::InterfaceTypeExtension(def) => def.print_graphql_js(),
            Definition::UnionTypeExtension(def) => def.print_graphql_js(),
            Definition::EnumTypeExtension(def) => def.print_graphql_js(),
            Definition::InputObjectTypeExtension(def) => def.print_graphql_js(),
        }
    }
}

impl OperationDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            operation_type,
            name,
            variables,
            directives,
            selection_set,
        } = self;
        let var_defs: Vec<String> = variables.iter().map(|var| var.print_graphql_js()).collect();
        let var_defs = if has_multiline_items(&var_defs) {
            wrap("(\n", &join_owned(&var_defs, "\n"), "\n)")
        } else {
            wrap("(", &join_owned(&var_defs, ", "), ")")
        };
        let name = name.as_ref().map(Name::as_str).unwrap_or_default();
        let prefix = join(
            [
                operation_type.name(),
                join([name, &var_defs], "").as_str(),
                print_directives(directives).as_str(),
            ],
            " ",
        );
        let selection_set = print_selection_set(selection_set);
        // Anonymous queries with no directives or variable definitions can use the query short form
        if prefix == "query" {
            selection_set
        } else {
            format!("{prefix} {selection_set}")
        }
    }
}

impl VariableDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            ty,
            default_value,
            directives,
        } = self;
        let default_value = default_value
            .as_ref()
            .map(|value| value.print_graphql_js())
            .unwrap_or_default();
        format!(
            "${name}: {ty}{}{}",
            wrap(" = ", &default_value, ""),
            wrap(" ", &print_directives(directives), "")
        )
    }
}

impl Selection {
    pub(crate) fn print_graphql_js(&self) -> String {
        match self {
            Selection::Field(x) => x.print_graphql_js(),
            Selection::FragmentSpread(x) => x.print_graphql_js(),
            Selection::InlineFragment(x) => x.print_graphql_js(),
        }
    }
}

impl Field {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            alias,
            name,
            arguments,
            directives,
            selection_set,
        } = self;
        let prefix = match alias {
            Some(alias) => format!("{alias}: {name}"),
            None => name.to_string(),
        };
        join(
            [
                wrapped_line_and_args(&prefix, &print_arguments(arguments)).as_str(),
                wrap(" ", &print_directives(directives), "").as_str(),
                wrap(" ", &print_selection_set(selection_set), "").as_str(),
            ],
            "",
        )
    }
}

impl Argument {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self { name, value } = self;
        format!("{name}: {}", value.print_graphql_js())
    }
}

impl FragmentSpread {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            fragment_name,
            directives,
        } = self;
        format!(
            "...{fragment_name}{}",
            wrap(" ", &print_directives(directives), "")
        )
    }
}

impl InlineFragment {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            type_condition,
            directives,
            selection_set,
        } = self;
        let type_condition = type_condition
            .as_ref()
            .map(Name::as_str)
            .unwrap_or_default();
        join(
            [
                "...",
                wrap("on ", type_condition, "").as_str(),
                print_directives(directives).as_str(),
                print_selection_set(selection_set).as_str(),
            ],
            " ",
        )
    }
}

impl FragmentDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            type_condition,
            directives,
            selection_set,
        } = self;
        format!(
            "fragment {name} on {type_condition} {}{}",
            wrap("", &print_directives(directives), " "),
            print_selection_set(selection_set)
        )
    }
}

impl Value {
    pub(crate) fn print_graphql_js(&self) -> String {
        match self {
            Value::Null => "null".to_owned(),
            Value::Enum(name) => name.to_string(),
            Value::Variable(name) => format!("${name}"),
            // The AST does not record whether the source used block-string syntax
            // (https://github.com/apollographql/apollo-rs/issues/1120),
            // so string values always print in the quoted form.
            Value::String(value) => print_string(value),
            Value::Float(value) => value.to_string(),
            Value::Int(value) => value.to_string(),
            Value::Boolean(true) => "true".to_owned(),
            Value::Boolean(false) => "false".to_owned(),
            Value::List(values) => {
                let printed: Vec<String> = values
                    .iter()
                    .map(|value| value.print_graphql_js())
                    .collect();
                let values_line = format!("[{}]", join_owned(&printed, ", "));
                if js_len(&values_line) > MAX_LINE_LENGTH {
                    format!("[\n{}\n]", indent(&join_owned(&printed, "\n")))
                } else {
                    values_line
                }
            }
            Value::Object(fields) => {
                let printed: Vec<String> = fields
                    .iter()
                    .map(|(name, value)| format!("{name}: {}", value.print_graphql_js()))
                    .collect();
                let fields_line = format!("{{ {} }}", join_owned(&printed, ", "));
                if js_len(&fields_line) > MAX_LINE_LENGTH {
                    block(&printed)
                } else {
                    fields_line
                }
            }
        }
    }
}

impl Directive {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self { name, arguments } = self;
        format!(
            "@{name}{}",
            wrap("(", &join_owned(&print_arguments(arguments), ", "), ")")
        )
    }
}

impl SchemaDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            directives,
            root_operations,
        } = self;
        print_description(description)
            + &join(
                [
                    "schema",
                    print_directives(directives).as_str(),
                    print_root_operations(root_operations).as_str(),
                ],
                " ",
            )
    }
}

impl ScalarTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            directives,
        } = self;
        print_description(description)
            + &join(["scalar", name, print_directives(directives).as_str()], " ")
    }
}

impl ObjectTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        print_description(description)
            + &join(
                [
                    "type",
                    name,
                    print_implements(implements_interfaces).as_str(),
                    print_directives(directives).as_str(),
                    print_fields(fields).as_str(),
                ],
                " ",
            )
    }
}

impl FieldDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            arguments,
            ty,
            directives,
        } = self;
        format!(
            "{}{name}{}: {ty}{}",
            print_description(description),
            print_arguments_definition(arguments),
            wrap(" ", &print_directives(directives), "")
        )
    }
}

impl InputValueDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            ty,
            default_value,
            directives,
        } = self;
        let default_value = default_value
            .as_ref()
            .map(|value| value.print_graphql_js())
            .unwrap_or_default();
        print_description(description)
            + &join(
                [
                    format!("{name}: {ty}").as_str(),
                    wrap("= ", &default_value, "").as_str(),
                    print_directives(directives).as_str(),
                ],
                " ",
            )
    }
}

impl InterfaceTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        print_description(description)
            + &join(
                [
                    "interface",
                    name,
                    print_implements(implements_interfaces).as_str(),
                    print_directives(directives).as_str(),
                    print_fields(fields).as_str(),
                ],
                " ",
            )
    }
}

impl UnionTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            directives,
            members,
        } = self;
        print_description(description)
            + &join(
                [
                    "union",
                    name,
                    print_directives(directives).as_str(),
                    print_union_members(members).as_str(),
                ],
                " ",
            )
    }
}

impl EnumTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            directives,
            values,
        } = self;
        print_description(description)
            + &join(
                [
                    "enum",
                    name,
                    print_directives(directives).as_str(),
                    print_enum_values(values).as_str(),
                ],
                " ",
            )
    }
}

impl EnumValueDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            value,
            directives,
        } = self;
        print_description(description) + &join([value, print_directives(directives).as_str()], " ")
    }
}

impl InputObjectTypeDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            directives,
            fields,
        } = self;
        print_description(description)
            + &join(
                [
                    "input",
                    name,
                    print_directives(directives).as_str(),
                    print_input_fields(fields).as_str(),
                ],
                " ",
            )
    }
}

impl DirectiveDefinition {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            description,
            name,
            arguments,
            repeatable,
            locations,
        } = self;
        format!(
            "{}directive @{name}{}{} on {}",
            print_description(description),
            print_arguments_definition(arguments),
            if *repeatable { " repeatable" } else { "" },
            join(locations.iter().map(|loc| loc.name()), " | ")
        )
    }
}

impl SchemaExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            directives,
            root_operations,
        } = self;
        join(
            [
                "extend schema",
                print_directives(directives).as_str(),
                print_root_operations(root_operations).as_str(),
            ],
            " ",
        )
    }
}

impl ScalarTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self { name, directives } = self;
        join(
            ["extend scalar", name, print_directives(directives).as_str()],
            " ",
        )
    }
}

impl ObjectTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        join(
            [
                "extend type",
                name,
                print_implements(implements_interfaces).as_str(),
                print_directives(directives).as_str(),
                print_fields(fields).as_str(),
            ],
            " ",
        )
    }
}

impl InterfaceTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        join(
            [
                "extend interface",
                name,
                print_implements(implements_interfaces).as_str(),
                print_directives(directives).as_str(),
                print_fields(fields).as_str(),
            ],
            " ",
        )
    }
}

impl UnionTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            directives,
            members,
        } = self;
        join(
            [
                "extend union",
                name,
                print_directives(directives).as_str(),
                print_union_members(members).as_str(),
            ],
            " ",
        )
    }
}

impl EnumTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            directives,
            values,
        } = self;
        join(
            [
                "extend enum",
                name,
                print_directives(directives).as_str(),
                print_enum_values(values).as_str(),
            ],
            " ",
        )
    }
}

impl InputObjectTypeExtension {
    pub(crate) fn print_graphql_js(&self) -> String {
        let Self {
            name,
            directives,
            fields,
        } = self;
        join(
            [
                "extend input",
                name,
                print_directives(directives).as_str(),
                print_input_fields(fields).as_str(),
            ],
            " ",
        )
    }
}

/// Defines the public `to_graphql_js_string` method, forwarding to `print_graphql_js`
macro_rules! impl_to_graphql_js_string {
    ($($ty: path)+) => {
        $(
            impl $ty {
                /// Serialize to GraphQL syntax formatted exactly like graphql-js `print()`.
                ///
                /// See [the `ast` module documentation][super#graphql-js-compatible-serialization]
                /// for details.
                pub fn to_graphql_js_string(&self) -> String {
                    self.print_graphql_js()
                }
            }
        )+
    }
}

impl_to_graphql_js_string! {
    Document
    Definition
    OperationDefinition
    FragmentDefinition
    DirectiveDefinition
    SchemaDefinition
    ScalarTypeDefinition
    ObjectTypeDefinition
    InterfaceTypeDefinition
    UnionTypeDefinition
    EnumTypeDefinition
    InputObjectTypeDefinition
    SchemaExtension
    ScalarTypeExtension
    ObjectTypeExtension
    InterfaceTypeExtension
    UnionTypeExtension
    EnumTypeExtension
    InputObjectTypeExtension
    VariableDefinition
    Selection
    Field
    FragmentSpread
    InlineFragment
    Argument
    Directive
    Value
    FieldDefinition
    InputValueDefinition
    EnumValueDefinition
}
