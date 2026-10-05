//! Checks input object cycle validation against a direct port of the spec's
//! [`InputObjectHasUnbreakableCycle()`](https://spec.graphql.org/draft/#InputObjectHasUnbreakableCycle()).
//! The compiler computes the same result with a different algorithm, so this
//! guards against the two drifting apart.

use apollo_compiler::Schema;
use proptest::prelude::*;

#[derive(Debug, Clone)]
enum FieldType {
    Scalar,
    List(usize),
    Named(usize),
    NonNull(usize),
}

#[derive(Debug, Clone)]
struct InputObject {
    one_of: bool,
    fields: Vec<FieldType>,
}

fn input_objects() -> impl Strategy<Value = Vec<InputObject>> {
    (1..=6usize)
        .prop_flat_map(|count| {
            let field = prop_oneof![
                Just(FieldType::Scalar),
                (0..count).prop_map(FieldType::List),
                (0..count).prop_map(FieldType::Named),
                (0..count).prop_map(FieldType::NonNull),
            ];
            // Empty input objects are rejected on their own, and the compiler
            // deliberately does not also report an empty @oneOf as a cycle.
            prop::collection::vec((any::<bool>(), prop::collection::vec(field, 1..=3)), count)
        })
        .prop_map(|types| {
            types
                .into_iter()
                .map(|(one_of, fields)| {
                    // @oneOf fields must be nullable, otherwise the schema has
                    // unrelated errors.
                    let fields = fields
                        .into_iter()
                        .map(|field| match field {
                            FieldType::NonNull(index) if one_of => FieldType::Named(index),
                            field => field,
                        })
                        .collect();
                    InputObject { one_of, fields }
                })
                .collect()
        })
}

fn to_sdl(types: &[InputObject]) -> String {
    let mut sdl = String::from("type Query { f: Int }\n");
    for (index, ty) in types.iter().enumerate() {
        let one_of = if ty.one_of { " @oneOf" } else { "" };
        sdl.push_str(&format!("input T{index}{one_of} {{"));
        for (field_index, field) in ty.fields.iter().enumerate() {
            let field_type = match field {
                FieldType::Scalar => "Int".to_string(),
                FieldType::List(target) => format!("[T{target}]"),
                FieldType::Named(target) => format!("T{target}"),
                FieldType::NonNull(target) => format!("T{target}!"),
            };
            sdl.push_str(&format!(" f{field_index}: {field_type}"));
        }
        sdl.push_str(" }\n");
    }
    sdl
}

fn input_object_has_unbreakable_cycle(
    types: &[InputObject],
    index: usize,
    visited: &mut Vec<usize>,
) -> bool {
    if visited.contains(&index) {
        return true;
    }
    visited.push(index);
    let ty = &types[index];
    let result = if ty.one_of {
        ty.fields
            .iter()
            .all(|field| input_field_type_has_unbreakable_cycle(types, field, visited))
    } else {
        ty.fields.iter().any(|field| {
            matches!(field, FieldType::NonNull(_))
                && input_field_type_has_unbreakable_cycle(types, field, visited)
        })
    };
    visited.pop();
    result
}

fn input_field_type_has_unbreakable_cycle(
    types: &[InputObject],
    field: &FieldType,
    visited: &mut Vec<usize>,
) -> bool {
    match field {
        FieldType::Scalar | FieldType::List(_) => false,
        FieldType::Named(index) | FieldType::NonNull(index) => {
            input_object_has_unbreakable_cycle(types, *index, visited)
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2048))]

    #[test]
    fn matches_spec_algorithm(types in input_objects()) {
        let sdl = to_sdl(&types);
        let errors = match Schema::parse_and_validate(&sdl, "schema.graphql") {
            Ok(_) => String::new(),
            Err(with_errors) => with_errors.errors.to_string(),
        };
        for index in 0..types.len() {
            let expected = input_object_has_unbreakable_cycle(&types, index, &mut Vec::new());
            let reported =
                errors.contains(&format!("`T{index}` input object cannot be constructed because of an unbreakable cycle"));
            prop_assert_eq!(reported, expected, "T{} in:\n{}\nerrors:\n{}", index, sdl, errors);
        }
    }
}
