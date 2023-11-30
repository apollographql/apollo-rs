# Changelog

All notable changes to `apollo-compiler` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2023-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance
## Documentation-->

# [1.0.0-beta.9](https://crates.io/crates/apollo-compiler/1.0.0-beta.9) - 2023-11-30

## Features

- **Add validation convenience APIs - [SimonSapin], [pull/764]:**
 * `ast::Document::to_schema_validate`
 * `ast::Document::to_executable_validate`
 * `DiagnosticList::new`
 * `DiagnosticList::merge`

[SimonSapin]: https://github.com/SimonSapin
[pull/764]: https://github.com/apollographql/apollo-rs/pull/764


# [1.0.0-beta.8](https://crates.io/crates/apollo-compiler/1.0.0-beta.8) - 2023-11-30

## BREAKING

- **API refactor to make it harder to ignore errors - [SimonSapin], 
  [pull/752] fixing [issue/709]:**
  - `ast::Document`, `Schema`, and `ExecutableDocument` not longer contain potential errors
    that users need to check separately.
  - Instead, various constructors and methods now return a `Result`,
    with the `Err` case containing both both errors and a maybe-incomplete value.
  - Change `validate` methods of `Schema` and `ExecutableDocument` to take ownership of `self`.
    On success they return the schema or document (unmodified) wrapped in a `Valid<_>` marker type,
    which is **immutable**.
  - Change `ExecutableDocument` to require a `&Valid<Schema>` instead of `&Schema`,
    forcing callers to either run validation or opt out explicitly with `Valid::assume_valid`.
  - Make `parse_mixed` and `to_mixed` validate both the schema and document.
    Rename them with a `_validate` suffix.
  - Corresponding changes to all of the above in `Parser` method signatures
  - Remove `ast::Document::check_parse_errors`:
    parse errors are now encoded in the return value of `parse`.
  - Remove `ast::Document::to_schema_builder`. Use `SchemaBuilder::add_ast` instead.
  - Move items from the crate top-level to `apollo_compiler::validation`:
    * `Diagnostic`
    * `DiagnosticList`
    * `FileId`
    * `NodeLocation`
  - Move items from the crate top-level to `apollo_compiler::execution`:
    * `GraphQLError`
    * `GraphQLLocation`
  - Remove warning-level and advice-level diagnostics. See [issue/751].

  Highlight of signature changes:

  ```diff
  +struct Valid<T>(T); // Implements `Deref` and `AsRef` but not `DerefMut` or `AsMut`
  +
  +struct WithErrors<T> {
  +    partial: T, // Errors may cause components to be missing
  +    errors: DiagnosticList,
  +}

  -pub fn parse_mixed(…) -> (Schema, ExecutableDocument)
  +pub fn parse_mixed_validate(…)
  +    -> Result<(Valid<Schema>, Valid<ExecutableDocument>), DiagnosticList>

   impl ast::Document {
  -    pub fn parse(…) -> Self
  +    pub fn parse(…) -> Result<Self, WithErrors<Self>>

  -    pub fn to_schema(&self) -> Schema
  +    pub fn to_schema(&self) -> Result<Schema, WithErrors<Schema>>

  -    pub fn to_executable(&self) -> ExecutableDocument
  +    pub fn to_executable(&self) -> Result<ExecutableDocument, WithErrors<ExecutableDocument>>

  -    pub fn to_mixed(&self) -> (Schema, ExecutableDocument)
  +    pub fn to_mixed_validate(
  +        &self,
  +    ) -> Result<(Valid<Schema>, Valid<ExecutableDocument>), DiagnosticList>
   }

   impl Schema {
  -    pub fn parse(…) -> Self
  -    pub fn validate(&self) -> Result<DiagnosticList, DiagnosticList>

  +    pub fn parse_and_validate(…) -> Result<Valid<Self>, WithErrors<Self>>
  +    pub fn parse(…) -> Result<Self, WithErrors<Self>>
  +    pub fn validate(self) -> Result<Valid<Self>, WithErrors<Self>>
   }

   impl SchemaBuilder {
  -    pub fn build(self) -> Schema
  +    pub fn build(self) -> Result<Schema, WithErrors<Schema>>
   }

   impl ExecutableDocument {
  -    pub fn parse(schema: &Schema, …) -> Self
  -    pub fn validate(&self, schema: &Schema) -> Result<(), DiagnosticList>

  +    pub fn parse_and_validate(schema: &Valid<Schema>, …) -> Result<Valid<Self>, WithErrors<Self>>
  +    pub fn parse(schema: &Valid<Schema>, …) -> Result<Self, WithErrors<Self>>
  +    pub fn validate(self, schema: &Valid<Schema>) -> Result<Valid<Self>, WithErrors<Self>>
   }
  ```

## Features

- **Add `parse_and_validate` constructors for `Schema` and `ExecutableDocument` - [SimonSapin],
  [pull/752]:**
  when mutating isn’t needed after parsing,
  this returns an immutable `Valid<_>` value in one step.

- **Implement serde `Serialize` and `Deserialize` for some AST types - [SimonSapin], [pull/760]:**
  * `Node`
  * `NodeStr`
  * `Name`
  * `IntValue`
  * `FloatValue`
  * `Value`
  * `Type`
  Source locations are not preserved through serialization.

- **Add `ast::Definition::as_*() -> Option<&_>` methods for each variant - [SimonSapin], [pull/760]**

- **Serialize (to GraphQL) multi-line strings as block strings - [SimonSapin], [pull/724]:**
  Example before:
  ```graphql
  "Example\n\nDescription description description"
  schema { query: MyQuery }
  ```
  After:
  ```graphql
  """
  Example
  
  Description description description
  """
  schema { query: MyQuery }
  ```

## Fixes

- **Limit recursion in validation - [goto-bus-stop], [pull/748] fixing [issue/742]:**
  Validation now bails out of very long chains of definitions that refer to each other,
  even if they don't strictly form a cycle. These could previously cause extremely long validation
  times or stack overflows.

  The limit for input objects and directives is set at 32. For fragments, the limit is set at 100.
  Based on our datasets, real-world documents don't come anywhere close to this.

[SimonSapin]: https://github.com/SimonSapin
[goto-bus-stop]: https://github.com/goto-bus-stop
[issue/709]: https://github.com/apollographql/apollo-rs/issues/709
[issue/742]: https://github.com/apollographql/apollo-rs/issues/742
[issue/751]: https://github.com/apollographql/apollo-rs/issues/751
[pull/724]: https://github.com/apollographql/apollo-rs/pull/724
[pull/748]: https://github.com/apollographql/apollo-rs/pull/748
[pull/752]: https://github.com/apollographql/apollo-rs/pull/752
[pull/760]: https://github.com/apollographql/apollo-rs/pull/760

# [1.0.0-beta.7](https://crates.io/crates/apollo-compiler/1.0.0-beta.7) - 2023-11-17

## Features

- **Helper features for `Name` and `Type` - [SimonSapin], [pull/739]:**
  * The `name!` macro also accepts an identifier: 
    `name!(Query)` and `name!("Query")` create equivalent `Name` values.
  * `InvalidNameError` now contain a public `NodeStr` for the input string that is invalid,
    and implements `Display`, `Debug`, and `Error` traits.
  * Add `TryFrom` conversion to `Name` from `NodeStr`, `&NodeStr`, `&str`, `String`, and `&String`.
  * Add a `ty!` macro to build a static `ast::Type` using GraphQL-like syntax.

[SimonSapin]: https://github.com/SimonSapin
[pull/739]: https://github.com/apollographql/apollo-rs/pull/739

- **Add parsing an `ast::Type` from a string - [lrlna] and [goto-bus-stop], [pull/718] fixing [issue/715]**

  Parses GraphQL type syntax:
  ```rust
  use apollo_compiler::ast::Type;
  let ty = Type::parse("[ListItem!]!")?;
  ```

[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/718]: https://github.com/apollographql/apollo-rs/pull/718
[issue/715]: https://github.com/apollographql/apollo-rs/issues/715

## Fixes

- **Fix list and null type validation bugs - [goto-bus-stop], [pull/746] fixing [issue/738]**
  Previous versions of apollo-compiler accepted `null` inside a list even if the list item type
  was marked as required. Lists were also accepted as inputs to non-list fields. This is now
  fixed.

  ```graphql
  input Args {
    string: String
    ints: [Int!]
  }
  type Query { example(args: Args): Int }
  query {
    example(args: {
      # Used to be accepted, now raises an error
      string: ["1"]
      # Used to be accepted, now raises an error
      ints: [1, 2, null, 4]
    })
  }
  ```

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/746]: https://github.com/apollographql/apollo-rs/pull/746
[issue/738]: https://github.com/apollographql/apollo-rs/issues/738

