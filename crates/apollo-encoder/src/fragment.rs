use std::fmt::{self, Write as _};

use crate::{Directive, SelectionSet};

/// The FragmentDefinition type represents a fragment definition
///
/// *FragmentDefinition*:
///     fragment FragmentName TypeCondition Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentDefinition).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Field, FragmentDefinition, Selection, SelectionSet, TypeCondition};
/// use indoc::indoc;
///
/// let selections = vec![Selection::Field(Field::new(String::from("myField")))];
/// let mut selection_set = SelectionSet::new();
/// selections
///     .into_iter()
///     .for_each(|s| selection_set.selection(s));
/// let mut fragment_def = FragmentDefinition::new(
///     String::from("myFragment"),
///     TypeCondition::new(String::from("User")),
///     selection_set,
/// );
///
/// assert_eq!(
///     fragment_def.to_string(),
///     indoc! {r#"
///         fragment myFragment on User {
///           myField
///         }
///     "#}
/// );
/// ```
#[derive(Debug)]
pub struct FragmentDefinition {
    name: String,
    type_condition: TypeCondition,
    directives: Vec<Directive>,
    selection_set: SelectionSet,
}

impl FragmentDefinition {
    /// Create an instance of FragmentDefinition.
    pub fn new(name: String, type_condition: TypeCondition, selection_set: SelectionSet) -> Self {
        Self {
            name,
            type_condition,
            selection_set,
            directives: Vec::new(),
        }
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for FragmentDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 0;
        write!(f, "fragment {} {}", self.name, self.type_condition)?;
        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }
        write!(
            f,
            " {}",
            self.selection_set.format_with_indent(indent_level)
        )?;

        Ok(())
    }
}

/// The FragmentSpread type represents a named fragment used in a selection set.
///
/// *FragmentSpread*:
///     ... FragmentName Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#FragmentSpread).
///
/// ### Example
/// ```rust
/// use apollo_encoder::FragmentSpread;
///
/// let fragment = FragmentSpread::new(String::from("myFragment"));
/// assert_eq!(fragment.to_string(), r#"...myFragment"#);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FragmentSpread {
    name: String,
    directives: Vec<Directive>,
}

impl FragmentSpread {
    /// Create a new instance of FragmentSpread
    pub fn new(name: String) -> Self {
        Self {
            name,
            directives: Vec::new(),
        }
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }
}

impl fmt::Display for FragmentSpread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "...{}", self.name)?;
        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        Ok(())
    }
}

/// The InlineFragment type represents an inline fragment in a selection set that could be used as a field
///
/// *InlineFragment*:
///     ... TypeCondition? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Inline-Fragments).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Field, InlineFragment, Selection, SelectionSet, TypeCondition};
/// use indoc::indoc;
///
/// let selections = vec![Selection::Field(Field::new(String::from("myField")))];
/// let mut selection_set = SelectionSet::new();
/// selections
///     .into_iter()
///     .for_each(|s| selection_set.selection(s));
///
/// let mut inline_fragment = InlineFragment::new(selection_set);
/// inline_fragment.type_condition(Some(TypeCondition::new(String::from("User"))));
///
/// assert_eq!(
///     inline_fragment.to_string(),
///     indoc! {r#"
///         ... on User {
///           myField
///         }
///     "#}
/// );
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct InlineFragment {
    type_condition: Option<TypeCondition>,
    directives: Vec<Directive>,
    selection_set: SelectionSet,
    pub(crate) indent_level: usize,
}

impl InlineFragment {
    /// Create an instance of InlineFragment
    pub fn new(selection_set: SelectionSet) -> Self {
        Self {
            selection_set,
            type_condition: Option::default(),
            directives: Vec::new(),
            indent_level: 0,
        }
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive)
    }

    /// Set the inline fragment's type condition.
    pub fn type_condition(&mut self, type_condition: Option<TypeCondition>) {
        self.type_condition = type_condition;
    }

    /// Should be used everywhere in this crate isntead of the Display implementation
    /// Display implementation is only useful as a public api
    pub(crate) fn format_with_indent(&self, indent_level: usize) -> String {
        let mut text = String::from("...");

        if let Some(type_condition) = &self.type_condition {
            let _ = write!(text, " {}", type_condition);
        }
        for directive in &self.directives {
            let _ = write!(text, " {}", directive);
        }

        let _ = write!(
            text,
            " {}",
            self.selection_set.format_with_indent(indent_level),
        );

        text
    }
}

