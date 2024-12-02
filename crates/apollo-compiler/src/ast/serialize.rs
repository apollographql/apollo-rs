use super::*;
use crate::executable;
use crate::schema;
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Serialize<'a, T> {
    pub(crate) node: &'a T,
    pub(crate) config: Config<'a>,
}

#[derive(Debug, Clone)]
pub(crate) struct Config<'a> {
    indent_prefix: Option<&'a str>,
    initial_indent_level: usize,
}

pub(crate) struct State<'config, 'fmt, 'fmt2> {
    config: Config<'config>,
    indent_level: usize,
    output: &'fmt mut fmt::Formatter<'fmt2>,
    /// Have we not written anything yet?
    output_empty: bool,
}

impl<'a, T> Serialize<'a, T> {
    /// Enable indentation and line breaks.
    ///
    /// `prefix` is repeated at the start of each line by the number of indentation levels.
    /// The default is `"  "`, two spaces.
    pub fn indent_prefix(mut self, prefix: &'a str) -> Self {
        self.config.indent_prefix = Some(prefix);
        self
    }

    /// Disable indentation and line breaks
    pub fn no_indent(mut self) -> Self {
        self.config.indent_prefix = None;
        self
    }

    pub fn initial_indent_level(mut self, initial_indent_level: usize) -> Self {
        self.config.initial_indent_level = initial_indent_level;
        self
    }
}

impl Default for Config<'_> {
    fn default() -> Self {
        Self {
            indent_prefix: Some("  "),
            initial_indent_level: 0,
        }
    }
}

macro_rules! display {
    ($state: expr, $e: expr) => {
        fmt::Display::fmt(&$e, $state.output)
    };
    ($state: expr, $($tt: tt)+) => {
        display!($state, format_args!($($tt)+))
    };

}

impl State<'_, '_, '_> {
    pub(crate) fn write(&mut self, str: &str) -> fmt::Result {
        self.output_empty = false;
        self.output.write_str(str)
    }

    pub(crate) fn indent(&mut self) -> fmt::Result {
        self.indent_level += 1;
        self.new_line_common(false)
    }

    pub(crate) fn indent_or_space(&mut self) -> fmt::Result {
        self.indent_level += 1;
        self.new_line_common(true)
    }

    pub(crate) fn dedent(&mut self) -> fmt::Result {
        self.indent_level -= 1; // checked underflow in debug mode
        self.new_line_common(false)
    }

    pub(crate) fn dedent_or_space(&mut self) -> fmt::Result {
        self.indent_level -= 1; // checked underflow in debug mode
        self.new_line_common(true)
    }

    pub(crate) fn new_line_or_space(&mut self) -> fmt::Result {
        self.new_line_common(true)
    }

    fn new_line_common(&mut self, space: bool) -> fmt::Result {
        if let Some(prefix) = self.config.indent_prefix {
            self.write("\n")?;
            for _ in 0..self.indent_level {
                self.write(prefix)?;
            }
        } else if space {
            self.write(" ")?
        }
        Ok(())
    }

    /// Panics if newlines are disabled
    fn require_new_line(&mut self) -> fmt::Result {
        let prefix = self
            .config
            .indent_prefix
            .expect("require_new_line called with newlines disabled");
        self.write("\n")?;
        for _ in 0..self.indent_level {
            self.write(prefix)?;
        }
        Ok(())
    }

    pub(crate) fn newlines_enabled(&self) -> bool {
        self.config.indent_prefix.is_some()
    }

    pub(crate) fn on_single_line<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let indent_prefix = self.config.indent_prefix.take();
        let result = f(self);
        self.config.indent_prefix = indent_prefix;
        result
    }
}

impl Document {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        top_level(state, &self.definitions, |state, def| {
            def.serialize_impl(state)
        })
    }
}

