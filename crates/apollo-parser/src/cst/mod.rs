//! Typed Concrete Syntax Tree module to access nodes in the tree.
//!
//! The nodes described here are those also described in the [GraphQL grammar],
//! with a few exceptions. For example, for easy of querying the CST we do not
//! separate `Definition` into `ExecutableDefinition` and
//! `TypeSystemDefinitionOrExtension`. Instead, all possible definitions and
//! extensions can be accessed with `Definition`.
//!
//! Each struct in this module has getter methods to access information that's
//! part of its node. For example, as per spec a `UnionTypeDefinition` is defined as follows:
//!
//! ```ungram
//! UnionTypeDefinition =
//!   Description? 'union' Name Directives? UnionMemberTypes?
//! ```
//!
//! It will then have getters for `Description`, union token, `Name`,
//! `Directives` and `UnionMemberTypes`. Checkout documentation for the Struct
//! you're working with to find out its exact API.
//!
//! ## Example
//! This example parses a subgraph schema and looks at the various Definition Names.
//!
//! ```rust
//! use apollo_parser::{cst, Parser};
//!
//! let schema = r#"
//! directive @tag(name: String!) repeatable on FIELD_DEFINITION
//!
//! type ProductVariation {
//!   id: ID!
//! }
//! scalar UUID @specifiedBy(url: "https://tools.ietf.org/html/rfc4122")
//!
//! union SearchResult = Photo | Person
//!
//! extend type Query {
//!   allProducts: [Product]
//!   product(id: ID!): Product
//! }
//! "#;
//! let parser = Parser::new(schema);
//! let cst = parser.parse();
//!
//! assert_eq!(0, cst.errors().len());
//! let document = cst.document();
//! for definition in document.definitions() {
//!     match definition {
//!         cst::Definition::DirectiveDefinition(directive) => {
//!             assert_eq!(
//!                 directive
//!                     .name()
//!                     .expect("Cannot get directive name.")
//!                     .text()
//!                     .as_ref(),
//!                 "tag"
//!             )
//!         }
//!         cst::Definition::ObjectTypeDefinition(object_type) => {
//!             assert_eq!(
//!                 object_type
//!                     .name()
//!                     .expect("Cannot get object type definition name.")
//!                     .text()
//!                     .as_ref(),
//!                 "ProductVariation"
//!             )
//!         }
//!         cst::Definition::UnionTypeDefinition(union_type) => {
//!             assert_eq!(
//!                 union_type
//!                     .name()
//!                     .expect("Cannot get union type definition name.")
//!                     .text()
//!                     .as_ref(),
//!                 "SearchResult"
//!             )
//!         }
//!         cst::Definition::ScalarTypeDefinition(scalar_type) => {
//!             assert_eq!(
//!                 scalar_type
//!                     .name()
//!                     .expect("Cannot get scalar type definition name.")
//!                     .text()
//!                     .as_ref(),
//!                 "UUID"
//!             )
//!         }
//!         cst::Definition::ObjectTypeExtension(object_type) => {
//!             assert_eq!(
//!                 object_type
//!                     .name()
//!                     .expect("Cannot get object type extension name.")
//!                     .text()
//!                     .as_ref(),
//!                 "Query"
//!             )
//!         }
//!         _ => unimplemented!(),
//!     }
//! }
//! ```
//!
//! [GraphQL grammar]: https://spec.graphql.org/October2021/#sec-Document-Syntax
mod generated;
mod node_ext;

use std::marker::PhantomData;

use crate::{SyntaxKind, SyntaxNodeChildren, SyntaxToken};

pub use crate::{parser::SyntaxNodePtr, SyntaxNode};

pub use generated::nodes::*;

/// The main trait to go from untyped `SyntaxNode`  to a typed CST. The
/// conversion itself has zero runtime cost: CST and syntax nodes have exactly
/// the same representation: a pointer to the tree root and a pointer to the
/// node itself.
pub trait CstNode {
    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxNode;

    fn source_string(&self) -> String {
        self.syntax().to_string()
    }

    fn clone_for_update(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_for_update()).unwrap()
    }

    fn clone_subtree(&self) -> Self
    where
        Self: Sized,
    {
        Self::cast(self.syntax().clone_subtree()).unwrap()
    }
}

/// Like `CstNode`, but wraps tokens rather than interior nodes.
pub trait CstToken {
    fn can_cast(token: SyntaxKind) -> bool
    where
        Self: Sized;

    fn cast(syntax: SyntaxToken) -> Option<Self>
    where
        Self: Sized;

    fn syntax(&self) -> &SyntaxToken;

    fn text(&self) -> &str {
        self.syntax().text()
    }
}

/// An iterator over `SyntaxNode` children of a particular CST type.
#[derive(Debug, Clone)]
pub struct CstChildren<N> {
    inner: SyntaxNodeChildren,
    ph: PhantomData<N>,
}

impl<N> CstChildren<N> {
    fn new(parent: &SyntaxNode) -> Self {
        CstChildren {
            inner: parent.children(),
            ph: PhantomData,
        }
    }
}

impl<N: CstNode> Iterator for CstChildren<N> {
    type Item = N;
    fn next(&mut self) -> Option<N> {
        self.inner.find_map(N::cast)
    }
}

mod support {
    use super::{CstChildren, CstNode, SyntaxKind, SyntaxNode, SyntaxToken};

    pub(super) fn child<N: CstNode>(parent: &SyntaxNode) -> Option<N> {
        parent.children().find_map(N::cast)
    }

    pub(super) fn children<N: CstNode>(parent: &SyntaxNode) -> CstChildren<N> {
        CstChildren::new(parent)
    }

    pub(super) fn token(parent: &SyntaxNode, kind: SyntaxKind) -> Option<SyntaxToken> {
        parent
            .children_with_tokens()
            .filter_map(|it| it.into_token())
            .find(|it| it.kind() == kind)
    }
}