# [1.0.0-beta.6](https://crates.io/crates/apollo-compiler/1.0.0-beta.6) - 2023-11-10

## BREAKING

- **Make everything know their own name - [SimonSapin], [pull/727] fixing [issue/708].**

  In a few places (but not consistently) a `name` field
  was omitted from some structs used as map values 
  on the basis that it would have been redundant with the map key.
  This reverts that decision,
  making it the user’s responsibility when mutating documents to keep names consistent.

  * Add a `pub name: Name` field to `executable::Fragment` as well as 
    `ScalarType`, `ObjectType`, `InterfaceType`, `EnumType`, `UnionType`, and `InputObjectType`
    in `schema`.
  * Add a `fn name(&self) -> &Name` method to the `schema::ExtendedType` enum
  * Add a `pub name: Option<Name>` field to `executable::Operation`
  * Remove `executable::OperationRef<'_>` 
    (which was equivalent to `(Option<&Name>, &Node<Operation>)`),
    replacing its uses with `&Node<Operation>`
- **Rename `Directives` and `Diagnostics` to `DirectiveList` and `DiagnosticList` - 
  [SimonSapin], [pull/732] fixing [issue/711].**
  The previous names were too similar to `Directive` and `Diagnostic` (singular).
- **Rename `ComponentStr` to `ComponentName` - [SimonSapin], [pull/713]**
  and its `node: NodeStr` field to `name: Name`.
- **Assorted changed to GraphQL names - [SimonSapin], [pull/713] fixing [issue/710].**
  - **Check validity of `ast::Name`.**
    `NodeStr` is a smart string type with infallible conversion from `&str`.
    `ast::Name` used to be a type alias for `NodeStr`, 
    leaving the possibility of creating one invalid in GraphQL syntax.
    Validation and serialization would not check this.
    `Name` is now a wrapper type for `NodeStr`.
    Its `new` constructor checks validity of the given string and returns a `Result`.
    A new `name!` macro (see below) creates a `Name` with compile-time checking.
  - **`OperationType::default_type_name` returns a `Name` instead of `&str`**
  - **`Type::new_named("x")` is removed. Use `Type::Named(name!("x"))` instead.**
  - **`ComponentStr` is renamed to `ComponentName`.**
    It no longer has infallible conversions from `&str` or `String`.
    Its `node` field is renamed to `name`;
    the type of that field is changed from `NodeStr` to `Name`.
  - **`NodeStr` no longer has a `to_component` method, only `Name` does**
  - **Various function or method parameters changed from `impl Into<Name>` to `Name`,**
    since there is no longer an infallible conversion from `&str`

## Features

- **Add serialization support for everything - [SimonSapin], [pull/728].**

  `Schema`, `ExecutableDocument`, and all AST types
  already supported serialization to GraphQL syntax
  through the `Display` trait and the `.serialize()` method.
  This is now also the case of all other Rust types
  representing some element of a GraphQL document:
  * `schema::Directives`
  * `schema::ExtendedType`
  * `schema::ScalarType`
  * `schema::ObjectType`
  * `schema::InterfaceType`
  * `schema::EnumType`
  * `schema::UnionType`
  * `schema::InputObjectType`
  * `executable::Operation`
  * `executable::Fragment`
  * `executable::SelectionSet`
  * `executable::Selection`
  * `executable::Field`
  * `executable::InlineFragment`
  * `executable::FragmentSpread`
  * `executable::FieldSet`
- **Assorted changed to GraphQL names - [SimonSapin], [pull/713] fixing [issue/710].**
  See also the BREAKING section above.
  - **Add a `name!("example")` macro**,
    to be imported with `use apollo_compiler::name;`.
    It creates an `ast::Name` from a string literal, with a compile-time validity checking.
    A `Name` created this way does not own allocated heap memory or a reference counter,
    so cloning it is extremely cheap.
  - **Add allocation-free `NodeStr::from_static`.**
    This mostly exists to support the `name!` macro, but can also be used on its own:
    ```rust
    let s = apollo_compiler::NodeStr::from_static(&"example");
    assert_eq!(s, "example");
    ```

## Fixes
- **Fix crash in validation of self-referential fragments - [goto-bus-stop], [pull/733] fixing [issue/716].**
  Now fragments that cyclically reference themselves inside a nested field also produce a
  validation error, instead of causing a stack overflow crash.

[SimonSapin]: https://github.com/SimonSapin
[goto-bus-stop]: https://github.com/goto-bus-stop
[issue/708]: https://github.com/apollographql/apollo-rs/issues/708
[issue/710]: https://github.com/apollographql/apollo-rs/issues/710
[issue/711]: https://github.com/apollographql/apollo-rs/issues/711
[issue/716]: https://github.com/apollographql/apollo-rs/issues/716
[pull/713]: https://github.com/apollographql/apollo-rs/pull/713
[pull/727]: https://github.com/apollographql/apollo-rs/pull/727
[pull/728]: https://github.com/apollographql/apollo-rs/pull/728
[pull/732]: https://github.com/apollographql/apollo-rs/pull/732
[pull/733]: https://github.com/apollographql/apollo-rs/pull/733


# [1.0.0-beta.5](https://crates.io/crates/apollo-compiler/1.0.0-beta.5) - 2023-11-08

## Features
- Diangostic struct is now public by [SimonSapin] in [11fe454]
- Improve lowercase enum value diagnostic by [goto-bus-stop] in [pull/725]

## Maintenance 
- Simplify `SchemaBuilder` internals by [SimonSapin] in [pull/722]
- Remove validation dead code by [SimonSapin] in [bd5d107]
- Skip schema AST conversion in ExecutableDocument::validate by [SimonSapin] in [pull/726]

[SimonSapin]: https://github.com/SimonSapin
[goto-bus-stop]: https://github.com/goto-bus-stop
[11fe454]: https://github.com/apollographql/apollo-rs/commit/11fe454f81b4cfbada4884a22575fa5c812a6ed4
[bd5d107]: https://github.com/apollographql/apollo-rs/commit/bd5d107eca14a7fc06dd885b2952346326e648cb
[pull/722]: https://github.com/apollographql/apollo-rs/pull/722
[pull/725]: https://github.com/apollographql/apollo-rs/pull/725
[pull/726]: https://github.com/apollographql/apollo-rs/pull/726


# [1.0.0-beta.4](https://crates.io/crates/apollo-compiler/1.0.0-beta.4) - 2023-10-16

## Features
- **JSON Serialisable compiler diagnostics - [lrlna] and [goto-bus-stop], [pull/698]:**
  This change brings back [JSON error format] for diagnostics introduced by
  [goto-bus-stop] in [pull/668] for compiler@0.11.3. As a result, diagnostics'
  line/column numbers are now also accessible as part of the public API.

  ```rust
  let json = expect_test::expect![[r#"
    {
      "message": "an executable document must not contain an object type definition",
      "locations": [
        {
          "line": 2,
          "column": 1
        }
      ]
    }"#]];
  let diagnostics = executable.validate(&schema).unwrap_err();
  diagnostics.iter().for_each(|diag| {
      assert_eq!(
          diag.get_line_column(),
          Some(GraphQLLocation { line: 2, column: 1 })
      );
      json.assert_eq(&serde_json::to_string_pretty(&diag.to_json()).unwrap());
  });
  ```
## Fixes

- **Don’t emit a validation error for relying on argument default - [SimonSapin], [pull/700]**
  A field argument or directive argument was incorrectly considered required
  as soon as it had a non-null type, even if it had a default value.