pub(crate) fn top_level<T>(
    state: &mut State,
    iter: impl IntoIterator<Item = T>,
    serialize_one: impl Fn(&mut State, T) -> fmt::Result,
) -> fmt::Result {
    let mut iter = iter.into_iter();
    if let Some(first) = iter.next() {
        serialize_one(state, first)?;
        iter.try_for_each(|item| {
            if state.newlines_enabled() {
                // Empty line between top-level definitions
                state.write("\n")?;
            }
            state.new_line_or_space()?;
            serialize_one(state, item)
        })?;
        // Trailing newline
        if state.newlines_enabled() {
            state.write("\n")?;
        }
    }
    Ok(())
}

impl Definition {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        match self {
            Definition::OperationDefinition(def) => def.serialize_impl(state),
            Definition::FragmentDefinition(def) => def.serialize_impl(state),
            Definition::DirectiveDefinition(def) => def.serialize_impl(state),
            Definition::SchemaDefinition(def) => def.serialize_impl(state),
            Definition::ScalarTypeDefinition(def) => def.serialize_impl(state),
            Definition::ObjectTypeDefinition(def) => def.serialize_impl(state),
            Definition::InterfaceTypeDefinition(def) => def.serialize_impl(state),
            Definition::UnionTypeDefinition(def) => def.serialize_impl(state),
            Definition::EnumTypeDefinition(def) => def.serialize_impl(state),
            Definition::InputObjectTypeDefinition(def) => def.serialize_impl(state),
            Definition::SchemaExtension(def) => def.serialize_impl(state),
            Definition::ScalarTypeExtension(def) => def.serialize_impl(state),
            Definition::ObjectTypeExtension(def) => def.serialize_impl(state),
            Definition::InterfaceTypeExtension(def) => def.serialize_impl(state),
            Definition::UnionTypeExtension(def) => def.serialize_impl(state),
            Definition::EnumTypeExtension(def) => def.serialize_impl(state),
            Definition::InputObjectTypeExtension(def) => def.serialize_impl(state),
        }
    }
}

impl OperationDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        // Deconstruct to get a warning if we forget to serialize something
        let Self {
            operation_type,
            name,
            variables,
            directives,
            selection_set,
        } = self;
        // Only use shorthand when this is the first item.
        // If not, it might be following a `[lookahead != "{"]` grammar production
        let shorthand = state.output_empty
            && *operation_type == OperationType::Query
            && name.is_none()
            && variables.is_empty()
            && directives.is_empty();
        if !shorthand {
            state.write(operation_type.name())?;
            if let Some(name) = &name {
                state.write(" ")?;
                state.write(name)?;
            }
            if !variables.is_empty() {
                state.on_single_line(|state| {
                    comma_separated(state, "(", ")", variables, |state, var| {
                        var.serialize_impl(state)
                    })
                })?
            }
            directives.serialize_impl(state)?;
            state.write(" ")?;
        }
        curly_brackets_space_separated(state, selection_set, |state, sel| sel.serialize_impl(state))
    }
}

impl FragmentDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            type_condition,
            directives,
            selection_set,
        } = self;
        display!(state, "fragment {} on {}", name, type_condition)?;
        directives.serialize_impl(state)?;
        state.write(" ")?;
        curly_brackets_space_separated(state, selection_set, |state, sel| sel.serialize_impl(state))
    }
}

impl DirectiveDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            arguments,
            repeatable,
            locations,
        } = self;
        serialize_description(state, description)?;
        state.write("directive @")?;
        state.write(name)?;
        serialize_arguments_definition(state, arguments)?;

        if *repeatable {
            state.write(" repeatable")?;
        }
        if let Some((first, rest)) = locations.split_first() {
            state.write(" on ")?;
            state.write(first.name())?;
            for location in rest {
                state.write(" | ")?;
                state.write(location.name())?;
            }
        }
        Ok(())
    }
}

fn serialize_arguments_definition(
    state: &mut State,
    arguments: &[Node<InputValueDefinition>],
) -> fmt::Result {
    if !arguments.is_empty() {
        let serialize_arguments = |state: &mut State| {
            comma_separated(state, "(", ")", arguments, |state, arg| {
                arg.serialize_impl(state)
            })
        };
        if arguments
            .iter()
            .any(|arg| arg.description.is_some() || !arg.directives.is_empty())
        {
            serialize_arguments(state)?
        } else {
            state.on_single_line(serialize_arguments)?
        }
    }
    Ok(())
}

