use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;

#[test]
fn test_graphql_float_variable_coercion() {
    // Small schema with a Float in the input object
    let sdl = r#"
      type Car { id: ID! kilometers: Float! }
      input CarInput { kilometers: Float! }
      type Query { getCarById(id: ID!): Car }
      type Mutation { insertACar(car: CarInput!): Car! }
    "#;

    let parsed_schema = Schema::parse_and_validate(sdl, "sdl").unwrap();

    let executable_mutation = ExecutableDocument::parse_and_validate(
        &parsed_schema,
        "mutation MyCarInsertMutation($car: CarInput!){ insertACar(car:$car) { id kilometers } }",
        "MyCarInsertMutation",
    )
    .unwrap();

    let operation = executable_mutation
        .operations
        .get(Some("MyCarInsertMutation"))
        .unwrap();

    let kilometers_value = 3000;

    // Provide an integer for a Float field
    let input_variables = serde_json_bytes::json!({ "car": { "kilometers": kilometers_value } });
    let map = match input_variables {
        serde_json_bytes::Value::Object(m) => m,
        _ => unreachable!(),
    };

    // Coerce and validate.
    let coerced = coerce_variable_values(&parsed_schema, operation, &map).unwrap();
    let vars_for_exec = coerced.into_inner();

    // ---- Assertions ----
    let car = vars_for_exec
        .get("car")
        .and_then(|value| value.as_object())
        .expect("coerced `car` object");
    assert_eq!(
        car.get("kilometers").unwrap(),
        kilometers_value,
        "kilometers should be present and a valid amount."
    );
}
