use apollo_compiler::executable::FieldSet;
use apollo_compiler::name;
use apollo_compiler::parser::SourceSpan;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::Schema;
use expect_test::expect;

fn common_schema() -> Valid<Schema> {
    let input = r#"
        type Query {
            id: ID
            organization: Org
        }
        type Org {
            id: ID
        }
    "#;
    Schema::parse_and_validate(input, "schema.graphql").unwrap()
}

#[test]
fn test_valid_field_sets() {
    let schema = common_schema();

    let input = "id";
    FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql").unwrap();

    let input = "id organization { id }";
    FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql").unwrap();
}

#[test]
fn test_invalid_field_sets() {
    let schema = common_schema();

    let input = "name";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("type `Query` does not have a field `name`"),
        "{errors}"
    );

    let input = "id organization";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("interface, union and object types must have a subselection set"),
        "{errors}"
    );
    assert!(
        errors.contains("`Query.organization` is an object type `Org` and must select fields"),
        "{errors}"
    );

    let input = "id(arg: true)";
    let errors = FieldSet::parse_and_validate(&schema, name!("Query"), input, "field_set.graphql")
        .unwrap_err()
        .errors
        .to_string();
    assert!(
        errors.contains("the argument `arg` is not supported"),
        "{errors}"
    );
}

/// Helper: find the span of an argument value on a directive applied to a type.
fn arg_value_span(
    schema: &Schema,
    type_name: &str,
    directive_name: &str,
    arg_name: &str,
) -> SourceSpan {
    let ExtendedType::Object(obj) = &schema.types[type_name] else {
        panic!("{type_name} is not an object type");
    };
    let dir = obj
        .directives
        .iter()
        .find(|d| d.name == directive_name)
        .unwrap_or_else(|| panic!("no @{directive_name} directive on {type_name}"));
    let arg = dir
        .arguments
        .iter()
        .find(|a| a.name == arg_name)
        .unwrap_or_else(|| panic!("no {arg_name} argument"));
    arg.value
        .location()
        .expect("argument value has no source location")
}

#[test]
fn parse_at_span_valid() {
    let sdl = r#"
        directive @sel(fields: String!) on OBJECT
        type Query { product: Product }
        type Product @sel(fields: "id") { id: ID }
    "#;
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");
    let result = FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span);
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn parse_at_span_block_string_valid() {
    let sdl = r#"
        directive @sel(fields: String!) on OBJECT
        type Query { product: Product }
        type Product @sel(fields: """id""") { id: ID }
    "#;
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");
    let result = FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span);
    assert!(result.is_ok(), "{}", result.unwrap_err());
}

#[test]
fn parse_at_span_error_rendering() {
    let sdl =
        "type Product @sel(fields: \"id details { nonexistent }\") { id: ID, details: Details }\n\
               type Details { name: String }\n\
               directive @sel(fields: String!) on OBJECT\n\
               type Query { product: Product }\n";
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");

    let err =
        FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span).unwrap_err();

    expect![[r#"
        Error: interface, union and object types must have a subselection set
           ╭─[ schema.graphql:1:31 ]
           │
         1 │ type Product @sel(fields: "id details { nonexistent }") { id: ID, details: Details }
           │                               ───────────┬───────────  
           │                                          ╰───────────── `Product.details` is an object type `Details` and must select fields
        ───╯
        Error: type `Details` does not have a field `nonexistent`
           ╭─[ schema.graphql:1:41 ]
           │
         1 │ type Product @sel(fields: "id details { nonexistent }") { id: ID, details: Details }
           │                                         ─────┬─────  
           │                                              ╰─────── field `nonexistent` selected here
         2 │ type Details { name: String }
           │      ───┬───  
           │         ╰───── type `Details` defined here
           │ 
           │ Note: path to the field: `Product → details → nonexistent`
        ───╯
    "#]]
    .assert_eq(&err.errors.to_string());
}

#[test]
fn parse_at_span_block_string_error_rendering() {
    let sdl = r#"type Product @sel(fields: """
id
details { nonexistent }
""") {
  id: ID
  details: Details
}
type Details { name: String }
directive @sel(fields: String!) on OBJECT
type Query { product: Product }
"#;
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");

    let err =
        FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span).unwrap_err();

    // NOTE(@goto-bus-stop): This output is wrong because I haven't accounted for whitespace
    // stripping in block strings
    expect![[r#"
        Error: interface, union and object types must have a subselection set
           ╭─[ schema.graphql:2:3 ]
           │
         2 │ ╭─▶ id
         3 │ ├─▶ details { nonexistent }
           │ │                             
           │ ╰───────────────────────────── `Product.details` is an object type `Details` and must select fields
        ───╯
        Error: type `Details` does not have a field `nonexistent`
           ╭─[ schema.graphql:3:10 ]
           │
         3 │ details { nonexistent }
           │          ─────┬─────  
           │               ╰─────── field `nonexistent` selected here
           │ 
         8 │ type Details { name: String }
           │      ───┬───  
           │         ╰───── type `Details` defined here
           │ 
           │ Note: path to the field: `Product → details → nonexistent`
        ───╯
    "#]]
    .assert_eq(&err.errors.to_string());
}

#[test]
fn parse_at_span_not_a_string() {
    let sdl = "directive @sel(fields: Int!) on OBJECT\n\
         type Query { product: Product }\n\
         type Product @sel(fields: 42) { id: ID }\n";
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");
    let err =
        FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span).unwrap_err();

    expect![[r#"
        Error: syntax error: expected a string literal
           ╭─[ schema.graphql:3:27 ]
           │
         3 │ type Product @sel(fields: 42) { id: ID }
           │                           ─┬  
           │                            ╰── expected a string literal
        ───╯
    "#]]
    .assert_eq(&err.errors.to_string());
}

#[test]
fn parse_at_span_utf8_in_selection() {
    // A multi-byte UTF-8 character inside the selection string is a parse error
    // (GraphQL names are ASCII-only). Verify the diagnostic renders with the
    // correct byte offset and file attribution.
    let sdl = "directive @sel(fields: String!) on OBJECT\n\
               type Query { product: Product }\n\
               type Product @sel(fields: \"id\\u0020caf한글\") { id: ID }\n";
    let schema = Schema::parse_and_validate(sdl, "schema.graphql").unwrap();
    let value_span = arg_value_span(&schema, "Product", "sel", "fields");
    let err =
        FieldSet::parse_and_validate_at_span(&schema, name!("Product"), value_span).unwrap_err();

    expect![[r#"
        Error: type `Product` does not have a field `caf`
           ╭─[ schema.graphql:3:31 ]
           │
         3 │ type Product @sel(fields: "id café") { id: ID }
           │      ───┬───                  ─┬─  
           │         ╰────────────────────────── type `Product` defined here
           │                                │   
           │                                ╰─── field `caf` selected here
           │ 
           │ Note: path to the field: `Product → caf`
        ───╯
        Error: syntax error: Unexpected character "é"
           ╭─[ schema.graphql:3:34 ]
           │
         3 │ type Product @sel(fields: "id café") { id: ID }
           │                                  ┬  
           │                                  ╰── Unexpected character "é"
        ───╯
    "#]]
    .assert_eq(&err.errors.to_string());
}