impl SchemaDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            directives,
            root_operations,
        } = self;
        serialize_description(state, description)?;
        state.write("schema")?;
        directives.serialize_impl(state)?;
        state.write(" ")?;
        curly_brackets_space_separated(state, root_operations, |state, op| {
            let (operation_type, operation_name) = &**op;
            display!(state, "{}: {}", operation_type, operation_name)
        })
    }
}

impl ScalarTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            directives,
        } = self;
        serialize_description(state, description)?;
        state.write("scalar ")?;
        state.write(name)?;
        directives.serialize_impl(state)
    }
}

impl ObjectTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        serialize_description(state, description)?;
        state.write("type ")?;
        serialize_object_type_like(state, name, implements_interfaces, directives, fields)
    }
}

fn serialize_object_type_like(
    state: &mut State,
    name: &str,
    implements_interfaces: &[Name],
    directives: &DirectiveList,
    fields: &[Node<FieldDefinition>],
) -> Result<(), fmt::Error> {
    state.write(name)?;
    if let Some((first, rest)) = implements_interfaces.split_first() {
        state.write(" implements ")?;
        state.write(first)?;
        for name in rest {
            state.write(" & ")?;
            state.write(name)?;
        }
    }
    directives.serialize_impl(state)?;

    if !fields.is_empty() {
        state.write(" ")?;
        curly_brackets_space_separated(state, fields, |state, field| field.serialize_impl(state))?;
    }
    Ok(())
}

impl InterfaceTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        serialize_description(state, description)?;
        state.write("interface ")?;
        serialize_object_type_like(state, name, implements_interfaces, directives, fields)
    }
}

impl UnionTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            directives,
            members,
        } = self;
        serialize_description(state, description)?;
        state.write("union ")?;
        serialize_union(state, name, directives, members)
    }
}

fn serialize_union(
    state: &mut State,
    name: &str,
    directives: &DirectiveList,
    members: &[Name],
) -> fmt::Result {
    state.write(name)?;
    directives.serialize_impl(state)?;
    if let Some((first, rest)) = members.split_first() {
        state.write(" = ")?;
        state.write(first)?;
        for member in rest {
            state.write(" | ")?;
            state.write(member)?;
        }
    }
    Ok(())
}

impl EnumTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            directives,
            values,
        } = self;
        serialize_description(state, description)?;
        state.write("enum ")?;
        state.write(name)?;
        directives.serialize_impl(state)?;
        if !values.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, values, |state, value| {
                value.serialize_impl(state)
            })?;
        }
        Ok(())
    }
}

impl InputObjectTypeDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            directives,
            fields,
        } = self;
        serialize_description(state, description)?;
        state.write("input ")?;
        state.write(name)?;
        directives.serialize_impl(state)?;
        if !fields.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, fields, |state, f| f.serialize_impl(state))?;
        }
        Ok(())
    }
}

impl SchemaExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            directives,
            root_operations,
        } = self;
        state.write("extend schema")?;
        directives.serialize_impl(state)?;
        if !root_operations.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, root_operations, |state, op| {
                let (operation_type, operation_name) = &**op;
                display!(state, "{}: {}", operation_type, operation_name)
            })?;
        }
        Ok(())
    }
}

impl ScalarTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self { name, directives } = self;
        state.write("extend scalar ")?;
        state.write(name)?;
        directives.serialize_impl(state)
    }
}

impl ObjectTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        state.write("extend type ")?;
        serialize_object_type_like(state, name, implements_interfaces, directives, fields)
    }
}

impl InterfaceTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            implements_interfaces,
            directives,
            fields,
        } = self;
        state.write("extend interface ")?;
        serialize_object_type_like(state, name, implements_interfaces, directives, fields)
    }
}

impl UnionTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            directives,
            members,
        } = self;
        state.write("extend union ")?;
        serialize_union(state, name, directives, members)
    }
}

impl EnumTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            directives,
            values,
        } = self;
        state.write("extend enum ")?;
        state.write(name)?;
        directives.serialize_impl(state)?;
        if !values.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, values, |state, value| {
                value.serialize_impl(state)
            })?;
        }
        Ok(())
    }
}

