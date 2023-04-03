# Changelog

All notable changes to `apollo-compiler` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- # [x.x.x] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
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
