use apollo_compiler::request::coerce_variable_values;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Schema;
use serde_json_bytes::ByteString;
use serde_json_bytes::Map;
use serde_json_bytes::Value;

///
/// Builds and coerces a GraphQL mutation variable map for testing Float coercion behavior.
///
/// Helper function, used in unit tests to verify how GraphQL variable coercion behaves when provided
/// with different numeric types or extreme values.
///
/// It defines a minimal GraphQL schema with a `Car` type and a corresponding `CarInput` that
/// contains two `Float!` fields: `range` and `totalKilometers`.
/// The function then constructs a mutation using these input types and attempts to coerce the provided
/// values into the correct GraphQL variable format.
///
/// # Type Parameters
/// * `R` – The type of the `range` argument, must implement [`Into<f64>`].
/// * `K` – The type of the `total_kilometers` argument, must implement [`Into<f64>`].
///
/// # Arguments
/// * `range` – A numeric value (or convertible type) representing the range attribute of a car.
/// * `total_kilometers` – A numeric value (or convertible type) representing the total kilometers of a car.
///
/// # Returns
/// A [`Result`] containing the coerced [`Map<ByteString, Value>`] representing the
/// GraphQL variable map if coercion succeeds.
/// If coercion fails (for example, due to exceeding `f64` limits or type mismatches),
/// an error message is returned as a [`String`].
fn build_and_coerce_test_mutation_variables<R, K>(
    range: R,
    total_kilometers: K,
) -> Result<Map<ByteString, Value>, String>
where
    R: Into<f64>,
    K: Into<f64>,
{
    // Example schema with Float fields that will coerce ints to floats where needed.
    let sdl = r#"
      type Car { id: ID! range: Float! totalKilometers: Float! }
      input CarInput { range: Float! totalKilometers: Float! }
      type Query { getCarById(id: ID!): Car }
      type Mutation { insertACar(car: CarInput!): Car! }
    "#;

    let parsed_schema = Schema::parse_and_validate(sdl, "sdl").map_err(|e| format!("{e:?}"))?;

    // Prepare a mutation that uses the variables passed in when this function is called.
    let executable_mutation = ExecutableDocument::parse_and_validate(
        &parsed_schema,
        "mutation InsertCarMutation($car: CarInput!){ insertACar(car:$car) { id range totalKilometers } }",
        "InsertCarMutation",
    )
        .map_err(|e| format!("{e:?}"))?;

    let operation = executable_mutation
        .operations
        .get(Some("InsertCarMutation"))
        .map_err(|e| format!("{e:?}"))?;

    // Build the GraphQL variables JSON, converting both values to `f64` to match the mutation’s `Float!` fields.
    let input_variables = serde_json_bytes::json!({
        "car": {
            "range": range.into(),
            "totalKilometers": total_kilometers.into()
        }
    });

    // Extract the map for coercion.
    let map = match input_variables {
        Value::Object(m) => m,
        _ => return Err("variables JSON must be an object".into()),
    };

    //  Attempt coercion, return an error if it fails!
    let coerced =
        coerce_variable_values(&parsed_schema, operation, &map).map_err(|e| format!("{e:?}"))?;
    let vars_for_exec = coerced.into_inner();

    // Return the inner `car` object.
    vars_for_exec
        .get("car")
        .and_then(Value::as_object)
        .cloned()
        .ok_or_else(|| "coerced `car` object missing".to_string())
}

#[test]
fn test_graphql_float_variable_coercion_with_expected_float_and_int() {
    let car = build_and_coerce_test_mutation_variables(344.678_f64, 50_000_i32).unwrap();

    let range = car
        .get("range")
        .and_then(Value::as_f64)
        .expect("range as f64");
    let total_km = car
        .get("totalKilometers")
        .and_then(Value::as_f64)
        .expect("totalKilometers as f64");

    assert_eq!(
        344.678_f64, range,
        "Expected `range` to be correctly coerced into Float."
    );
    assert_eq!(
        50_000_f64, total_km,
        "Expected `totalKilometers` to be correctly coerced into Float."
    );
}

#[test]
fn test_graphql_failing_coercion_because_greater_i64_max() {
    // Use a very large integer value to simulate a value that exceeds the precision range of a 64-bit floating point number.
    // When cast to f64, this value will lose precision and trigger coercion issues in GraphQL variable validation for `Float! fields.
    let range: f64 = "170141183460469231731687303715884105727"
        .parse::<f64>()
        .expect("invalid float");

    // Provide a normal, expected value for the 2nd field.
    let total_kilometers = 50_000;

    let car = build_and_coerce_test_mutation_variables(range, total_kilometers);

    assert!(
        car.is_err(),
        "Expected coercion to fail for given 'range' and 'total_kilometers' params."
    );
}

#[test]
fn test_graphql_failing_coercion_because_infinity_value() {
    // Define a floating-point value for the `range` field.
    // This represents a typical valid input value within normal f64 precision limits.
    let range = 433.777_f64;

    // An extremely large floating-point value which is beyond the maximum representable f64 range
    // and overflows to `f64::INFINITY`.
    // This should trigger coercion validation errors when used in GraphQL input variable.
    let total_kilometers = "1e1000".parse::<f64>().expect("invalid float");

    let car = build_and_coerce_test_mutation_variables(range, total_kilometers);

    assert!(
        car.is_err(),
        "Expected coercion to fail for given 'range' and 'total_kilometers' params."
    );
}