[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[SimonSapin]: https://github.com/SimonSapin
[pull/698]: https://github.com/apollographql/apollo-rs/pull/698
[pull/668]: https://github.com/apollographql/apollo-rs/pull/668
[pull/700]: https://github.com/apollographql/apollo-rs/pull/700
[JSON error format]: https://spec.graphql.org/draft/#sec-Errors.Error-Result-Format

# [1.0.0-beta.3](https://crates.io/crates/apollo-compiler/1.0.0-beta.3) - 2023-10-13

## BREAKING

- **Keep source files in `Arc<Map<…>>` everywhere - [SimonSapin], [pull/696]**
  Change struct fields from `sources: IndexMap<FileId, Arc<SourceFile>>` (in `Schema`)
  or `source: Option<(FileId, Arc<SourceFile>)>` (in `Document`, `ExecutablDocument`, `FieldSet`)
  to `sources: SourceMap`, with:
  ```rust
  pub type SourceMap = Arc<IndexMap<FileId, Arc<SourceFile>>>;
  ```
  Cases other than `Schema` still only have zero or one source when created by apollo-compiler,
  but it is now possible to make more sources available to diagnostics,
  for example when merging documents:
  ```rust
  Arc::make_mut(&mut doc1.sources).extend(doc2.sources.iter().map(|(k, v)| (*k, v.clone())));
  ```

## Features

- **Add iteration over individual diagnostics - [SimonSapin], [pull/696]:**
  ```rust
  let schema = Schema::parse(input, "schema.graphql");
  if let Err(errors) = schema.validate() {
      for error in errors.iter() {
          eprintln!("{error}")
      }
  }
  ```

## Fixes

- **Don’t panic in validation or omit diagnostics when a source location is missing - [SimonSapin], [pull/697]**
  In apollo-compiler 0.11 every element of the HIR always had a source location because
  it always came from a parsed input file.
  In 1.0 source location is always optional.
  When a node relevant to some diagnostic does not have a source location,
  the diagnostic should still be emitted but its labels (each printing a bit of source code)
  may be missing.
  Essential information should therefore be in the main message, not only in labels.

[SimonSapin]: https://github.com/SimonSapin
[pull/696]: https://github.com/apollographql/apollo-rs/pull/696
[pull/697]: https://github.com/apollographql/apollo-rs/pull/697

# [1.0.0-beta.2](https://crates.io/crates/apollo-compiler/1.0.0-beta.2) - 2023-10-10

## BREAKING

**Assorted `Schema` API changes - [SimonSapin], [pull/678]**
- Type of the `schema_definition` field changed
  from `Option<SchemaDefinition>` to `SchemaDefinition`.
  Default root operations based on object type names
  are now stored explicitly in `SchemaDefinition`.
  Serialization relies on a heuristic to decide on implicit schema definition.
- Removed `schema_definition_directives` method: no longer having an `Option` allows
  field `schema.schema_definition.directives` to be accessed directly
- Removed `query_root_operation`, `mutation_root_operation`, and `subscription_root_operation`
  methods. Instead `schema.schema_definition.query` etc can be accessed directly.

## Features

- **Add `executable::FieldSet` for a selection set with optional outer brackets - [lrlna], [pull/685] fixing [issue/681]**
  This is intended to parse string value of a [`FieldSet` custom scalar][fieldset]
  used in some Apollo Federation directives in the context of a specific schema and type.
  Its `validate` method calls a subset of validation rules relevant to selection sets.
  which is not part of a document.
  ```rust
  let input = r#"
    type Query {
      id: ID
      organization: Org
    }
    type Org {
      id: ID
    }
  "#;
  let schema = Schema::parse(input, "schema.graphql");
  schema.validate().unwrap();
  let input = "id organization { id }";
  let field_set = FieldSet::parse(&schema, "Query", input, "field_set.graphql");
  field_set.validate(&schema).unwrap();
  ```

- **Add opt-in configuration for “orphan” extensions to be “adopted” - [SimonSapin], [pull/678]**

  Type extensions and schema extensions without a corresponding definition
  are normally ignored except for recording a validation error.
  In this new mode, an implicit empty definition to extend is generated instead.
  This behavious is not the default, as it's non-standard.
  Configure a schema builder to opt in:
  ```rust
  let input = "extend type Query { x: Int }";
  let schema = apollo_compiler::Schema::builder()
      .adopt_orphan_extensions()
      .parse(input, "schema.graphql")
      .build();
  schema.validate()?;
  ```

## Fixes

- **Allow built-in directives to be redefined - [SimonSapin], [pull/684] fixing [issue/656]**
- **Allow schema extensions to extend a schema definition implied by object types named after default root operations - [SimonSapin], [pull/678] fixing [issues/682]**

[lrlna]: https://github.com/lrlna
[SimonSapin]: https://github.com/SimonSapin
[issue/656]: https://github.com/apollographql/apollo-rs/issues/656
[issue/682]: https://github.com/apollographql/apollo-rs/issues/682
[issue/681]: https://github.com/apollographql/apollo-rs/issues/681
[pull/678]: https://github.com/apollographql/apollo-rs/pull/678
[pull/684]: https://github.com/apollographql/apollo-rs/pull/684
[pull/685]: https://github.com/apollographql/apollo-rs/pull/685
[fieldset]: https://www.apollographql.com/docs/federation/subgraph-spec/#scalar-fieldset

# [1.0.0-beta.1](https://crates.io/crates/apollo-compiler/1.0.0-beta.1) - 2023-10-05

## BREAKING

Compared to 0.11, version 1.0 is a near-complete rewrite of the library
and revamp of the public API.
While in beta, there may still be breaking changes (though not as dramatic)
until 1.0.0 “final”.
If using a beta version, we recommend specifying an exact dependency in `Cargo.toml`:

```toml
apollo-compiler = "=1.0.0-beta.1"
```

## Features

The API is now centered on `Schema` and `ExecutableDocument` types.
Users no longer need to create a compiler, add inputs to it, and track them by ID.
Validation is now a method of these types, and returns a `Result` to indicate errors.

These types are serializable
(through `Display`, `.to_string()`, and a `.serialize()` config builder),
integrating the functionality of the apollo-encoder crate.

They are also mutable, and can be created programmatically out of thin air.
`Node<T>` is a thread-safe reference-counted smart pointer
that provides structural sharing and copy-on-write semantics.

# [0.11.3](https://crates.io/crates/apollo-compiler/0.11.3) - 2023-10-06

## Features
- expose line/column location and JSON format from diagnostics, by [goto-bus-stop] in [pull/668]

  You can now use `diagnostic.get_line_column()` to access the line/column number where a validation error occurred.

  `diagnostic.to_json()` returns a GraphQL error structure that's serializable with serde, matching the [JSON error format].

  ```rust
  let diagnostics = compiler.db.validate();
  let errors = diagnostics.into_iter().map(ApolloDiagnostic::to_json).collect::<Vec<_>>();

  let error_response = serde_json::to_string(&serde_json::json!({
      "errors": errors,
  }))?;
  ```

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/668]: https://github.com/apollographql/apollo-rs/pull/668
[JSON error format]: https://spec.graphql.org/draft/#sec-Errors.Error-Result-Format

- improve validation error summaries, by [goto-bus-stop] in [pull/674]

  Adds more context and a more consistent voice to the "main" message for validation errors. They are now concise,
  matter-of-fact descriptions of the problem. Information about how to solve the problem is usually already provided
  by labels and notes on the diagnostic.

  > - operation `getName` is defined multiple times
  > - interface `NamedEntity` implements itself

  The primary use case for this is to make `diagnostic.data.to_string()` return a useful message for text-only error
  reports, like in JSON responses. The JSON format for diagnostics uses these new messages.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/674]: https://github.com/apollographql/apollo-rs/pull/674

# [0.11.2](https://crates.io/crates/apollo-compiler/0.11.2) - 2023-09-11

## Features
- Add `validate_standalone_executable` function to validate an executable document without access to a schema, by [goto-bus-stop] in [pull/631], [issue/629]

  This runs just those validations that can be done on operations without knowing the types of things.
  ```rust
  let compiler = ApolloCompiler::new();
  let file_id = compiler.add_executable(r#"
  {
    user { ...userData }
  }
  "#, "query.graphql");
  let diagnostics = compiler.db.validate_standalone_executable(file_id);
  // Complains about `userData` fragment not existing, but does not complain about `user` being an unknown query.
  ```

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/631]: https://github.com/apollographql/apollo-rs/pull/631
[issue/629]: https://github.com/apollographql/apollo-rs/issues/629

## Fixes
- validate input value types, by [goto-bus-stop] in [pull/642]

  This fixes an oversight in the validation rules implemented by `compiler.db.validate()`. Previously, incorrect
  types on input values and arguments were not reported:
  ```graphql
  type ObjectType {
    id: ID!
  }
  input InputObject {
    # accepted in <= 0.11.1, reports "TypeThatDoesNotExist is not in scope" in 0.11.2
    property: TypeThatDoesNotExist
    # accepted in <= 0.11.1, reports "ObjectType is not an input type" in 0.11.2
    inputType: ObjectType
  }
  ```

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/642]: https://github.com/apollographql/apollo-rs/pull/642

# [0.12.0] (unreleased) - 2023-mm-dd

## BREAKING

- (TODO: write this)

# [0.11.1](https://crates.io/crates/apollo-compiler/0.11.1) - 2023-08-24

## Features
- disable colours in diagnostics output if the terminal is not interactive, by [EverlastingBugstopper] in [pull/628], [issue/499]

