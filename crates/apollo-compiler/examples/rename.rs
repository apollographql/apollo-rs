//! This example shows how to rename a type definition

use apollo_compiler::name;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::Schema;

#[cfg(not(test))]
fn main() {
    print!("{}", renamed())
}

fn renamed() -> Valid<Schema> {
    let input = "type Query { field: Int }";
    let mut schema = Schema::parse(input, "schema.graphql").unwrap();

    // 1. Remove the definition from the `types` map, using its old name as a key
    let mut type_def = schema.types.remove("Query").unwrap();

    // 2. Set the new name in the struct
    let ExtendedType::Object(obj) = &mut type_def else {
        panic!("expected an object type")
    };
    let new_name = name!("MyQuery");
    obj.make_mut().name = new_name.clone();

    // 3. Insert back into the map using the new name as the key
    // WARNING: it’s your responsibility to make sure to use the same name as in the struct!
    // Failing to do so make cause code elsewhere to behave incorrectly, or potentially panic.
    schema.types.insert(new_name.clone(), type_def);

    // 4. Update any existing reference to the old name
    schema
        .schema_definition
        .make_mut()
        .query
        .as_mut()
        .unwrap()
        .name = new_name;

    schema.validate().unwrap()
}

#[test]
fn test_renamed() {
    let expected = expect_test::expect![[r#"
        schema {
          query: MyQuery
        }

        type MyQuery {
          field: Int
        }
    "#]];
    expected.assert_eq(&renamed().to_string());
}