impl InputObjectTypeExtension {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            directives,
            fields,
        } = self;
        state.write("extend input ")?;
        state.write(name)?;
        directives.serialize_impl(state)?;
        if !fields.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, fields, |state, f| f.serialize_impl(state))?;
        }
        Ok(())
    }
}

impl DirectiveList {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        for dir in self {
            state.write(" ")?;
            dir.serialize_impl(state)?;
        }
        Ok(())
    }
}

impl Directive {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self { name, arguments } = self;
        state.write("@")?;
        state.write(name)?;
        serialize_arguments(state, arguments)
    }
}

impl VariableDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            name,
            ty,
            default_value,
            directives,
        } = self;
        state.write("$")?;
        state.write(name)?;
        state.write(": ")?;
        display!(state, ty)?;
        if let Some(value) = default_value {
            state.write(" = ")?;
            value.serialize_impl(state)?
        }
        directives.serialize_impl(state)
    }
}

impl FieldDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            arguments,
            ty,
            directives,
        } = self;
        serialize_description(state, description)?;
        state.write(name)?;
        serialize_arguments_definition(state, arguments)?;
        state.write(": ")?;
        display!(state, ty)?;
        directives.serialize_impl(state)
    }
}

impl InputValueDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            name,
            ty,
            default_value,
            directives,
        } = self;
        serialize_description(state, description)?;
        state.write(name)?;
        state.write(": ")?;
        display!(state, ty)?;
        if let Some(value) = default_value {
            state.write(" = ")?;
            value.serialize_impl(state)?
        }
        directives.serialize_impl(state)
    }
}

impl EnumValueDefinition {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            description,
            value,
            directives,
        } = self;
        serialize_description(state, description)?;
        state.write(value)?;
        directives.serialize_impl(state)
    }
}

impl Selection {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        match self {
            Selection::Field(x) => x.serialize_impl(state),
            Selection::FragmentSpread(x) => x.serialize_impl(state),
            Selection::InlineFragment(x) => x.serialize_impl(state),
        }
    }
}

impl Field {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            alias,
            name,
            arguments,
            directives,
            selection_set,
        } = self;
        if let Some(alias) = alias {
            state.write(alias)?;
            state.write(": ")?;
        }
        state.write(name)?;
        serialize_arguments(state, arguments)?;
        directives.serialize_impl(state)?;
        if !selection_set.is_empty() {
            state.write(" ")?;
            curly_brackets_space_separated(state, selection_set, |state, sel| {
                sel.serialize_impl(state)
            })?
        }
        Ok(())
    }
}

impl FragmentSpread {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            fragment_name,
            directives,
        } = self;
        state.write("...")?;
        state.write(fragment_name)?;
        directives.serialize_impl(state)
    }
}

impl InlineFragment {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        let Self {
            type_condition,
            directives,
            selection_set,
        } = self;
        if let Some(type_name) = type_condition {
            state.write("... on ")?;
            state.write(type_name)?;
        } else {
            state.write("...")?;
        }
        directives.serialize_impl(state)?;
        state.write(" ")?;
        curly_brackets_space_separated(state, selection_set, |state, sel| sel.serialize_impl(state))
    }
}

impl Value {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        match self {
            Value::Null => state.write("null"),
            Value::Boolean(true) => state.write("true"),
            Value::Boolean(false) => state.write("false"),
            Value::Enum(name) => state.write(name),
            Value::String(value) => {
                let is_description = false;
                serialize_string_value(state, is_description, value)
            }
            Value::Variable(name) => display!(state, "${}", name),
            Value::Float(value) => display!(state, value),
            Value::Int(value) => display!(state, value),
            Value::List(value) => comma_separated(state, "[", "]", value, |state, value| {
                value.serialize_impl(state)
            }),
            Value::Object(value) => {
                comma_separated(state, "{", "}", value, |state, (name, value)| {
                    state.write(name)?;
                    state.write(": ")?;
                    value.serialize_impl(state)
                })
            }
        }
    }
}

