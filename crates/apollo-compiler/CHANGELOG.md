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