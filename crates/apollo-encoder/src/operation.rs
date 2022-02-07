use std::fmt;

use crate::{Directive, SelectionSet, VariableDef};

/// The __operationDef type represents an operation definition
///
/// *OperationDefinition*:
///     OperationType Name? VariableDefinitions? Directives? SelectionSet
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Language.Operations).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Argument, Field, InlineFragment, Directive, OperationDef, OperationType, Selection, SelectionSet, TypeCondition, Type_, Value, VariableDef};
/// use indoc::indoc;
///
/// let selection_set = {
///     let sels = vec![
///         Selection::Field(Field::new(String::from("first"))),
///         Selection::Field(Field::new(String::from("second"))),
///     ];
///     let mut sel_set = SelectionSet::new();
///     sels.into_iter().for_each(|sel| sel_set.selection(sel));
///
///     sel_set
/// };
/// let var_def = VariableDef::new(
///     String::from("variable_def"),
///     Type_::List {
///         ty: Box::new(Type_::NamedType {
///             name: String::from("Int"),
///         }),
///     },
/// );
/// let mut new_op = OperationDef::new(OperationType::Query, selection_set);
/// let mut directive = Directive::new(String::from("testDirective"));
/// directive.arg(Argument::new(
///     String::from("first"),
///     Value::String("one".to_string()),
/// ));
/// new_op.variable_definition(var_def);
/// new_op.directive(directive);
///
/// assert_eq!(
///     new_op.to_string(),
///     indoc! { r#"
///         query($variable_def: [Int]) @testDirective(first: "one") {
///           first
///           second
///         }
///     "#}
/// );
/// ```
#[derive(Debug)]
pub struct OperationDef {
    operation_type: OperationType,
    name: Option<String>,
    variable_definitions: Vec<VariableDef>,
    directives: Vec<Directive>,
    selection_set: SelectionSet,
}

impl OperationDef {
    /// Create a new instance of OperationDef
    pub fn new(operation_type: OperationType, selection_set: SelectionSet) -> Self {
        Self {
            operation_type,
            selection_set,
            name: None,
            variable_definitions: Vec::new(),
            directives: Vec::new(),
        }
    }

    /// Set the operation def's name.
    pub fn name(&mut self, name: Option<String>) {
        self.name = name;
    }

    /// Add a variable definitions.
    pub fn variable_definition(&mut self, variable_definition: VariableDef) {
        self.variable_definitions.push(variable_definition);
    }

    /// Add a directive.
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }
}

impl fmt::Display for OperationDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let indent_level = 0;

        write!(f, "{}", self.operation_type)?;
        if let Some(name) = &self.name {
            write!(f, " {}", name)?;
        }
        if !self.variable_definitions.is_empty() {
            write!(f, "(")?;
            for (i, var_def) in self.variable_definitions.iter().enumerate() {
                if i == self.variable_definitions.len() - 1 {
                    write!(f, "{}", var_def)?;
                } else {
                    write!(f, "{}, ", var_def)?;
                }
            }
            write!(f, ")")?;
        }
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

/// The __operationType type represents the kind of operation
///
/// *OperationType*:
///     query | mutation | subscription
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#OperationType).
#[derive(Debug)]
pub enum OperationType {
    /// Represents a query operation
    Query,
    /// Represents a mutation operation
    Mutation,
    /// Represents a subscription operation
    Subscription,
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationType::Query => write!(f, "query"),
            OperationType::Mutation => write!(f, "mutation"),
            OperationType::Subscription => write!(f, "subscription"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{field::Field, Argument, FragmentSpread, Selection, Type_, Value};
    use indoc::indoc;

    #[test]
    fn it_encodes_a_query_operation() {
        let selection_set = {
            let sels = vec![
                Selection::Field(Field::new(String::from("first"))),
                Selection::Field(Field::new(String::from("second"))),
            ];
            let mut sel_set = SelectionSet::new();
            sels.into_iter().for_each(|sel| sel_set.selection(sel));

            sel_set
        };
        let var_def = VariableDef::new(
            String::from("variable_def"),
            Type_::List {
                ty: Box::new(Type_::NamedType {
                    name: String::from("Int"),
                }),
            },
        );
        let mut new_op = OperationDef::new(OperationType::Query, selection_set);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));
        new_op.variable_definition(var_def);
        new_op.directive(directive);

        assert_eq!(
            new_op.to_string(),
            indoc! { r#"
                query($variable_def: [Int]) @testDirective(first: "one") {
                  first
                  second
                }
            "#}
        );
    }

    #[test]
    fn it_encodes_a_deeper_query_operation() {
        // ----- Selection set creation
        let fourth_field = Field::new("fourth".to_string());
        let mut third_field = Field::new("third".to_string());
        third_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            fourth_field,
        )])));
        let mut second_field = Field::new("second".to_string());
        second_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            third_field,
        )])));

        let mut first_field = Field::new("first".to_string());
        first_field.selection_set(Some(SelectionSet::with_selections(vec![Selection::Field(
            second_field,
        )])));

        let selections = vec![
            Selection::Field(first_field),
            Selection::FragmentSpread(FragmentSpread::new(String::from("myFragment"))),
        ];
        let mut selection_set = SelectionSet::new();
        selections
            .into_iter()
            .for_each(|s| selection_set.selection(s));
        // -------------------------

        let var_def = VariableDef::new(
            String::from("variable_def"),
            Type_::List {
                ty: Box::new(Type_::NamedType {
                    name: String::from("Int"),
                }),
            },
        );
        let mut new_op = OperationDef::new(OperationType::Query, selection_set);
        let mut directive = Directive::new(String::from("testDirective"));
        directive.arg(Argument::new(
            String::from("first"),
            Value::String("one".to_string()),
        ));
        new_op.variable_definition(var_def);
        new_op.directive(directive);

        assert_eq!(
            new_op.to_string(),
            indoc! { r#"
                query($variable_def: [Int]) @testDirective(first: "one") {
                  first {
                    second {
                      third {
                        fourth
                      }
                    }
                  }
                  ...myFragment
                }
            "#}
        );
    }
}
