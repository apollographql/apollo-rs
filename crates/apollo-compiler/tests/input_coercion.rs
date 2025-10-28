use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use serde_json::Number;

#[test]
fn test_graphql_float_variable_coercion() {
    // Small schema with a Float in the input object
    let sdl = r#"
      type Car { id: ID! totalKilometers: Float! }
      input CarInput { totalKilometers: Float! }
      type Query { getCarById(id: ID!): Car }
      type Mutation { insertACar(car: CarInput!): Car! }
    "#;

    let parsed_schema = Schema::parse_and_validate(sdl, "sdl").unwrap();

    let executable_mutation = ExecutableDocument::parse_and_validate(
        &parsed_schema,
        "mutation MyCarInsertMutation($car: CarInput!){ insertACar(car:$car) { id totalKilometers } }",
        "MyCarInsertMutation",
    )
        .unwrap();

    let operation = executable_mutation
        .operations
        .get(Some("MyCarInsertMutation"))
        .unwrap();

    // Case 1: Run coercion for a valid float and assert success:

    let total_kilometers_value = 3000;

    let input_variables_valid_float =
        serde_json_bytes::json!({ "car": { "totalKilometers": total_kilometers_value } });
    let map_valid_float = match input_variables_valid_float {
        serde_json_bytes::Value::Object(m) => m,
        _ => unreachable!(),
    };

    let coerced_valid_float =
        coerce_variable_values(&parsed_schema, operation, &map_valid_float).unwrap();
    let vars_for_exec_valid_float = coerced_valid_float.into_inner();

    let car = vars_for_exec_valid_float
        .get("car")
        .and_then(|value| value.as_object())
        .expect("coerced `car` object");
    assert_eq!(
        car.get("totalKilometers").unwrap(),
        total_kilometers_value,
        "totalKilometers should be present and a valid amount."
    );
}
