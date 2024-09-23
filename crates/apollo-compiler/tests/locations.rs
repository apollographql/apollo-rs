//! Test the locations of schema elements

use apollo_compiler::parser::LineColumn;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::schema::Value;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use std::ops::Range;

const DIRECTIVE_WITH_INPUTS: &str = r#"
directive @withSomeArgs(int: Int, str: String, complex: Object) on FIELD_DEFINITION

type Query {
  field: String @withSomeArgs(int: 1, str: "string", complex: { field: "value" })
  anotherField: String @withSomeArgs(
    str: """
    multiline
    """
  )
}

input Object {
  field: String
}
"#;

mod directive_inputs {
    use super::*;
    use pretty_assertions::assert_eq;

    fn schema() -> Schema {
        Schema::parse(DIRECTIVE_WITH_INPUTS, "").unwrap()
    }

    fn input_for_field<'a>(
        schema: &'a Schema,
        field_name: &str,
        argument_name: &str,
    ) -> &'a Node<Value> {
        let ExtendedType::Object(query) = &schema.types["Query"] else {
            panic!("Query was not an object");
        };
        let field = query.fields.get(field_name).unwrap();
        let directive = field.directives.get("withSomeArgs").unwrap();
        directive.specified_argument_by_name(argument_name).unwrap()
    }
    fn argument_input_location(field_name: &str, argument_name: &str) -> Range<LineColumn> {
        let schema = schema();
        let arg = input_for_field(&schema, field_name, argument_name);
        arg.line_column_range(&schema.sources).unwrap()
    }

    #[test]
    fn int() {
        assert_eq!(
            argument_input_location("field", "int"),
            Range {
                start: LineColumn {
                    line: 5,
                    column: 36
                },
                end: LineColumn {
                    line: 5,
                    column: 37
                }
            }
        );
    }

    #[test]
    fn str() {
        assert_eq!(
            argument_input_location("field", "str"),
            Range {
                start: LineColumn {
                    line: 5,
                    column: 44
                },
                end: LineColumn {
                    line: 5,
                    column: 52
                }
            }
        );
    }

    #[test]
    fn complex() {
        assert_eq!(
            argument_input_location("field", "complex"),
            Range {
                start: LineColumn {
                    line: 5,
                    column: 63
                },
                end: LineColumn {
                    line: 5,
                    column: 81
                }
            }
        );
    }

    #[test]
    fn multiline() {
        assert_eq!(
            argument_input_location("anotherField", "str"),
            Range {
                start: LineColumn {
                    line: 7,
                    column: 10
                },
                end: LineColumn { line: 9, column: 8 }
            }
        );
    }

    #[test]
    fn field_within_complex() {
        let schema = schema();
        let arg = input_for_field(&schema, "field", "complex");
        let (name, value) = &arg.as_object().unwrap()[0];
        assert_eq!(name, "field");
        assert_eq!(
            value.line_column_range(&schema.sources).unwrap(),
            Range {
                start: LineColumn {
                    line: 5,
                    column: 72
                },
                end: LineColumn {
                    line: 5,
                    column: 79
                }
            }
        );
    }
}
