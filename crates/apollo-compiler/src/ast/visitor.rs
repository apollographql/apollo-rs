use super::*;

/// Visitor for an AST. Implement this trait and pass it to
/// document.walk(visitor). This visitor is expected to mutate the AST, so each
/// call gets `&mut` (and the AST gets fully copied-on-write as it walks it).
/// Visitors cannot control the walk or return a value, but they can mutate
/// their nodes. It is a preorder traversal; any changes made by a `visit_*`
/// function affect which children are visited.
trait Visitor {
    fn visit_document(&mut self, _document: &mut Document) {}
    fn visit_operation_definition(&mut self, _operation_definition: &mut OperationDefinition) {}
    fn visit_fragment_definition(&mut self, _fragment_definition: &mut FragmentDefinition) {}
    fn visit_directive_definition(&mut self, _directive_definition: &mut DirectiveDefinition) {}
    fn visit_arguments(&mut self, _argument: &mut Vec<Node<Argument>>) {}
    fn visit_argument(&mut self, _argument: &mut Argument) {}
    fn visit_directives(&mut self, _directives: &mut Directives) {}
    fn visit_directive(&mut self, _directive: &mut Directive) {}
    fn visit_operation_type(&mut self, _operation_type: &mut OperationType) {}
    fn visit_directive_location(&mut self, _directive_location: &mut DirectiveLocation) {}
    fn visit_variable_definition(&mut self, _variable_definition: &mut VariableDefinition) {}
    // XXX Maybe should visit the individual enum variants instead?
    // We aren't calling this recursively right now.
    fn visit_type(&mut self, _ty: &mut Type) {}
    fn visit_input_value_definition(&mut self, _input_value_definition: &mut InputValueDefinition) {
    }
    fn visit_selection_set(&mut self, _selection_set: &mut Vec<Selection>) {}
    fn visit_field(&mut self, _field: &mut Field) {}
    fn visit_fragment_spread(&mut self, _fragment_spread: &mut FragmentSpread) {}
    fn visit_inline_fragment(&mut self, _inline_fragment: &mut InlineFragment) {}
    fn visit_value(&mut self, _value: &mut Value) {}
}

/// Trait encapsulating the traversing logic for the AST
trait Walkable {
    /// Visit children of the current node
    fn walk<V: Visitor>(&mut self, visitor: &mut V);
}

impl Walkable for Document {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_document(self);
        for definition in &mut self.definitions {
            definition.walk(visitor);
        }
    }
}

impl Walkable for Definition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        match self {
            Definition::OperationDefinition(node) => node.make_mut().walk(visitor),
            Definition::FragmentDefinition(node) => node.make_mut().walk(visitor),
            Definition::DirectiveDefinition(node) => node.make_mut().walk(visitor),
            _ => panic!("SDL visitors not implemented"),
        }
    }
}

impl Walkable for OperationDefinition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_operation_definition(self);
        self.operation_type.walk(visitor);
        // #MoreSpecificTypes self.name.walk(visitor);
        for variable in &mut self.variables {
            variable.make_mut().walk(visitor);
        }
        self.directives.walk(visitor);
        self.selection_set.walk(visitor);
    }
}

impl Walkable for FragmentDefinition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_fragment_definition(self);
        // #MoreSpecificTypes self.name.walk(visitor);
        // #MoreSpecificTypes self.type_condition.walk(visitor);
        self.directives.walk(visitor);
        self.selection_set.walk(visitor);
    }
}

impl Walkable for DirectiveDefinition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_directive_definition(self);
        // #MoreSpecificTypes self.description.walk(visitor);
        // #MoreSpecificTypes self.name.walk(visitor);
        for argument in &mut self.arguments {
            argument.make_mut().walk(visitor);
        }
        for location in &mut self.locations {
            location.walk(visitor);
        }
    }
}

impl Walkable for Vec<Node<Argument>> {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_arguments(self);
        for argument in self {
            argument.make_mut().walk(visitor);
        }
    }
}

impl Walkable for Argument {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_argument(self);
        // #MoreSpecificTypes self.name.walk(visitor);
        self.value.make_mut().walk(visitor);
    }
}

impl Walkable for Directives {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_directives(self);
        for directive in &mut self.0 {
            directive.make_mut().walk(visitor);
        }
    }
}

impl Walkable for Directive {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_directive(self);
        // #MoreSpecificTypes self.name.walk(visitor);
        self.arguments.walk(visitor);
    }
}

impl Walkable for OperationType {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_operation_type(self);
    }
}

impl Walkable for DirectiveLocation {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_directive_location(self);
    }
}

impl Walkable for VariableDefinition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_variable_definition(self);
        // #MoreSpecificTypes self.name.walk(visitor);
        self.ty.make_mut().walk(visitor);
        if let Some(value) = self.default_value.as_mut() {
            value.make_mut().walk(visitor)
        }
        self.directives.walk(visitor);
    }
}

