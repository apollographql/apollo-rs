use apollo_compiler::ast;
use apollo_compiler::schema;
use apollo_compiler::schema::Component;
use apollo_compiler::schema::ComponentStr;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::ApolloCompiler;
use apollo_compiler::ReprDatabase;
use apollo_compiler::Schema;
use indexmap::IndexMap;
use indexmap::IndexSet;

type MergeError = &'static str;

/// Naively merge multiple schemas into one
fn merge_schemas(inputs: &[&str]) -> Result<String, MergeError> {
    let mut compiler = ApolloCompiler::new();
    let id = compiler.add_document("", "");
    let schemas: Vec<_> = inputs
        .iter()
        .map(|input| {
            compiler.update_document(id, input);
            compiler.db.schema()
        })
        .collect();
    let mut merged = Schema::new();
    for schema in schemas {
        merge_options(&mut merged.description, &schema.description)?;
        merge_vecs(&mut merged.directives, &schema.directives)?;
        merge_maps(
            &mut merged.directive_definitions,
            &schema.directive_definitions,
            |merged_def, new_def| {
                let _ = merged_def.location();
                let _ = new_def.location();
                Err("incompatible directive definitions")
            },
        )?;
        merge_maps(&mut merged.types, &schema.types, merge_type_definitions)?;
        merge_options(&mut merged.query_type, &schema.query_type)?;
        merge_options(&mut merged.mutation_type, &schema.mutation_type)?;
        merge_options(&mut merged.subscription_type, &schema.subscription_type)?;
    }
    Ok(merged.to_string())
}

fn merge_options<T: Eq + Clone>(merged: &mut Option<T>, new: &Option<T>) -> Result<(), MergeError> {
    match (&mut *merged, new) {
        (_, None) => {}
        (None, Some(_)) => *merged = new.clone(),
        (Some(a), Some(b)) => {
            if a != b {
                return Err("conflicting optional values");
            }
        }
    }
    Ok(())
}

fn merge_vecs<T>(merged: &mut Vec<T>, new: &Vec<T>) -> Result<(), MergeError>
where
    T: Clone + Eq,
{
    for new in new {
        if !merged.contains(new) {
            merged.push(new.clone())
        }
    }
    Ok(())
}

fn merge_maps<K, V>(
    merged: &mut IndexMap<K, V>,
    new: &IndexMap<K, V>,
    merge_values: impl Fn(&mut V, &V) -> Result<(), MergeError>,
) -> Result<(), MergeError>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone + Eq,
{
    for (key, value) in new {
        if let Some(merged_value) = merged.get_mut(key) {
            if merged_value != value {
                merge_values(merged_value, value)?;
            }
        } else {
            merged.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}

fn merge_sets(merged: &mut IndexSet<ComponentStr>, new: &IndexSet<ComponentStr>) {
    for value in new {
        if !merged.contains(value) {
            merged.insert(value.clone());
        }
    }
}

fn merge_type_definitions(merged: &mut ExtendedType, new: &ExtendedType) -> Result<(), MergeError> {
    match (merged, new) {
        (ExtendedType::Scalar(merged), ExtendedType::Scalar(new)) => {
            merge_scalar_types(merged.make_mut(), new)
        }
        (ExtendedType::Object(merged), ExtendedType::Object(new)) => {
            merge_object_types(merged.make_mut(), new)
        }
        (ExtendedType::Interface(merged), ExtendedType::Interface(new)) => {
            merge_interface_types(merged.make_mut(), new)
        }
        (ExtendedType::Union(merged), ExtendedType::Union(new)) => {
            merge_union_types(merged.make_mut(), new)
        }
        (ExtendedType::Enum(merged), ExtendedType::Enum(new)) => {
            merge_enum_types(merged.make_mut(), new)
        }
        (ExtendedType::InputObject(merged), ExtendedType::InputObject(new)) => {
            merge_input_object_types(merged.make_mut(), new)
        }
        _ => Err("incompatible kinds of types"),
    }
}

fn merge_scalar_types(
    merged: &mut schema::ScalarType,
    new: &schema::ScalarType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)
}

fn merge_object_types(
    merged: &mut schema::ObjectType,
    new: &schema::ObjectType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)?;
    merge_sets(
        &mut merged.implements_interfaces,
        &new.implements_interfaces,
    );
    merge_maps(&mut merged.fields, &new.fields, merge_fields)
}

fn merge_interface_types(
    merged: &mut schema::InterfaceType,
    new: &schema::InterfaceType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)?;
    merge_sets(
        &mut merged.implements_interfaces,
        &new.implements_interfaces,
    );
    merge_maps(&mut merged.fields, &new.fields, merge_fields)
}

fn merge_union_types(
    merged: &mut schema::UnionType,
    new: &schema::UnionType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)?;
    merge_sets(&mut merged.members, &new.members);
    Ok(())
}

fn merge_enum_types(
    merged: &mut schema::EnumType,
    new: &schema::EnumType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)?;
    merge_maps(
        &mut merged.values,
        &new.values,
        |merged_value, new_value| {
            let merged_value = merged_value.make_mut();
            merge_options(&mut merged_value.description, &new_value.description)?;
            merge_vecs(&mut merged_value.directives, &new_value.directives)
        },
    )
}

fn merge_input_object_types(
    merged: &mut schema::InputObjectType,
    new: &schema::InputObjectType,
) -> Result<(), &'static str> {
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)?;
    merge_maps(
        &mut merged.fields,
        &new.fields,
        |merged_input_field, new_input_field| {
            if (&merged_input_field.ty, &merged_input_field.default_value)
                != (&new_input_field.ty, &new_input_field.default_value)
            {
                return Err("incompatible input type field definitions");
            }
            let merged_input_field = merged_input_field.make_mut();
            merge_options(
                &mut merged_input_field.description,
                &new_input_field.description,
            )?;
            merge_vecs(
                &mut merged_input_field.directives,
                &new_input_field.directives,
            )
        },
    )
}

fn merge_fields(
    merged: &mut Component<ast::FieldDefinition>,
    new: &Component<ast::FieldDefinition>,
) -> Result<(), &'static str> {
    if (&merged.ty, &merged.arguments) != (&new.ty, &new.arguments) {
        return Err("incompatible field definitions");
    }
    let merged = merged.make_mut();
    merge_options(&mut merged.description, &new.description)?;
    merge_vecs(&mut merged.directives, &new.directives)
}

#[test]
fn test_ok() {
    let inputs = [
        r#"
            type Query {
                t: T
            }

            type T @key(fields: "k") {
                k: ID
            }

            type S {
                x: Int
            }

            union U = S | T
        "#,
        r#"
            type T @key(fields: "k") {
                k: ID
                a: Int
                b: String
            }
    
            enum E {
                V1
                V2
            }
        "#,
    ];
    let expected = r#"type Query {
  t: T
}

type T @key(fields: "k") {
  k: ID
  a: Int
  b: String
}

type S {
  x: Int
}

union U = S | T

enum E {
  V1
  V2
}
"#;
    assert_eq!(merge_schemas(&inputs).as_deref(), Ok(expected))
}
