# Changelog

All notable changes to `apollo-encoder` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
# [0.2.1](https://crates.io/crates/apollo-encoder/0.2.1) - 2021-02-17
## Fixes

- **Remove leading and ending `"` in `BlockStringCharacter` encoding - [bnjjj], [pull/168]**
  This ensures that a StringValue of type BlockStringCharacter, like the one in
  the test example below, does not getting encoded with an additional `"` ending
  up with `""""` leading set of characters.

  ```rust
  let desc = StringValue::Top {
      source: Some(
          "\"Favourite nap spots include: plant corner, pile of clothes.\""
              .to_string(),
      ),
  };

  assert_eq!(
      desc.to_string(),
      r#""""
  Favourite  nap spots include: plant corner, pile of clothes.
  """
  "#
    );

  [bnjjj]: https://github.com/bnjjj
  [pull/168]: https://github.com/apollographql/apollo-rs/pull/168
  
# [0.2.0](https://crates.io/crates/apollo-encoder/0.2.0) - 2021-02-11

> Important: 5 breaking changes below, indicated by **BREAKING**

## BREAKING
- **`Schema` is removed in favour of `Document` - [bnjjj], [pull/162] [issue/161]**

  To align closer to GraphQL spec, we have removed the `Schema` type and instead created a `Document` type to fully represent all GraphQL types in a given document.
  The previous version of `apollo-encoder` used the schema type to create type system definitions.

  **Example of using `Document` to create type system definitions:**
  ```rust
  use apollo_encoder::{
    Argument, Directive, Document, DirectiveDefinition, EnumValue,
    EnumDefinition, UnionDefinition, Value
  };
  use indoc::indoc;

  let mut document = Document::new();

  // Create an Enum Definition
  let enum_ty_1 = EnumValue::new("Bed".to_string());
  let mut enum_ty_2 = EnumValue::new("CardboardBox".to_string());
  let mut directive = Directive::new(String::from("deprecated"));
  directive.arg(Argument::new(
      String::from("reason"),
      Value::String("Box was recycled.".to_string()),
  ));
  enum_ty_2.directive(directive);

  let mut enum_def = EnumDefinition::new("NapSpots".to_string());
  enum_def.description(Some("Favourite cat\nnap spots.".to_string()));
  enum_def.value(enum_ty_1);
  enum_def.value(enum_ty_2);
  document.enum_(enum_def);
  // Union Definition
  let mut union_def = UnionDefinition::new("Pet".to_string());
  union_def.description(Some(
      "A union of all pets represented within a household.".to_string(),
  ));
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
    CardboardBox @deprecated(reason: "Box was recycled.")
  }

  "#}
  );
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161

- **All types with `Def` suffix were replaced with `Definition` suffix - [bnjjj], [pull/169] [issue/167]**

  `Definition` suffix should provide extra clarity to all types created in `apollo-encoder` and how the relate to the spec.

  [bnjjj]: https://github.com/bnjjj
  [pull/169]: https://github.com/apollographql/apollo-rs/pull/169
  [issue/167]: https://github.com/apollographql/apollo-rs/issues/167

- **`InputField` and `Field` are now separate types - [bnjjj], [pull/162] [issue/161]**

  `InputField` represents a field type within an `InputObject`, where as `Field` is a type within a `SelectionSet`. The two are not interchangeable according to the spec, and we have now corrected this behaviour to adhere to the spec.

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161

- **`deprecated` method is removed - [bnjjj], [pull/162] [issue/161]**

  `deprecated` method to add a deprecated directive to a given type no longer exists on `InputValueDefinition`, `EnumValueDefinition`, and `FieldDefinition`. A `Directive` type was instead added to account for any number of directives that could be on a given type.

  **Example of adding a deprecated directive to a Field Definition:**
  ```rust
  // toys: DanglerPoleToys @deprecated(reason: """"DanglerPoleToys" are no longer interesting""")
  let ty = Type_::NamedType {
      name: "DanglerPoleToys".to_string(),
  };
  let mut field = FieldDefinition::new("toys".to_string(), ty);
  let mut deprecated_directive = Directive::new(String::from("deprecated"));
  let argument = Argument::new(
      String::from("reason"),
      Value::String(String::from(
          "\"DanglerPoleToys\" are no longer entertaining",
      )),
  );
  deprecated_directive.arg(argument);
  field.directive(deprecated_directive);
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161

## Features

- **Adding support to missing types outlined in GraphQL spec - [bnjjj], [pull/162] [issue/161]**

  The following types are now part of `apollo-encoder`:
    - `OperationDefinition`
    - `FragmentDefinition`
    - `Value`
    - `VariableDefinition`
    - `DirectiveDefinition`
    - `SelectionSet`

  **Example of creating an Operation Definition:**
  ```rust
  use apollo_encoder::{
      Argument, Directive, Document, Field, OperationDefinition, OperationType, Selection, SelectionSet, Type_, Value,
      VariableDefinition,
  };
  use indoc::indoc;
  let mut document = Document::new();
  let selection_set = {
      let sels = vec![
          Selection::Field(Field::new(String::from("first"))),
          Selection::Field(Field::new(String::from("second"))),
      ];
      let mut sel_set = SelectionSet::new();
      sels.into_iter().for_each(|sel| sel_set.selection(sel));
      sel_set
  };
  let var_def = VariableDefinition::new(
      String::from("variable_def"),
      Type_::List {
          ty: Box::new(Type_::NamedType {
              name: String::from("Int"),
          }),
      },
  );
  let mut new_op = OperationDefinition::new(OperationType::Query, selection_set);
  let mut directive = Directive::new(String::from("testDirective"));
  directive.arg(Argument::new(
      String::from("first"),
      Value::String("one".to_string()),
  ));
  new_op.variable_definition(var_def);
  new_op.directive(directive);
  document.operation(new_op);
  assert_eq!(
      document.to_string(),
      indoc! { r#"
          query($variable_def: [Int]) @testDirective(first: "one") {
            first
            second
          }
      "#}
  );
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161

- **Adding support to Type System Extensions - [bnjjj], [pull/162] [issue/161]**

  The following types now have extensions:
    - `Schema`
    - `Scalar`
    - `Interface`
    - `Union`
    - `Enum`
    - `Input`

  **Example of creating an extension for a `Scalar` type:**
  ```rust
  use apollo_encoder::{Argument, Directive, ScalarDefinition, Value};
  let mut scalar = ScalarDefinition::new("NumberOfTreatsPerDay".to_string());
  scalar.description(Some(
      "Int representing number of treats received.".to_string(),
  ));
  scalar.extend();

  let mut directive = Directive::new(String::from("tag"));
  directive.arg(Argument::new(
      String::from("name"),
      Value::String("team-admin".to_string()),
  ));
  scalar.directive(directive);

  assert_eq!(
      scalar.to_string(),
      r#"extend scalar NumberOfTreatsPerDay @tag(name: "team-admin")
  "#
  );
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161

- **`Directives` can be added to applicable types - [bnjjj], [pull/162] [issue/161]**

  The following types now have `Directives` support:
    - `SchemaDefinition`
    - `InputValueDefinition`
    - `ScalarDefinition`
    - `ObjectDefinition`
    - `InterfaceDefinition`
    - `UnionDefinition`
    - `EnumDefinition`
    - `InputObjectDefinition`

  [bnjjj]: https://github.com/bnjjj
  [pull/162]: https://github.com/apollographql/apollo-rs/pull/162
  [issue/161]: https://github.com/apollographql/apollo-rs/issues/161