[EverlastingBugstopper]: https://github.com/EverlastingBugstopper
[pull/628]: https://github.com/apollographql/apollo-rs/pull/628
[issue/499]: https://github.com/apollographql/apollo-rs/issues/499

# [0.11.0](https://crates.io/crates/apollo-compiler/0.11.0) - 2023-08-18

## Features
- add `InterfaceTypeDefinition::implementors(&db)` to list object types and other interfaces that implement an interface, by [Geal] in [pull/616]

[Geal]: https://github.com/Geal
[pull/616]: https://github.com/apollographql/apollo-rs/pull/616

## Fixes
- fix `SelectionSet::is_introspection` when the same fragment is spread multiple times, by [glasser] in [pull/614], [issue/613]

[glasser]: https://github.com/glasser
[issue/613]: https://github.com/apollographql/apollo-rs/issues/613
[pull/614]: https://github.com/apollographql/apollo-rs/pull/614

## Maintenance
- update `apollo-parser` to 0.6.0, by [goto-bus-stop] in [pull/621]

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/621]: https://github.com/apollographql/apollo-rs/pull/621

# [0.10.0](https://crates.io/crates/apollo-compiler/0.10.0) - 2023-06-20

## BREAKING
- `SelectionSet::merge` is renamed to `SelectionSet::concat` to clarify that it doesn't do field merging, by [goto-bus-stop] in [pull/570]
- `hir::InlineFragment::type_condition` now only returns `Some()` if a type condition was explicitly specified, by [goto-bus-stop] in [pull/586]

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/570]: https://github.com/apollographql/apollo-rs/pull/570
[pull/586]: https://github.com/apollographql/apollo-rs/pull/586

## Features
- add `root_operation_name(OperationType)` helper method on `hir::SchemaDefinition` by [SimonSapin] in [pull/579]
- add an `UndefinedDirective` diagnostic type, by [goto-bus-stop] in [pull/587]

  This is used for directives instead of `UndefinedDefinition`.

[goto-bus-stop]: https://github.com/goto-bus-stop
[SimonSapin]: https://github.com/SimonSapin
[pull/579]: https://github.com/apollographql/apollo-rs/pull/579
[pull/587]: https://github.com/apollographql/apollo-rs/pull/587

## Fixes
- accept objects as values for custom scalars, by [goto-bus-stop] in [pull/585]

  The GraphQL spec is not entirely clear on this, but this is used in the real world with things
  like the `_Any` type in Apollo Federation.

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/585]: https://github.com/apollographql/apollo-rs/pull/585

## Maintenance
- update dependencies, by [goto-bus-stop] in [commit/daf918b]
- add a test for validation with `set_type_system_hir()`, by [goto-bus-stop] in [pull/583]

[goto-bus-stop]: https://github.com/goto-bus-stop
[commit/daf918b]: https://github.com/apollographql/apollo-rs/commit/daf918b62a19242bf1b8863dd598ac2912a7074e
[pull/583]: https://github.com/apollographql/apollo-rs/pull/583

# [0.9.4](https://crates.io/crates/apollo-compiler/0.9.4) - 2023-06-05

## Features
- accept any primitive value type for custom scalar validation, by [lrlna] in [pull/575]

  If you provide a value to a custom scalar in your GraphQL source text, apollo-compiler
  now accepts any value type. Previously it was not possible to write values for custom
  scalars into a query or schema because the value you wrote would never match the custom
  scalar type.

  This now works:
  ```graphql
  scalar UserID @specifiedBy(url: "https://my-app.net/api-docs/users#id")
  type Query {
    username (id: UserID): String
  }
  ```
  ```graphql
  {
    username(id: 575)
  }
  ```

- add type name to the `UndefinedField` diagnostic data, by [goto-bus-stop] in [pull/577]

  When querying a field that does not exist, the type name that's being queried is stored on
  the diagnostic, so you can use it when handling the error.

[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/575]: https://github.com/apollographql/apollo-rs/pull/575
[pull/577]: https://github.com/apollographql/apollo-rs/pull/577

# [0.9.3](https://crates.io/crates/apollo-compiler/0.9.3) - 2023-05-26

## Fixes
- fix nullable / non-nullable validations inside lists, by [lrlna] in [pull/567]

  Providing a variable of type `[Int!]!` to an argument of type `[Int]` is now allowed.

[lrlna]: https://github.com/lrlna
[pull/567]: https://github.com/apollographql/apollo-rs/pull/567

## Maintenance
- use official ariadne release, by [goto-bus-stop] in [pull/568]

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/568]: https://github.com/apollographql/apollo-rs/pull/568

# [0.9.2](https://crates.io/crates/apollo-compiler/0.9.2) - 2023-05-23

## Features
- add `as_$type()` methods to `hir::Value`, by [goto-bus-stop] in [pull/564]

  These methods simplify casting the `hir::Value` enum to single Rust types.
  Added methods:

  - `hir::Value::as_i32() -> Option<i32>`
  - `hir::Value::as_f64() -> Option<f64>`
  - `hir::Value::as_str() -> Option<&str>`
  - `hir::Value::as_bool() -> Option<bool>`
  - `hir::Value::as_list() -> Option<&Vec<Value>>`
  - `hir::Value::as_object() -> Option<&Vec<(Name, Value)>>`
  - `hir::Value::as_variable() -> Option<&Variable>`

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/564]: https://github.com/apollographql/apollo-rs/pull/564

## Fixes
-  non-nullable variables should be accepted for nullable args, by [lrlna] in [pull/565]

   Fixes several `null`-related issues from 0.9.0.

-  add an `UndefinedVariable` diagnostic, by [goto-bus-stop] in [pull/563]

   Previously undefined variables were reported with an `UndefinedDefinition` diagnostic.
   Splitting it up lets us provide a better error message for missing variables.

[goto-bus-stop]: https://github.com/goto-bus-stop
[lrlna]: https://github.com/lrlna
[pull/563]: https://github.com/apollographql/apollo-rs/pull/563
[pull/565]: https://github.com/apollographql/apollo-rs/pull/565

# [0.9.1](https://crates.io/crates/apollo-compiler/0.9.1) - 2023-05-19

## Fixes
- Update the apollo-parser dependency version, by [goto-bus-stop] in [pull/559]

[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/559]: https://github.com/apollographql/apollo-rs/pull/559

# [0.9.0](https://crates.io/crates/apollo-compiler/0.9.0) - 2023-05-12

This release completes GraphQL validation specification, making the compiler spec-compliant.

You can validate the entire corpus of the compiler, or run individual compiler validation rules. The example below runs the whole corpus, as well specifically runs `compiler.db.validate_executable(file_id)` for each of the two defined operations.

```rust
let schema = r#"
type Query {
  cat: Cat
}

type Cat{
  name: String!
  nickname: String
  purrVolume: Int
  doesKnowCommand(catCommand: CatCommand!): Boolean!
}

enum CatCommand {
  HOP
}
    "#;

let cat_name_op = r#"
query getCatName {
  cat {
    name
  }
}
    "#;

let cat_command_op = r#"
query getCatName {
  cat {
    doesNotKnowCommand
  }
}
    "#;
let mut compiler = ApolloCompiler::new();
compiler.add_type_system(schema, "schema.graphl");
let cat_name_file = compiler.add_executable(cat_name_op, "cat_name_op.graphql");
let cat_command_file = compiler.add_executable(cat_command_op, "cat_command_op.graphql");

// validate both the operation and the type system
let all_diagnostics = compiler.validate();
assert_eq!(all_diagnostics.len(), 1);

// validate just the executables individual
let cat_name_op_diagnotics = compiler.db.validate_executable(cat_name_file);
assert!(cat_name_op_diagnotics.is_empty());

let cat_command_op_diagnotics = compiler.db.validate_executable(cat_command_file);
// This one has an error, where a field queries is not defined.
assert_eq!(cat_command_op_diagnotics.len(), 1);
for diag in cat_command_op_diagnotics {
    println!("{}", diag);
}
```

## BREAKING
- remove `impl Default` for ApolloCompiler, by [dariuszkuc] in [pull/542]
- align HIR extension getters to those of their type definition, by [lrlna] in [pull/540]

  The following methods were changed:
    - `InputObjectTypeExtension.fields_definition()` -> `InputObjectTypeDefinition.fields()`
    - `ObjectTypeExtension.fields_definition()` -> `ObjectTypeExtension.fields()`
    - `InterfaceTypeExtension.fields_definition()` -> `InterfaceTypeExtension.fields()`
    - `EnumTypeExtension.enum_values_definition()` -> `EnumTypeExtension.values()`
    - `UnionTypeExtension.union_members()` -> `UnionTypeExtension.members()`