impl Argument {
    fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        state.write(&self.name)?;
        state.write(": ")?;
        self.value.serialize_impl(state)
    }
}

fn serialize_arguments(state: &mut State, arguments: &[Node<Argument>]) -> fmt::Result {
    if !arguments.is_empty() {
        state.on_single_line(|state| {
            comma_separated(state, "(", ")", arguments, |state, argument| {
                argument.serialize_impl(state)
            })
        })?
    }
    Ok(())
}

/// Example output: `[a, b, c]` or
///
/// ```text
/// [
///     a,
///     b,
///     c,
/// ]
/// ```
fn comma_separated<T>(
    state: &mut State,
    open: &str,
    close: &str,
    values: &[T],
    serialize_one: impl Fn(&mut State, &T) -> fmt::Result,
) -> fmt::Result {
    state.write(open)?;
    if let Some((first, rest)) = values.split_first() {
        state.indent()?;
        serialize_one(state, first)?;
        for value in rest {
            state.write(",")?;
            state.new_line_or_space()?;
            serialize_one(state, value)?;
        }
        // Trailing comma
        if state.newlines_enabled() {
            state.write(",")?;
        }
        state.dedent()?;
    }
    state.write(close)
}

/// Example output: `{ a b c }` or
///
/// ```text
/// {
///     a
///     b
///     c
/// }
/// ```
pub(crate) fn curly_brackets_space_separated<T>(
    state: &mut State,
    values: &[T],
    serialize_one: impl Fn(&mut State, &T) -> fmt::Result,
) -> fmt::Result {
    state.write("{")?;
    if let Some((first, rest)) = values.split_first() {
        state.indent_or_space()?;
        serialize_one(state, first)?;
        for value in rest {
            state.new_line_or_space()?;
            serialize_one(state, value)?;
        }
        state.dedent_or_space()?;
    }
    state.write("}")
}

fn serialize_string_value(state: &mut State, is_description: bool, mut str: &str) -> fmt::Result {
    let contains_newline = str.contains('\n');
    let prefer_block_string = is_description || contains_newline;
    if state.newlines_enabled() && prefer_block_string && can_be_block_string(str) {
        return serialize_block_string(state, contains_newline, str);
    }
    state.write("\"")?;
    loop {
        if let Some(i) = str.find(|c| (c < ' ' && c != '\t') || c == '"' || c == '\\') {
            let (without_escaping, rest) = str.split_at(i);
            state.write(without_escaping)?;
            // All characters that need escaping are in the ASCII range,
            // and so take a single byte in UTF-8.
            match rest.as_bytes()[0] {
                b'\x08' => state.write("\\b")?,
                b'\n' => state.write("\\n")?,
                b'\x0C' => state.write("\\f")?,
                b'\r' => state.write("\\r")?,
                b'"' => state.write("\\\"")?,
                b'\\' => state.write("\\\\")?,
                byte => display!(state, "\\u{:04X}", byte)?,
            }
            str = &rest[1..]
        } else {
            state.write(str)?;
            break;
        }
    }
    state.write("\"")
}

fn serialize_block_string(state: &mut State, contains_newline: bool, str: &str) -> fmt::Result {
    const TRIPLE_QUOTE: &str = "\"\"\"";
    const ESCAPED_TRIPLE_QUOTE: &str = "\\\"\"\"";
    const _: () = assert!(TRIPLE_QUOTE.len() == 3);
    const _: () = assert!(ESCAPED_TRIPLE_QUOTE.len() == 4);

    fn serialize_line(state: &mut State, mut line: &str) -> Result<(), fmt::Error> {
        while let Some((before, after)) = line.split_once(TRIPLE_QUOTE) {
            state.write(before)?;
            state.write(ESCAPED_TRIPLE_QUOTE)?;
            line = after;
        }
        state.write(line)
    }

    let multi_line =
        contains_newline || str.len() > 70 || str.ends_with('"') || str.ends_with('\\');

    state.write(TRIPLE_QUOTE)?;
    if !multi_line {
        // """example""""
        serialize_line(state, str)?
    } else {
        // """
        // example
        // """

        // `can_be_block_string` excludes \r, so the only remaining line terminator is \n
        for line in str.split('\n') {
            if line.is_empty() {
                // Skip indentation which would be trailing whitespace
                state.write("\n")?;
            } else {
                state.require_new_line()?;
                serialize_line(state, line)?;
            }
        }
        state.require_new_line()?;
    }
    state.write(TRIPLE_QUOTE)
}

