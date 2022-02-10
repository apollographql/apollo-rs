# Changelog

All notable changes to `apollo-encoder` will be documented in this file.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# [0.2.0] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

- Delete `Schema` type. You should use `Document` instead.
- All types suffixed by `Def` are now suffixed by `Definition`
- `InputField` and `Field` has been separated
- Method `deprecated` has been deleted in favor of `directive` with your own directive

## Features

- Add support of directives on `SchemaDefinition`, `InputValueDefinition`, `ScalarDefinition`, `ObjectDefinition`, `InterfaceDefinition`, `UnionDefinition`, `EnumDefinition`, `InputObjectDefinition`
- Add support of `Operation`, `Fragment`, `Value`, `VariableDefinition`, `DirectiveDefinition`, `SelectionSet`
- Add support of extensions (`Schema`, `Scalar`, `Interface`, `Union`, `Enum`, `Input Object`).

## Fixes

## Maintenance

## Documentation

<!-- # [x.x.x] (unreleased) - 2021-mm-dd

> Important: X breaking changes below, indicated by **BREAKING**

## BREAKING

## Features

## Fixes

## Maintenance

## Documentation -->
