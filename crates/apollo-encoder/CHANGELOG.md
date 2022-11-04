# Changelog

All notable changes to `apollo-encoder` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2022-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
# [0.3.4](https://crates.io/crates/apollo-encoder/0.3.4) - 2022-11-04
## Maintenance
- **apollo-parser@0.3.1 - [lrlna], [pull/348]**

  [lrlna]: https://github.com/lrlna
  [pull/348]: https://github.com/apollographql/apollo-rs/pull/348

# [0.3.3](https://crates.io/crates/apollo-encoder/0.3.3) - 2022-10-31

## Features
- **provide TryFrom<parser types> for encoder types - [goto-bus-stop] [pull/329]**
  Provides `TryFrom` impls for the apollo-encoder AST types.

  The conversion effectively assumes that the apollo-parser AST is valid and
  complete. Otherwise you get an Err, but not a very useful one, because the
  TryFrom impl doesn't know anything about its parent (so we can't show source
  code where the error originated for example).

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/329]: https://github.com/apollographql/apollo-rs/pull/329
# [0.3.2](https://crates.io/crates/apollo-encoder/0.3.2) - 2022-08-19

## Fixes
- **trim double quotes in multilingual description encodings - [lrlna]**

  Mutilingual descriptions failed to be encoded when containing block string
  characters. The encoder now works with block string multilingual descriptions
  such as:

  ```graphql
  """
  котя любить дрімати в "кутку" з рослинами
  """
  ```

  [lrlna]: https://github.com/lrlna

# [0.3.1](https://crates.io/crates/apollo-encoder/0.3.1) - 2022-04-29

## Fixes
- **directive definition args are of ArgumentsDefinition type - [lrlna], [pull/211]**

  Directive Definition incorrectly had a `Vec<InputValueDefinition>` for internal
  arguments type instead of `ArgumentsDefinition`. This commit aligns this bit to
  the spec and uses `ArgumentsDefinition` as type.

  [lrlna]: https://github.com/lrlna
  [pull/211]: https://github.com/apollographql/apollo-rs/pull/211

# [0.3.0](https://crates.io/crates/apollo-encoder/0.3.0) - 2022-04-28

> Important: 4 breaking change below, indicated by **BREAKING**

## BREAKING

- **ArgumentsDefinition::new() creates an empty instance of ArgumentsDefinition - [lrlna], [pull/207]**

  ArgumentsDefinition::new() now takes no arguments and creates a default empty
  vector of input value definitions. Previously, `::new()` would accept a vector of input value definitions. This API is now represented as `::with_values()`.

  [lrlna]: https://github.com/lrlna
  [pull/207]: https://github.com/apollographql/apollo-rs/pull/207

- **all descriptions setters accept paramater of type String - [lrlna], [pull/207]**

  Previously all descriptions were set with a parameter of type
  `Option<String>`, which was not very user-friendly.

  ```rust
  let mut directive_def = DirectiveDefinition::new("provideTreat".to_string());
  directive_def.description("Ensures cats get treats.".to_string());
  ```

  [lrlna]: https://github.com/lrlna
  [pull/207]: https://github.com/apollographql/apollo-rs/pull/207

- **all default setters accept paramater of type String - [lrlna], [pull/208]**

  Similarly to above, previously all defaults were set with a parameter of type
  `Option<String>`. All defaults now accept `String`.

  [lrlna]: https://github.com/lrlna
  [pull/208]: https://github.com/apollographql/apollo-rs/pull/208

- **all default value setters are renamed to `default_value` - [lrlna], [pull/208]**

  Previously used "default" setters represented "default_value". Renaming the
  setters directly to `default_value` aligns with the spec.

  ```rust
    let ty = Type_::NamedType { name: "CatBreed".to_string() };
    let mut field = InputField::new("cat".to_string(), ty);
    field.default_value("Norwegian Forest".to_string());
  ```

  [lrlna]: https://github.com/lrlna
  [pull/208]: https://github.com/apollographql/apollo-rs/pull/208

## Features
- **ArgumentsDefinition input value setter - [lrlna], [pull/207]**

  Individual input value definitions can be set with `input_value` setter:

  ```rust
  let input_value = InputValueDefinition::new(
      String::from("first"),
      Type_::NamedType {
          name: String::from("Int"),
      },
  );
  let args_definition = ArgumentsDefinition::new();
  args_definition.input_value(input_value);
  ```

  [lrlna]: https://github.com/lrlna
  [pull/207]: https://github.com/apollographql/apollo-rs/pull/207

## Fixes

- **Use a more readable serialisation for input value definitions - [lrlna], [pull/207]**

  If any of the input value definitions in a given field definition comes with a
  description, we will multiline all input value definitions. That is to say,
  instead of serializing arguments definition like this:

  ```graphql
  type Foo {
    "This is a description of the \`one\` field."
    one("This is a description of the \`argument\` argument." argument: InputType!): Type
  }
  ```

  we serialize it as:

  ```graphql
  type Foo {
    "This is a description of the \`one\` field."
    one(
      "This is a description of the \`argument\` argument."
      argument: InputType!
    ): Type
  }
  ```

  This makes it a lot more readable, especially for users with a large number of
  input value definitions with descriptions.

  [lrlna]: https://github.com/lrlna
  [pull/207]: https://github.com/apollographql/apollo-rs/pull/207

# [0.2.3](https://crates.io/crates/apollo-encoder/0.2.3) - 2022-04-01

> Important: 1 breaking change below, indicated by **BREAKING**

## BREAKING

- **GraphQL Int Value is an i32 - [bnjjj], [pull/197]**
  We previously represented Int Values as i64, which is not compliant with the spec. This is now rectified.

  [bnjjj]: https://github.com/bnjjj
  [pull/197]: https://github.com/apollographql/apollo-rs/pull/197

## Features

- **Support 'alias' on fields - [bnjjj], [pull/191]**

  ```rust
  // results in "smallImage: displayImage" encoding
  let mut aliased_field = Field::new(String::from("displayImage"));
  aliased_field.alias(Some(String::from("smallImage")));
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/191]: https://github.com/apollographql/apollo-rs/pull/191


# [0.2.2](https://crates.io/crates/apollo-encoder/0.2.2) - 2022-02-28

> Important: 2 breaking changes below, indicated by **BREAKING**
## BREAKING
- **Rename `InputValueDef` into `InputValueDefinition` for consistency - [bnjjj], [pull/182]**

  [bnjjj]: https://github.com/bnjjj
  [pull/182]: https://github.com/apollographql/apollo-rs/pull/182

- **Rename `input_object_` method into `input_object` on `Document` - [bnjjj], [pull/182]**

  [bnjjj]: https://github.com/bnjjj
  [pull/182]: https://github.com/apollographql/apollo-rs/pull/182

## Fixes
- **Remove leading and ending `"` in `BlockStringCharacter` encoding only when it starts and end with a `"` - [bnjjj], [pull/182]**
  This ensures that a StringValue of type BlockStringCharacter, like the one in
  the test example below, does not getting encoded with an additional `"` ending
  up with `""""` leading set of characters.

  ```rust
  let desc = StringValue::Reason {
            source: Some("One of my cat is called:\r \"Mozart\"".to_string()),
        };

        assert_eq!(
            desc.to_string(),
            String::from("\n  \"\"\"\n  One of my cat is called:\r \"Mozart\"\n  \"\"\"\n  ")
        );
    );
  ```

  [bnjjj]: https://github.com/bnjjj
  [pull/168]: https://github.com/apollographql/apollo-rs/pull/182

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
  ```

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