/// Is it possible to create a serialization that, when fed through
/// [BlockStringValue](https://spec.graphql.org/October2021/#BlockStringValue()),
/// returns exactly `value`?
fn can_be_block_string(value: &str) -> bool {
    // `BlockStringValue` splits its inputs at any `LineTerminator` (\n, \r\n, or \r)
    // and eventually joins lines but always with \n. So its output can never contain \r
    if value.contains('\r') {
        return false;
    }

    /// <https://spec.graphql.org/October2021/#WhiteSpace>
    fn trim_start_graphql_whitespace(value: &str) -> &str {
        value.trim_start_matches([' ', '\t'])
    }

    // With the above, \n is the only remaining LineTerminator
    let mut lines = value.split('\n');
    if lines
        .next()
        .is_some_and(|first| trim_start_graphql_whitespace(first).is_empty())
        || lines
            .next_back()
            .is_some_and(|last| trim_start_graphql_whitespace(last).is_empty())
    {
        // Leading or trailing whitespace-only line would be trimmed by `BlockStringValue`
        return false;
    }

    let common_indent = {
        let lines = value.split('\n');
        let each_line_indent_utf8_len = lines.filter_map(|line| {
            let after_indent = trim_start_graphql_whitespace(line);
            if !after_indent.is_empty() {
                Some(line.len() - after_indent.len())
            } else {
                None // skip whitespace-only lines
            }
        });
        each_line_indent_utf8_len.min().unwrap_or(0)
    };
    // If there is common indent `BlockStringValue` would remove it
    // and incorrectly round-trip to a different value.
    common_indent == 0
}

fn serialize_description(state: &mut State, description: &Option<Node<str>>) -> fmt::Result {
    if let Some(description) = description {
        let is_description = true;
        serialize_string_value(state, is_description, description)?;
        state.new_line_or_space()?;
    }
    Ok(())
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Named(name) => std::write!(f, "{name}"),
            Type::NonNullNamed(name) => std::write!(f, "{name}!"),
            Type::List(inner) => std::write!(f, "[{inner}]"),
            Type::NonNullList(inner) => std::write!(f, "[{inner}]!"),
        }
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

impl fmt::Display for DirectiveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

macro_rules! impl_display {
    ($($ty: path)+) => {
        $(
            /// Serialize to GraphQL syntax with the default configuration
            impl Display for $ty {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    self.serialize().fmt(f)
                }
            }

            /// Serialize to GraphQL syntax
            impl Display for Serialize<'_, $ty> {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    let mut state = State {
                        config: self.config.clone(),
                        indent_level: self.config.initial_indent_level,
                        output: f,
                        output_empty: true,
                    };
                    // Indent the first line.
                    // Subsequent lines will be indented when writing a line break.
                    if let Some(prefix) = state.config.indent_prefix {
                        for _ in 0..state.indent_level {
                            state.write(prefix)?;
                        }
                    }
                    self.node.serialize_impl(&mut state)
                }
            }
        )+
    }
}

impl_display! {
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
    DirectiveList
    Directive
    VariableDefinition
    FieldDefinition
    InputValueDefinition
    EnumValueDefinition
    Selection
    Field
    FragmentSpread
    InlineFragment
    Value
    crate::Schema
    crate::ExecutableDocument
    schema::DirectiveList
    schema::ExtendedType
    schema::ScalarType
    schema::ObjectType
    schema::InterfaceType
    schema::EnumType
    schema::UnionType
    schema::InputObjectType
    executable::Operation
    executable::Fragment
    executable::SelectionSet
    executable::Selection
    executable::Field
    executable::InlineFragment
    executable::FragmentSpread
    executable::FieldSet
}