impl Walkable for Type {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_type(self);
        // No recursive walks for the moment (including to the name). #MoreSpecificTypes
    }
}

impl Walkable for InputValueDefinition {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_input_value_definition(self);
        // #MoreSpecificTypes self.name.walk(visitor);
        self.ty.make_mut().walk(visitor);
        if let Some(value) = self.default_value.as_mut() {
            value.make_mut().walk(visitor)
        }
        self.directives.walk(visitor);
    }
}

impl Walkable for Vec<Selection> {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_selection_set(self);
        for selection in self {
            selection.walk(visitor);
        }
    }
}

impl Walkable for Selection {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        match self {
            Selection::Field(node) => node.make_mut().walk(visitor),
            Selection::FragmentSpread(node) => node.make_mut().walk(visitor),
            Selection::InlineFragment(node) => node.make_mut().walk(visitor),
        }
    }
}

impl Walkable for Field {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_field(self);
        // #MoreSpecificTypes self.alias.walk(visitor);
        // #MoreSpecificTypes self.name.walk(visitor);
        self.arguments.walk(visitor);
        self.directives.walk(visitor);
        self.selection_set.walk(visitor);
    }
}

impl Walkable for FragmentSpread {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_fragment_spread(self);
        // #MoreSpecificTypes self.fragment_name.walk(visitor);
        self.directives.walk(visitor);
    }
}

impl Walkable for InlineFragment {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        visitor.visit_inline_fragment(self);
        // #MoreSpecificTypes self.type_condition.walk(visitor);
        self.directives.walk(visitor);
        self.selection_set.walk(visitor);
    }
}

impl Walkable for Value {
    fn walk<V: Visitor>(&mut self, visitor: &mut V) {
        // #MoreSpecificTypes I would rather have visitors for StringValue,
        // IntValue, etc, instead of a Value visitor that has to match.
        visitor.visit_value(self);
        match self {
            Value::List(values) => {
                for value in values {
                    value.make_mut().walk(visitor);
                }
            }
            Value::Object(values) => {
                for (_, value) in values {
                    value.make_mut().walk(visitor);
                }
            }
            // Nothing else is needed for non-recursive cases.
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Diagnostics;

    use super::*;

    struct HideStringAndNumericLiterals;

    impl Visitor for HideStringAndNumericLiterals {
        fn visit_value(&mut self, value: &mut Value) {
            // This is awkward; I'd prefer to define visit_int_value, etc.
            match value {
                Value::Int(int_value) => int_value.0 = "0".to_string(),
                Value::Float(float_value) => float_value.0 = "0".to_string(),
                Value::String(node_str) => *node_str = NodeStr::new(""),
                _ => {}
            }
        }
    }

    #[test]
    fn it_hides_string_and_numeric_literals() -> Result<(), Diagnostics> {
        let input = r#"
          query Foo($b: Int = 5, $a: Boolean, $c: String = "asdf", $d: Float = 0.5) {
            user(
              name: "hello"
              age: 5
              pct: 0.4
              lst: ["a", "b", "c"]
              obj: { a: "a", b: 1 }
            ) {
              ...Bar
              ... on User {
                hello
                bee
              }
              tz @someDirective(a: [500])
              aliased: name
              withInputs(
                str: "hi"
                int: 2
                flt: 0.3
                lst: ["", "", ""]
                obj: { q: "", s: 0 }
              )
            }
          }
  
          fragment Bar on User {
            age @skip(if: $a)
            ...Nested
          }
  
          fragment Nested on User {
            blah
          }
        "#;

        let expected_output = r#"
          query Foo($b: Int = 0, $a: Boolean, $c: String = "", $d: Float = 0) {
            user(
              name: ""
              age: 0
              pct: 0
              lst: ["", "", ""]
              obj: { a: "", b: 0 }
            ) {
              ...Bar
              ... on User {
                hello
                bee
              }
              tz @someDirective(a: [0])
              aliased: name
              withInputs(
                str: ""
                int: 0
                flt: 0
                lst: ["", "", ""]
                obj: { q: "", s: 0 }
              )
            }
          }
  
          fragment Bar on User {
            age @skip(if: $a)
            ...Nested
          }
  
          fragment Nested on User {
            blah
          }
        "#;
        let mut document = Document::parse(input, "input.graphql");
        document.check_parse_errors()?;
        document.walk(&mut HideStringAndNumericLiterals);

        // We don't care about formatting differences, so normalize the above
        // expected output.
        let normalized_expected_output =
            Document::parse(expected_output, "expected_output.graphql").to_string();

        assert_eq!(document.to_string(), normalized_expected_output);

        Ok(())
    }
}
