<div align="center">
  <h1><code>apollo-encoder</code></h1>

  <p>
    <strong>A library to generate GraphQL Code, SDL.</strong>
  </p>
  <p>
    <a href="https://crates.io/crates/apollo-encoder">
        <img src="https://img.shields.io/crates/v/apollo-encoder.svg?style=flat-square" alt="Crates.io" />
    </a>
    <a href="https://crates.io/crates/apollo-encoder">
        <img src="https://img.shields.io/crates/d/apollo-encoder.svg?style=flat-square" alt="Download" />
    </a>
    <a href="https://docs.rs/apollo-encoder/">
        <img src="https://img.shields.io/static/v1?label=docs&message=apollo-encoder&color=blue&style=flat-square" alt="docs.rs docs" />
    </a>
  </p>
</div>

## Getting started

Add this to your `Cargo.toml` to start using `apollo-encoder`:

```toml
# Just an example, change to the necessary package version.
[dependencies]
apollo-encoder = "0.3.3"
```

Or using [cargo-edit]:

```bash
cargo add apollo-encoder
```

## Examples

### Create executable definitions
```rust
  use apollo_encoder::{
      Document, Field, FragmentSpread, InlineFragment, OperationDefinition, OperationType,
      Selection, SelectionSet, TypeCondition,
  };
  use indoc::indoc;

  let mut document = Document::new();

  let mut droid_selection_set = SelectionSet::new();
  let primary_function_field = Selection::Field(Field::new(String::from("primaryFunction")));
  droid_selection_set.selection(primary_function_field);

  let mut droid_inline_fragment = InlineFragment::new(droid_selection_set);
  droid_inline_fragment.type_condition(Some(TypeCondition::new(String::from("Droid"))));

  let comparison_fields_fragment_spread =
      FragmentSpread::new(String::from("comparisonFields"));

  let name_field = Field::new(String::from("name"));

  let hero_selection_set = vec![
      Selection::Field(name_field),
      Selection::FragmentSpread(comparison_fields_fragment_spread),
      Selection::InlineFragment(droid_inline_fragment),
  ];

  let mut hero_field = Field::new(String::from("hero"));
  hero_field.selection_set(Some(SelectionSet::with_selections(hero_selection_set)));

  let hero_for_episode_selection_set = vec![Selection::Field(hero_field)];
  let mut hero_for_episode_operation = OperationDefinition::new(
      OperationType::Query,
      SelectionSet::with_selections(hero_for_episode_selection_set),
  );
  hero_for_episode_operation.name(Some(String::from("HeroForEpisode")));

  document.operation(hero_for_episode_operation);

  assert_eq!(
      document.to_string(),
      indoc! { r#"
          query HeroForEpisode {
            hero {
              name
              ...comparisonFields
              ... on Droid {
                primaryFunction
              }
            }
          }
      "#}
  );
```

### Create type system definitions
```rust
use apollo_encoder::{
    Argument, Directive, Document, DirectiveDefinition, EnumValue,
    EnumDefinition, UnionDefinition, Value
};
use indoc::indoc;

let mut document = Document::new();

// Create a Directive Definition.
let mut directive_def = DirectiveDefinition::new("provideTreat".to_string());
directive_def.description("Ensures cats get treats.".to_string());
directive_def.location("OBJECT".to_string());
directive_def.location("FIELD_DEFINITION".to_string());
directive_def.location("INPUT_FIELD_DEFINITION".to_string());
document.directive(directive_def);

// Create an Enum Definition
let mut enum_ty_1 = EnumValue::new("CatTree".to_string());
enum_ty_1.description("Top bunk of a cat tree.".to_string());
let enum_ty_2 = EnumValue::new("Bed".to_string());
let mut enum_ty_3 = EnumValue::new("CardboardBox".to_string());
let mut directive = Directive::new(String::from("deprecated"));
directive.arg(Argument::new(
    String::from("reason"),
    Value::String("Box was recycled.".to_string()),
));
enum_ty_3.directive(directive);

let mut enum_def = EnumDefinition::new("NapSpots".to_string());
enum_def.description("Favourite cat\nnap spots.".to_string());
enum_def.value(enum_ty_1);
enum_def.value(enum_ty_2);
enum_def.value(enum_ty_3);
document.enum_(enum_def);
// Union Definition
let mut union_def = UnionDefinition::new("Pet".to_string());
union_def.description(
    "A union of all pets represented within a household.".to_string(),
);
union_def.member("Cat".to_string());
union_def.member("Dog".to_string());
document.union(union_def);

assert_eq!(
    document.to_string(),
    indoc! { r#"
"A union of all pets represented within a household."
union Pet = Cat | Dog
"""
Favourite cat
nap spots.
"""
enum NapSpots {
  "Top bunk of a cat tree."
  CatTree
  Bed
  CardboardBox @deprecated(reason: "Box was recycled.")
}
"Ensures cats get treats."
directive @provideTreat on OBJECT | FIELD_DEFINITION | INPUT_FIELD_DEFINITION

"#}
);
```
## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

[cargo-edit]: https://github.com/killercup/cargo-edit