## Features
- validate values are of correct type, by [lrlna]  in [pull/550]
- support the built-in `@deprecated` directive on arguments and input values, by [goto-bus-stop] in [pull/518]
- validate that variable usage is allowed, by [lrlna] in [pull/537]
- validate executable documents do not contain type definitions, by [goto-bus-stop] in [pull/535]
- validate union extensions, by [goto-bus-stop] in [pull/534]
- validate input object extensions, by [goto-bus-stop] in [pull/533]
- validate interface extensions, by [goto-bus-stop] in [pull/532]
- validate enum extensions, by [goto-bus-stop] in [pull/528]
- validate object type extensions, by [goto-bus-stop] in [pull/524]
- validate fragment spread is possible, by [goto-bus-stop] in [pull/511]

## Fixes
- fix recursion cycle in `is_introspection` HIR getter, by [allancalix] and [goto-bus-stop] in [pull/544] and [pull/552]

[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[allancalix]: https://github.com/allancalix
[pull/511]: https://github.com/apollographql/apollo-rs/pull/511
[pull/524]: https://github.com/apollographql/apollo-rs/pull/524
[pull/518]: https://github.com/apollographql/apollo-rs/pull/518
[pull/528]: https://github.com/apollographql/apollo-rs/pull/528
[pull/532]: https://github.com/apollographql/apollo-rs/pull/532
[pull/533]: https://github.com/apollographql/apollo-rs/pull/533
[pull/534]: https://github.com/apollographql/apollo-rs/pull/534
[pull/535]: https://github.com/apollographql/apollo-rs/pull/535
[pull/537]: https://github.com/apollographql/apollo-rs/pull/537
[pull/540]: https://github.com/apollographql/apollo-rs/pull/540
[pull/542]: https://github.com/apollographql/apollo-rs/pull/542
[pull/544]: https://github.com/apollographql/apollo-rs/pull/544
[pull/550]: https://github.com/apollographql/apollo-rs/pull/550
[pull/552]: https://github.com/apollographql/apollo-rs/pull/552

# [0.8.0](https://crates.io/crates/apollo-compiler/0.8.0) - 2023-04-13
## BREAKING
There is now an API to set parser's token limits via `apollo-compiler`. To
accommodate an additional limit, we changed the API to set several limits
simultaneously.

```rust
let op = r#"
    query {
        a {
            a {
                a {
                    a
                }
            }
        }
    }
"#;
let mut compiler = ApolloCompiler::new().token_limit(22).recursion_limit(10);
compiler.add_executable(op, "op.graphql");
let errors = compiler.db.syntax_errors();

assert_eq!(errors.len(), 1)
```
by [lrlna] in [pull/512]

## Features
- validate fragment definitions are used, by [gocamille] in [pull/483]
- validate fragment type condition exists in the type system and are declared on composite types, by [gocamille] in [pull/483]
- validate fragment definitions do not contain cycles, by [goto-bus-stop] in [pull/518]

## Fixes
- fix duplicate directive location info, by [goto-bus-stop]  in [pull/516]
- use `LimitExceeded` diagnostic for limit related errors, by [lrlna] in [pull/520]

[lrlna]: https://github.com/lrlna
[gocamille]: https://github.com/gocamille
[goto-bus-stop]: https://github.com/goto-bus-stop
[pull/483]: https://github.com/apollographql/apollo-rs/pull/483
[pull/512]: https://github.com/apollographql/apollo-rs/pull/512
[pull/516]: https://github.com/apollographql/apollo-rs/pull/516
[pull/518]: https://github.com/apollographql/apollo-rs/pull/518
[pull/520]: https://github.com/apollographql/apollo-rs/pull/520

# [0.7.2](https://crates.io/crates/apollo-compiler/0.7.2) - 2023-04-03

## Features
- validate fragment spread target is defined, by [goto-bus-stop] in [pull/506]
- validate circular input objects, by [lrlna] in [pull/505]

## Fixes
- `db.interfaces()` checks pre-computed hir for interfaces first, by [lrlna] in [de4baea]

[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[de4baea]: https://github.com/apollographql/apollo-rs/commit/de4baea13745089ace3821bccc30cf1c4008ba20
[pull/505]: https://github.com/apollographql/apollo-rs/pull/505
[pull/506]: https://github.com/apollographql/apollo-rs/pull/506

# [0.7.1](https://crates.io/crates/apollo-compiler/0.7.1) - 2023-03-28

## Features
- validate indirectly self-referential directives by [goto-bus-stop] in [pull/494]

## Fixes
- include built-in enum types in `db.type_system()` query by [SimonSapin] in [pull/501]
- `field.field_definition()` works for interface types by [zackangelo] in [pull/502]
- validate used variables in lists and objects by [yanns] in [pull/497]

[SimonSapin]: https://github.com/SimonSapin
[goto-bus-stop]: https://github.com/goto-bus-stop
[yanns]: https://github.com/yanns
[zackangelo]: https://github.com/zackangelo
[pull/494]: https://github.com/apollographql/apollo-rs/pull/494
[pull/497]: https://github.com/apollographql/apollo-rs/pull/497
[pull/501]: https://github.com/apollographql/apollo-rs/pull/501
[pull/502]: https://github.com/apollographql/apollo-rs/pull/502

# [0.7.0](https://crates.io/crates/apollo-compiler/0.7.0) - 2023-03-28

> Important: X breaking changes below, indicated by **BREAKING**

This release encompasses quite a few validation rules the compiler was missing.
Here, we primarily focused on field and directive validation, as well as supporting
multi-file diagnostics.
## BREAKING
- `find_operation` query now mimics spec's
[`getOperation`](https://spec.graphql.org/October2021/#GetOperation())
functionality and returns the anonymous operation if `None` is specified for
operation name; by [lrlna] in [pull/447]
- Extensions are applied implicitly in HIR by [SimonSapin] in [pull/481],
[pull/482], and [pull/484]

Adding a GraphQL type extension is similar to modifying a type.
This release makes a number of breaking changes to API signatures and behavior
so these modifications are accounted for implicitly.
For example, `interface.field(name)` may now return a field
from an `extend interface` extension or from the original `interface` definition.
We expect that most callers don’t need to tell the difference.
For callers that do, methods with a `self_` prefix are added (or renamed)
for accessing components of a definition itself as opposed to added by an extension.

Renamed methods:

* `SchemaDefinition::root_operation_type_definition` → `self_root_operations`
* `ObjectTypeDefinition::fields_definition` → `self_fields`
* `InterfaceTypeDefinition::fields_definition` → `self_fields`
* `InputObjectTypeDefinition::input_fields_definition` → `self_fields`
* `ObjectTypeDefinition::implements_interfaces` → `self_implements_interfaces`
* `InterfaceTypeDefinition::implements_interfaces` → `self_implements_interfaces`
* `UnionTypeDefinition::union_members` → `self_members`
* `EnumTypeDefinition::enum_values_definition` → `self_values`
* `TypeDefiniton::directives` → `self_directives`
* `SchemaDefiniton::directives` → `self_directives`
* `EnumTypeDefiniton::directives` → `self_directives`
* `UnionTypeDefiniton::directives` → `self_directives`
* `ObjectTypeDefiniton::directives` → `self_directives`
* `ScalarTypeDefiniton::directives` → `self_directives`
* `InterfaceTypeDefiniton::directives` → `self_directives`
* `InputObjectTypeDefiniton::directives` → `self_directives`

Method names freed by the above are now redefined with new behaviour and
signature, and include extensions:

* `ObjectTypeDefinition::implements_interfaces() -> impl Iterator`
* `InterfaceTypeDefinition::implements_interfaces() -> impl Iterator`
* `TypeDefiniton::directives() -> impl Iterator`
* `SchemaDefiniton::directives() -> impl Iterator`
* `EnumTypeDefiniton::directives() -> impl Iterator`
* `UnionTypeDefiniton::directives() -> impl Iterator`
* `ObjectTypeDefiniton::directives() -> impl Iterator`
* `ScalarTypeDefiniton::directives() -> impl Iterator`
* `InterfaceTypeDefiniton::directives() -> impl Iterator`
* `InputObjectTypeDefiniton::directives() -> impl Iterator`

Methods whose behaviour and signature changed, where each method now returns the
name of an object type instead of its definition:

* `SchemaDefinition::query() -> Option<&str>`
* `SchemaDefinition::mutation() -> Option<&str>`
* `SchemaDefinition::subscription() -> Option<&str>`

Methods whose behaviour changed to consider extensions, and no signature has changed

* `TypeDefinition::field(name) -> Option`
* `ObjectTypeDefinition::field(name) -> Option`
* `InterfaceTypeDefinition::field(name) -> Option`

New methods which take extensions into consideration:

* `SchemaDefinition::root_operations() -> impl Iterator`
* `ObjectTypeDefinition::fields() -> impl Iterator`
* `ObjectTypeDefinition::implements_interface(name) -> bool`
* `InterfaceTypeDefinition::fields() -> impl Iterator`
* `InterfaceTypeDefinition::implements_interface(name) -> bool`
* `InputObjectTypeDefinition::self_fields() -> &[_]`
* `InputObjectTypeDefinition::fields() -> impl Iterator`
* `InputObjectTypeDefinition::field(name) -> Option`
* `UnionTypeDefinition::members() -> impl Iterator`
* `UnionTypeDefinition::has_member(name) -> bool`
* `EnumTypeDefinition::values() -> impl Iterator`
* `EnumTypeDefinition::value(name) -> Option`

New methods for every type which have a `directives` method:

* `directive_by_name(name) -> Option`
* `directives_by_name(name) -> impl Iterator`

## Features
- support mutli-file diagnostics by [goto-bus-stop] in [pull/414]
- validate directive locations by [lrlna] in [pull/417]
- validate undefined directives by [lrlna] in [pull/417]
- validate non-repeatable directives in a given location by [goto-bus-stop] in [pull/488]
- validate conflicting fields in a selection set (spec: fields can merge) by [goto-bus-stop] in [pull/470]
- validate introspection fields in subscriptions by [gocamille] in [pull/438]
- validate required arguments by [goto-bus-stop] in [pull/452]
- validate unique variables by [lrlna] in [pull/455]
- validate variables are of input type by [lrlna] in [pull/455]
- validate root operation type is of Object Type by [lrlna] in [pull/419]
- validate nested fields in selection sets by [erikwrede] in [pull/441]
- validate extension existance and kind by [goto-bus-stop] in [pull/458]
- validate leaf field selection by [yanns] in [pull/465]
- introduce `op.is_introspection` helper method by [jregistr] in [pull/421]
- built-in graphql types, including introspection types, are now part of the
compiler context by [lrlna] in [pull/489]

## Fixes
- fix variables in directive arguments being reported as unused [goto-bus-stop] in [pull/487]
- `op.operation_ty()` does not deref [lrlna] in [pull/434]

## Maintenance
- tests for operation field type resolution v.s. type extensions by [SimonSapin] in [pull/492]
- using [salsa::invoke] macro to annotate trait function location by [lrlna] in [pull/491]
- use `Display` for `hir::OperationType` and `hir::DirectiveLocation` by [goto-bus-stop] in [pull/435]
- rework and simplify validation database by [lrlna] in [pull/436]
- reset `FileId` between directory tests by [goto-bus-stop] in [pull/437]
- remove unncessary into_iter() calls by [goto-bus-stop] in [pull/472]
- check test numbers are unique in test output files by [goto-bus-stop] in [pull/471]

[SimonSapin]: https://github.com/SimonSapin
[lrlna]: https://github.com/lrlna
[goto-bus-stop]: https://github.com/goto-bus-stop
[jregistr]: https://github.com/jregistr
[yanns]: https://github.com/yanns
[gocamille]: https://github.com/gocamille
[erikwrede]: https://github.com/erikwrede
[pull/414]: https://github.com/apollographql/apollo-rs/pull/414
[pull/417]: https://github.com/apollographql/apollo-rs/pull/417
[pull/419]: https://github.com/apollographql/apollo-rs/pull/419
[pull/421]: https://github.com/apollographql/apollo-rs/pull/421
[pull/434]: https://github.com/apollographql/apollo-rs/pull/434
[pull/435]: https://github.com/apollographql/apollo-rs/pull/435
[pull/436]: https://github.com/apollographql/apollo-rs/pull/436
[pull/437]: https://github.com/apollographql/apollo-rs/pull/437
[pull/441]: https://github.com/apollographql/apollo-rs/pull/441
[pull/447]: https://github.com/apollographql/apollo-rs/pull/447
[pull/452]: https://github.com/apollographql/apollo-rs/pull/452
[pull/455]: https://github.com/apollographql/apollo-rs/pull/455
[pull/458]: https://github.com/apollographql/apollo-rs/pull/458
[pull/465]: https://github.com/apollographql/apollo-rs/pull/465
[pull/470]: https://github.com/apollographql/apollo-rs/pull/470
[pull/471]: https://github.com/apollographql/apollo-rs/pull/471
[pull/472]: https://github.com/apollographql/apollo-rs/pull/472
[pull/481]: https://github.com/apollographql/apollo-rs/pull/481
[pull/482]: https://github.com/apollographql/apollo-rs/pull/482
[pull/484]: https://github.com/apollographql/apollo-rs/pull/484
[pull/487]: https://github.com/apollographql/apollo-rs/pull/487
[pull/488]: https://github.com/apollographql/apollo-rs/pull/488
[pull/489]: https://github.com/apollographql/apollo-rs/pull/489
[pull/491]: https://github.com/apollographql/apollo-rs/pull/491
[pull/492]: https://github.com/apollographql/apollo-rs/pull/492

# [0.6.0](https://crates.io/crates/apollo-compiler/0.6.0) - 2023-01-18

This release has a few breaking changes as we try to standardise APIs across the
compiler. We appreciate your patience with these changes. If you run into trouble, please [open an issue].

## BREAKING
- Rename `compiler.create_*` methods to `compiler.add_*`, [SimonSapin] in [pull/412]
- Rename `schema` to `type_system` for `compiler.add_` and `compiler.update_`
methods, [SimonSapin] in [pull/413]
- Unify `ty`, `type_def` and `kind` namings in HIR, [lrlna] in [pull/415]
  - in `Type` struct impl: `ty()` --> `type_def()`
  - in `TypeDefinition` struct impl: `ty()` --> `kind()`
  - in `FragmentDefinition` struct impl: `ty()` --> `type_def()`
  - in `RootOperationTypeDefinition` struct: `operation_type` field -->
  `operation_ty`

## Features
- `FileId`s are unique per process, [SimonSapin] in [405]
- Type alias `compiler.snapshot()` return type to `Snapshot`, [SimonSapin] in
[410]
- Introduce a type system high-level intermediate representation (HIR) as input
to the compiler, [SimonSapin] in [407]

## Fixes
- Use `#[salsa::transparent]` for `find_*` queries, i.e. not caching query results, [lrlna] in [403]

## Maintenance
- Add compiler benchmarks, [lrlna] in [404]

## Documentation
- Document `apollo-rs` runs on stable, [SimonSapin] in [402]

[open an issue]: https://github.com/apollographql/apollo-rs/issues/new/choose
[lrlna]: https://github.com/lrlna
[SimonSapin]: https://github.com/SimonSapin
[pull/402]: https://github.com/apollographql/apollo-rs/pull/402
[pull/403]: https://github.com/apollographql/apollo-rs/pull/403
[pull/404]: https://github.com/apollographql/apollo-rs/pull/404
[pull/405]: https://github.com/apollographql/apollo-rs/pull/405
[pull/407]: https://github.com/apollographql/apollo-rs/pull/407
[pull/410]: https://github.com/apollographql/apollo-rs/pull/410
[pull/412]: https://github.com/apollographql/apollo-rs/pull/412
[pull/413]: https://github.com/apollographql/apollo-rs/pull/413
[pull/415]: https://github.com/apollographql/apollo-rs/pull/415


# [0.5.0](https://crates.io/crates/apollo-compiler/0.5.0) - 2023-01-04

## Highlights
### Multi-file support
You can now build a compiler from multiple sources. This  is especially useful
when various parts of a GraphQL document are coming in at different times and
need to be analysed as a single context. Or, alternatively, you are looking to
lint or validate multiple GraphQL files part of the same context in a given directory or workspace.

The are three different kinds of sources:
- `document`: for when a source is composed of executable and type system
definitions, or you're uncertain of definitions types
- `schema`: for sources with type system definitions or extensions
- `executable`: for sources with executable definitions/GraphQL queries

You can add a source with `create_` and update it with `update_`, for example
`create_document` and `update_document`. Here is an example:

```rust
    let schema = r#"
type Query {
  dog: Dog
}

type Dog {
  name: String!
}
    "#;

    let query = r#"
query getDogName {
  dog {
    name
  }
}

# duplicate name, should show up in diagnostics
query getDogName {
  dog {
    owner {
      name
    }
  }
}
    "#;
    let updated_schema = r#"
type Query {
  dog: Dog
}

type Dog {
  name: String!
  owner: Human
}

type Human {
  name: String!
}
    "#;
    let mut compiler = ApolloCompiler::new();
    let schema_id = compiler.create_schema(schema, "schema.graphl");
    let executable_id = compiler.create_executable(query, "query.graphql");
    compiler.update_schema(updated_schema, schema_id);
```

For more elaborate examples, please refer to [`multi_source_validation`] and
[`file_watcher`] examples in the `examples` dir.

We look forward to your feedback on this feature, should you be using it.

Completed in [pull/368] in collaboration with [goto-bus-stop], [SimonSapin] and
[lrlna].

## BREAKING
- Remove UUID helpers and related UUID APIs from database by [SimonSapin] in
[pull/391]
- Merge `DocumentDatabase` trait into `HIRDatabase` by [SimonSapin] in
[pull/394]
- Replace `hir::Definition` enum with `hir::TypeSystemDefinitions` struct by
[SimonSapin] in [pull/395]
- `db.type_system_definitions` returns a `TypeSystemDefinitions` by [SimonSapin]
in [pull/395]
- Remove `db.db_definitions`, `find_definition_by_name` and
`find_type_system_definition_by_name` by [SimonSapin] in [pull/395]
- Remove queries returning type extensions, instead type definitions in the HIR
contain extension information by [SimonSapin] in [pull/387]

## Features
- `db.fragments`, `db.object_types`, `db.scalars`, `db.enums`, `db.unions`,
`db.interfaces`, `db.input_objects`, and `db.directive_definitions` return
name-indexed maps by [SimonSapin] in [pull/387]

[`file_watcher`]: https://github.com/apollographql/apollo-rs/blob/eb9687fc64dfe0bf618f2025f633e52950940b8a/crates/apollo-compiler/examples/file_watcher.rs
[`multi_source_validation`]: https://github.com/apollographql/apollo-rs/blob/8c66c2c36053ff592682504276307a3fead0b3ad/crates/apollo-compiler/examples/multi_source_validation.rs
[goto-bus-stop]: https://github.com/goto-bus-stop
[lrlna]: https://github.com/lrlna
[SimonSapin]: https://github.com/SimonSapin
[pull/368]: https://github.com/apollographql/apollo-rs/pull/368
[pull/391]: https://github.com/apollographql/apollo-rs/pull/391
[pull/394]: https://github.com/apollographql/apollo-rs/pull/394
[pull/395]: https://github.com/apollographql/apollo-rs/pull/395
[pull/387]: https://github.com/apollographql/apollo-rs/pull/387

# [0.4.1](https://crates.io/crates/apollo-compiler/0.4.1) - 2022-12-13
## Features
- **add new APIs - [SimonSapin], [pull/382]**
  - [`db.find_enum_by_name()`] to look up an `EnumTypeDefinition`.
  - [`directive.argument_by_name()`] to look up the value of an argument to a directive call.
  - [`scalar_type.is_built_in()`] to check if a `ScalarTypeDefinition` is defined by the GraphQL spec rather than the schema text.
  - [`enum_value.directives()`] to access the directives used on an enum value.
  - `hir::Float` is now `Copy` so it can be passed around more easily; use [`hir_float.get()`] to access the underlying `f64` or [`hir_float.to_i32_checked()`] to convert to an `i32`.

  [SimonSapin]: https://github.com/SimonSapin
  [pull/382]: https://github.com/apollographql/apollo-rs/pull/382
  [`db.find_enum_by_name()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/trait.DocumentDatabase.html#tymethod.find_union_by_name
  [`directive.argument_by_name()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/database/hir/struct.Directive.html#method.argument_by_name
  [`scalar_type.is_built_in()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/database/hir/struct.ScalarTypeDefinition.html#method.is_built_in
  [`enum_value.directives()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/database/hir/struct.EnumValueDefinition.html#method.directives
  [`hir_float.get()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/database/hir/struct.Float.html#method.get
  [`hir_float.to_i32_checked()`]: https://docs.rs/apollo-compiler/0.4.1/apollo_compiler/database/hir/struct.Float.html#method.to_i32_checked

## Fixes
- **do not panic when creating HIR from a parse tree with syntax errors - [goto-bus-stop], [pull/381]**

  When using the compiler, nodes with syntax errors in them are ignored. As syntax errors are returned
  from the parser, you can still tell that something is wrong. The compiler just won't crash the whole
  program anymore.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/381]: https://github.com/apollographql/apollo-rs/pull/381

# [0.4.0](https://crates.io/crates/apollo-compiler/0.4.0) - 2022-11-29
## Features
- **add parser recursion limit API - [SimonSapin], [pull/353], [issue/296]**

  Calling `ApolloCompiler::with_recursion_limit` instead of `ApolloCompiler::new`
  makes the compiler configure [the corresponding parser limit][with].
  This limit protects against stack overflow and is enabled either way.
  Configuring it may be useful for example if you’re also configuring the stack size.

  [SimonSapin]: https://github.com/SimonSapin
  [pull/353]: https://github.com/apollographql/apollo-rs/pull/353
  [issue/296]: https://github.com/apollographql/apollo-rs/issues/296
  [with]: https://docs.rs/apollo-parser/0.3.1/apollo_parser/struct.Parser.html#method.recursion_limit

- **expose the repeatable attribute on `DirectiveDefinition` - [allancalix], [pull/367]**

  There was previously no way to access the `repeatable` field on the `DirectiveDefinition` type.
  This field is required for validation rules.

  [allancalix]: https://github.com/allancalix
  [pull/367]: https://github.com/apollographql/apollo-rs/pull/367

- **add type extensions - [SimonSapin], [pull/369]**

  apollo-compiler now partially supports GraphQL `extend` types. The `is_subtype` query takes
  extensions into account.

  Some other parts of the compiler, like validation, do not yet support extensions.

  [SimonSapin]: https://github.com/SimonSapin
  [pull/369]: https://github.com/apollographql/apollo-rs/pull/369

## Fixes
- **fix `@include` allowed directive locations - [allancalix], [pull/366]**

  The locations for the `@include` directive wrongly specified `FragmentDefinition` instead of `FragmentSpread`.
  It now matches the spec.

  [allancalix]: https://github.com/allancalix
  [pull/366]: https://github.com/apollographql/apollo-rs/pull/366

## Maintenance
- **avoid double lookup in `SchemaDefinition::{query,mutation,subscription}` - [SimonSapin], [pull/364]**

  [SimonSapin]: https://github.com/SimonSapin
  [pull/364]: https://github.com/apollographql/apollo-rs/pull/364

# [0.3.0](https://crates.io/crates/apollo-compiler/0.3.0) - 2022-11-02
## Breaking
- **compiler.parse is renamed to compiler.ast - [lrlna], [pull/290]**

  `compiler.ast()` returns the `SyntaxTree` produced by the parser and is much
  clearer method than `compiler.parse()`.

  [lrlna]: https://github.com/lrlna
  [pull/290]: https://github.com/apollographql/apollo-rs/pull/290

- **selection.ty(db) now expects a `db` parameter - [lrlna], [pull/290]**

  As byproduct of separating compiler's query_groups into individual components.
  Selection's type can now be accessed like so:

    ```rust
    let ctx = ApolloCompiler::new(input);
    let top_product_fields: Vec<String> = top_products
      .iter()
      .filter_map(|field| Some(field.ty(&ctx.db)?.name()))
      .collect();
    ```

  [lrlna]: https://github.com/lrlna
  [pull/290]: https://github.com/apollographql/apollo-rs/pull/290

- **removes db.definitions() API - [lrlna], [pull/295]**

  `db.definitions()` returned a `!Send` value that is no longer possible with
  the `ParallelDatabase` implementation.

  To access HIR definitions, use `db.db_definitions()` and `db.type_system_definitions`.

  [lrlna]: https://github.com/lrlna
  [pull/295]: https://github.com/apollographql/apollo-rs/pull/295

## Features
- **add subtype_map and is_subtype queries - [lrlna]/[SimonSapin], [pull/333]**

  This allows users to check whether a particular type is a subtype of another
  type. For example, in a UnionDefinition such as `union SearchResult = Photo |
  Person`, `Person` is a suptype of `SearchResult`. In an InterfaceDefinition such
  as `type Business implements NamedEntity & ValuedEntity { # fields }`,
  `Business` is a subtype of `NamedEntity`.

  [lrlna]: https://github.com/lrlna
  [SimonSapin]: https://github.com/SimonSapin
  [pull/333]: https://github.com/apollographql/apollo-rs/pull/333

- **pub compiler storage - allow database composition - [lrlna], [pull/328]**

  This allows for internal query_groups to be exported, and allows users to
  compose various databases from compiler's existing dbs and their queries.

  This is how you'd create a database with storage from apollo-compiler:
    ```rust
    use apollo_compiler::{database::{AstStorage, DocumentStorage}};

    #[salsa::database(AstStorage, DoumentStorage)]
    pub struct AnotherDatabase {
        pub storage: salsa::Storage<AnotherDatabase>,
    }
    ```

  You can also see a more detailed linting example in [examples] dir.

  [lrlna]: https://github.com/lrlna
  [pull/328]: https://github.com/apollographql/apollo-rs/pull/328
  [examples]: ./examples/extend_db.rs

- **validate argument name uniqueness - [goto-bus-stop], [pull/317]**

  It's an error to declare or provide multiple arguments by the same name, eg:

    ```graphql
    type Query {
      things(offset: Int!, offset: Int!): [Thing]
      # ERR: duplicate argument definition: offset
    }
    ```

    ```graphql
    query GetThings {
      things(offset: 10, offset: 20) { id }
      # ERR: duplicate argument values: offset
    }
    ```

  This adds `UniqueArgument` diagnostics and checks for argument duplications in:
  field definitions, fields, directives, interfaces and directive definitions.

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/317]: https://github.com/apollographql/apollo-rs/pull/317

- **getter for directives in HIR FragmentSpread - [allancalix], [pull/315]**

  Allow accessing directives in a given FragmentSpread node in a high-level
  intermediate representation of the compiler.

  [allancalix]: https://github.com/allancalix
  [pull/315]: https://github.com/apollographql/apollo-rs/pull/315

- **create validation database - [lrlna], [pull/303]**

  All validation now happens in its own database, which can be accessed with
  `ValidationDatabase` and `ValidationStorage`.

  [lrlna]: https://github.com/lrlna
  [pull/303]: https://github.com/apollographql/apollo-rs/pull/303

- **thread-safe compiler: introduce snapshots - [lrlna], [pull/295] + [pull/332]**

  Implements `ParallelDatabase` for `RootDatabase` of the compiler. This allows
  us to create snapshots that can allow users to query the database from
  multiple threads. For example:

    ```rust
    let input = r#"
    type Query {
      website: URL,
      amount: Int
    }

    scalar URL @specifiedBy(url: "https://tools.ietf.org/html/rfc3986")
    "#;

    let ctx = ApolloCompiler::new(input);
    let diagnostics = ctx.validate();
    for diagnostic in &diagnostics {
        println!("{}", diagnostic);
    }

    assert!(diagnostics.is_empty());

    let snapshot = ctx.snapshot();
    let snapshot2 = ctx.snapshot();

    let thread1 = std::thread::spawn(move || snapshot.find_object_type_by_name("Query".into()));
    let thread2 = std::thread::spawn(move || snapshot2.scalars());

    thread1.join().expect("object_type_by_name panicked");
    thread2.join().expect("scalars failed");
    ```

  [lrlna]: https://github.com/lrlna
  [pull/295]: https://github.com/apollographql/apollo-rs/pull/295
  [pull/332]: https://github.com/apollographql/apollo-rs/pull/332

- **add description getters to compiler's HIR nodes - [aschaeffer], [pull/289]**

  Expose getters for descriptions that can be accessed for any definitions that
  support them. For example:
    ```rust
    let input = r#"
    "Books in a given libary"
    type Book {
      id: ID!
    }
    "#;

    let ctx = ApolloCompiler::new(input);

    let desc = ctx.db.find_object_type_by_name("Book".to_string()).unwrap().description();
    ```
  [aschaeffer]: https://github.com/aschaeffer
  [pull/289]: https://github.com/apollographql/apollo-rs/pull/289

## Fixes
- **update parser version - [goto-bus-stop], [pull/331]**

  [goto-bus-stop]: https://github.com/goto-bus-stop
  [pull/331]: https://github.com/apollographql/apollo-rs/pull/331

- **unused variables return an error diagnostic - [lrlna], [pull/314]**

  We were previously returning a warning for any unused variables, it is now
  reported as an error.

  [lrlna]: https://github.com/lrlna
  [pull/314]: https://github.com/apollographql/apollo-rs/pull/314
## Maintenance
- **split up db into several components - [lrlna], [pull/290]**

  We are splitting up the single `query_group` we had in our db into several
  `query_group`s that currently just build upon each other, and eventually could
  support more complex relationships between one another. The current structure:

  `Inputs` --> `DocumentParser` --> `Definitions` --> `Document`

  All of these `query_group`s make up the `RootDatabase`, i.e. `salsa::database`.

  This also allows external users to build out their own databases with
  compiler's query groups.

  [lrlna]: https://github.com/lrlna
  [pull/290]: https://github.com/apollographql/apollo-rs/pull/290

- **support wasm compiler target - [allancalix], [pull/287], [issue/288]**

  `apollo-compiler` can now compile to a Wasm target with `cargo check --target wasm32-unknown-unknown`.

  [allancalix]: https://github.com/allancalix
  [pull/287]: https://github.com/apollographql/apollo-rs/pull/287
  [issue/288]: https://github.com/apollographql/apollo-rs/issues/288

# [0.2.0](https://crates.io/crates/apollo-compiler/0.2.0) - 2022-08-16

## Breaking
- **inline_fragment().type_condition() returns Option<&str> - [lrlna], [pull/282]**

  Instead of returning `Option<&String>`, we now return `Option<&str>`

  [lrlna]: https://github.com/lrlna
  [pull/272]: https://github.com/apollographql/apollo-rs/pull/282

- **Value -> DefaultValue for default_value fields - [lrlna], [pull/276]**

  default_value getters in Input Value Definitions and Variable Definitions now
  return `DefaultValue` type. This is a type alias to Value, and makes it
  consistent with the GraphQL spec.

  [lrlna]: https://github.com/lrlna
  [pull/276]: https://github.com/apollographql/apollo-rs/pull/276

## Fixes
- **add type information to inline fragments  - [lrlna], [pull/282], [issue/280]**

  Inline fragments were missing type correct type information. We now search for
  applicable type's fields if a type condition exists, otherwise infer
  information from the current type in scope ([as per spec](https://spec.graphql.org/October2021/#sel-GAFbfJABAB_F3kG ))
  .Inline fragments

  [lrlna]: https://github.com/lrlna
  [pull/282]: https://github.com/apollographql/apollo-rs/pull/282
  [issue/280]: https://github.com/apollographql/apollo-rs/issues/280

- **fix cycle error in fragment spreads referencing fragments - [lrlna], [pull/283], [issue/281]**

  Because fragment definitions can have fragment spreads, we are running into a
  self-referential cycle when searching for a fragment definition to get its id.
  Instead, search for the fragment definition when getting a fragment in a
  fragment spread in the wrapper API.

  [lrlna]: https://github.com/lrlna
  [pull/283]: https://github.com/apollographql/apollo-rs/pull/283
  [issue/281]: https://github.com/apollographql/apollo-rs/issues/281

## Features
- **pub use ApolloDiagnostic - [EverlastingBugstopper], [pull/268]**

  Exports ApolloDiagnostic to allow users to use it in other contexts.

  [EverlastingBugstopper]: https://github.com/EverlastingBugstopper
  [pull/268]: https://github.com/apollographql/apollo-rs/pull/268

- **default_value getter in input_value_definition - [lrlna], [pull/273]**

  A getter for default values in input value definitions.

  [lrlna]: https://github.com/lrlna
  [pull/273]: https://github.com/apollographql/apollo-rs/pull/273

- **feat(compiler): add db.find_union_by_name() - [lrlna], [pull/272]**

  Allows to query unions in the database.

  [lrlna]: https://github.com/lrlna
  [pull/272]: https://github.com/apollographql/apollo-rs/pull/272

- **adds inline_fragments getter to SelectionSet - [lrlna], [pull/282]**

  Convenience method to get all inline fragments in the current selection set.

  [lrlna]: https://github.com/lrlna
  [pull/282]: https://github.com/apollographql/apollo-rs/pull/282

# [0.1.0](https://crates.io/crates/apollo-compiler/0.1.0) - 2022-07-27

Introducing `apollo-compiler`!

A query-based compiler for the GraphQL language.

The compiler provides validation and context for GraphQL documents with a comprehensive API.

This is still a work in progress, for outstanding issues, checkout out the
[apollo-compiler label] in our issue tracker.

[apollo-compiler label]: https://github.com/apollographql/apollo-rs/labels/apollo-compiler