// This impl is only useful when we generate only an InlineFragment
// If it's used from a parent element, we call `format_with_indent`
impl fmt::Display for InlineFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 0;
        write!(f, "{}", self.format_with_indent(indent_level))
    }
}

/// The TypeCondition type represents where a fragment could be applied
///
/// *TypeCondition*:
///     on NamedType
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#TypeCondition).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCondition {
    name: String,
}

impl TypeCondition {
    /// Create an instance of TypeCondition
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl fmt::Display for TypeCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "on {}", self.name)
    }
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use crate::{field::Field, Argument, Selection, Value};

    use super::*;

    #[test]
    fn it_encodes_simple_inline_fragment() {
        let selections = vec![Selection::Field(Field::new(String::from("myField")))];
        let mut selection_set = SelectionSet::new();
        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));

        let mut inline_fragment = InlineFragment::new(selection_set);
        inline_fragment.type_condition(Some(TypeCondition::new(String::from("User"))));

        assert_eq!(
            inline_fragment.to_string(),
            indoc! {r#"
                ... on User {
                  myField
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_simple_fragment_spread() {
        let fragment = FragmentSpread::new(String::from("myFragment"));

        assert_eq!(fragment.to_string(), indoc! {r#"...myFragment"#});
    }

    #[test]
    fn it_encodes_deeper_inline_fragment() {
        let another_nested_field_bis = Field::new(String::from("anotherNestedBisField"));

        let mut another_nested_field = Field::new(String::from("anotherNestedField"));
        let mut selection_set = SelectionSet::new();
        selection_set.selection(Selection::Field(another_nested_field_bis));
        another_nested_field.selection_set(selection_set.into());

        let mut selection_set = SelectionSet::new();
        selection_set.selection(Selection::Field(another_nested_field.clone()));
        another_nested_field.selection_set(selection_set.into());

        let nested_selections = vec![
            Selection::Field(Field::new(String::from("nestedField"))),
            Selection::Field(another_nested_field),
        ];
        let mut nested_selection_set = SelectionSet::new();
        nested_selections
            .into_iter()
            .for_each(|s| nested_selection_set.selection(s));

        let other_inline_fragment = InlineFragment::new(nested_selection_set);
        let selections = vec![
            Selection::Field(Field::new(String::from("myField"))),
            Selection::FragmentSpread(FragmentSpread::new(String::from("myFragment"))),
            Selection::InlineFragment(other_inline_fragment),
        ];
        let mut selection_set = SelectionSet::new();
        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));

        let mut inline_fragment = InlineFragment::new(selection_set);
        inline_fragment.type_condition(Some(TypeCondition::new(String::from("User"))));

        assert_eq!(
            inline_fragment.to_string(),
            indoc! {r#"
                ... on User {
                  myField
                  ...myFragment
                  ... {
                    nestedField
                    anotherNestedField {
                      anotherNestedField {
                        anotherNestedBisField
                      }
                    }
                  }
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_fragment_definition() {
        let selections = vec![Selection::Field(Field::new(String::from("myField")))];
        let mut selection_set = SelectionSet::new();
        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));
        let mut fragment_def = FragmentDefinition::new(
            String::from("myFragment"),
            TypeCondition::new(String::from("User")),
            selection_set,
        );
        let mut directive = Directive::new(String::from("myDirective"));
        directive.arg(Argument::new(String::from("first"), Value::Int(5)));
        fragment_def.directive(directive);

        assert_eq!(
            fragment_def.to_string(),
            indoc! {r#"
                fragment myFragment on User @myDirective(first: 5) {
                  myField
                }
            "#}
        );
    }
}
