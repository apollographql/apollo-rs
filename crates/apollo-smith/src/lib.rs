#![doc = include_str!("../README.md")]

pub(crate) mod argument;
pub(crate) mod definition;
pub(crate) mod description;
pub(crate) mod directive;
pub(crate) mod document;
pub(crate) mod enum_;
pub(crate) mod field;
pub(crate) mod fragment;
pub mod generators;
pub(crate) mod implements_graph;
pub(crate) mod input_object;
pub(crate) mod input_value;
pub(crate) mod interface;
pub(crate) mod name;
pub(crate) mod object;
pub(crate) mod operation;
pub mod random;
pub(crate) mod response;
pub(crate) mod scalar;
pub(crate) mod schema;
pub(crate) mod selection_set;
#[cfg(test)]
pub(crate) mod test_support;
pub(crate) mod ty;
pub(crate) mod union;
pub(crate) mod variable;

use crate::definition::TypeDefinition;
use apollo_compiler::ast;
use apollo_compiler::ast::Argument;
use apollo_compiler::ast::DirectiveDefinition;
use apollo_compiler::ast::Document;
use apollo_compiler::ast::FragmentDefinition;
use apollo_compiler::ast::OperationDefinition;
use apollo_compiler::ast::Type;
use apollo_compiler::coordinate::TypeAttributeCoordinate;
use apollo_compiler::parser::SourceMap;
use apollo_compiler::Name;
use apollo_compiler::Node;
pub use arbitrary::Result;
pub use arbitrary::Unstructured;
pub use document::ArbitraryDocument;
pub use generators::BooleanGenerator;
pub use generators::FloatGenerator;
pub use generators::Generator;
pub use generators::Generators;
pub use generators::IdGenerator;
pub use generators::IntGenerator;
pub use generators::StringGenerator;
use indexmap::IndexMap;
pub use random::RandProvider;
pub use random::RandomProvider;
pub use random::ResponseError;
pub use response::ResponseBuilder;
pub use serde_json_bytes::Value;
use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Debug, Clone, Copy)]
enum DefinitionKind {
    Scalar,
    Enum,
    Interface,
    Object,
    Union,
    InputObject,
    Directive,
    Fragment,
    Operation,
}

/// DocumentBuilder is a struct to build an arbitrary valid GraphQL document
///
/// ```ignore
/// // fuzz/fuzz_targets/my_apollo_smith_fuzz_target.rs
/// #![no_main]
///
/// use apollo_smith::{DocumentBuilder, Unstructured};
/// use libfuzzer_sys::fuzz_target;
///
/// fuzz_target!(|input: &[u8]| {
///     let mut u = Unstructured::new(input);
///     let Ok(document) = DocumentBuilder::new(&mut u).build() else {
///         return;
///     };
///     let document_str = document.to_string();
///
///     // Your code here...
/// });
/// ```
pub struct DocumentBuilder<'a> {
    pub(crate) u: &'a mut Unstructured<'a>,
    pub(crate) input_object_type_defs: Vec<ast::Definition>,
    pub(crate) object_type_defs: Vec<ast::Definition>,
    pub(crate) interface_type_defs: Vec<ast::Definition>,
    pub(crate) union_type_defs: Vec<ast::Definition>,
    pub(crate) enum_type_defs: Vec<ast::Definition>,
    pub(crate) scalar_type_defs: Vec<ast::Definition>,
    pub(crate) schema_def: Option<Node<ast::SchemaDefinition>>,
    sources: SourceMap,
    definitions: Vec<ast::Definition>,
    pub(crate) directive_defs: Vec<Node<DirectiveDefinition>>,
    pub(crate) operation_defs: Vec<Node<OperationDefinition>>,
    pub(crate) fragment_defs: Vec<Node<FragmentDefinition>>,
    // A graph with edges representing the "implements" relationship between types
    pub(crate) implements_graph: implements_graph::ImplementsGraph,
    // Composite definitions being selected while generating an operation or fragment
    pub(crate) stack: Vec<ast::Definition>,
    // Useful to keep the same arguments for a specific field on a specific type
    pub(crate) chosen_arguments: IndexMap<TypeAttributeCoordinate, Vec<Argument>>,
    // Set while generating an operation, which may define variables
    pub(crate) operation_variables: Option<variable::OperationVariables>,
    // Fields selected so far in each operation or fragment definition being
    // generated, so later selections stay mergeable with earlier ones
    pub(crate) merge_tracking: Vec<selection_set::MergeTracking>,
    // What each fragment definition selects, captured once since fragments
    // don't change after they are generated
    pub(crate) fragment_selections: IndexMap<Name, selection_set::SelectionsByResponseName>,
    // How many selection sets deep each fragment definition nests
    pub(crate) fragment_depths: IndexMap<Name, usize>,
    // How many selection sets deep the selection being generated is
    pub(crate) selection_depth: usize,
    pub(crate) used_type_names: HashSet<String>,
}

impl Debug for DocumentBuilder<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentBuilder")
            .field("input_object_type_defs", &self.input_object_type_defs)
            .field("object_type_defs", &self.object_type_defs)
            .field("interface_type_defs", &self.interface_type_defs)
            .field("union_type_defs", &self.union_type_defs)
            .field("enum_type_defs", &self.enum_type_defs)
            .field("scalar_type_defs", &self.scalar_type_defs)
            .field("schema_def", &self.schema_def)
            .field("directive_defs", &self.directive_defs)
            .field("operation_defs", &self.operation_defs)
            .field("fragment_defs", &self.fragment_defs)
            .finish()
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an instance of `DocumentBuilder`
    pub fn new(u: &'a mut Unstructured<'a>) -> Self {
        Self {
            u,
            object_type_defs: Vec::new(),
            interface_type_defs: Vec::new(),
            enum_type_defs: Vec::new(),
            schema_def: None,
            sources: SourceMap::default(),
            definitions: Vec::new(),
            directive_defs: Vec::new(),
            operation_defs: Vec::new(),
            fragment_defs: Vec::new(),
            scalar_type_defs: Vec::new(),
            union_type_defs: Vec::new(),
            input_object_type_defs: Vec::new(),
            implements_graph: implements_graph::ImplementsGraph::new(),
            stack: Vec::new(),
            chosen_arguments: IndexMap::new(),
            operation_variables: None,
            merge_tracking: Vec::new(),
            fragment_selections: IndexMap::new(),
            fragment_depths: IndexMap::new(),
            selection_depth: 0,
            used_type_names: HashSet::new(),
        }
    }

    /// Generate random definitions and return the resulting [`Document`].
    pub fn build(mut self) -> Result<Document> {
        while !self.u.is_empty() {
            let kinds = self.available_definition_kinds();
            let kind = *self.u.choose(&kinds)?;
            let Err(e) = self.definition(kind) else {
                continue;
            };
            // Some definitions can't be built from the current state, such as
            // an extension with nothing left to add, so that turn is skipped.
            if !matches!(e, arbitrary::Error::IncorrectFormat) {
                return Err(e);
            }
        }

        // A schema needs a query root, and a document an operation.
        if self.object_type_defs.is_empty() {
            self.definition(DefinitionKind::Object)?;
        }
        if self.operation_defs.is_empty() {
            self.definition(DefinitionKind::Operation)?;
        }

        self.prune_unused_fragments();

        // Adding another operation may have named an earlier anonymous operation.
        for (definition, operation) in self
            .definitions
            .iter_mut()
            .filter_map(|definition| match definition {
                ast::Definition::OperationDefinition(operation) => Some(operation),
                _ => None,
            })
            .zip(self.operation_defs)
        {
            *definition = operation;
        }
        Ok(Document {
            sources: self.sources,
            definitions: self.definitions,
        })
    }

    /// Seed generation with a compiler document, including its extensions.
    pub fn with_document(u: &'a mut Unstructured<'a>, document: Document) -> Result<Self> {
        let mut builder = Self::new(u);
        builder.sources = document.sources;
        for definition in document.definitions {
            builder.add_definition(definition);
        }
        builder.resolve_fragments();
        Ok(builder)
    }

    fn add_definition(&mut self, definition: ast::Definition) {
        self.definitions.push(definition.clone());
        if let Some(name) = definition.name() {
            self.used_type_names.insert(name.to_string());
        }
        if matches!(
            definition,
            ast::Definition::ObjectTypeDefinition(_)
                | ast::Definition::ObjectTypeExtension(_)
                | ast::Definition::InterfaceTypeDefinition(_)
                | ast::Definition::InterfaceTypeExtension(_)
        ) {
            self.implements_graph.node_for(definition.type_name());
            for parent in definition.interfaces() {
                self.implements_graph
                    .add_edge(definition.type_name(), parent);
            }
        }
        match definition {
            ast::Definition::OperationDefinition(d) => self.operation_defs.push(d),
            ast::Definition::FragmentDefinition(d) => self.fragment_defs.push(d),
            ast::Definition::DirectiveDefinition(d) => self.directive_defs.push(d),
            ast::Definition::SchemaDefinition(d) => self.schema_def = Some(d),
            d @ (ast::Definition::ScalarTypeDefinition(_)
            | ast::Definition::ScalarTypeExtension(_)) => self.scalar_type_defs.push(d),
            d @ (ast::Definition::ObjectTypeDefinition(_)
            | ast::Definition::ObjectTypeExtension(_)) => self.object_type_defs.push(d),
            d @ (ast::Definition::InterfaceTypeDefinition(_)
            | ast::Definition::InterfaceTypeExtension(_)) => self.interface_type_defs.push(d),
            d @ (ast::Definition::UnionTypeDefinition(_)
            | ast::Definition::UnionTypeExtension(_)) => self.union_type_defs.push(d),
            d
            @ (ast::Definition::EnumTypeDefinition(_) | ast::Definition::EnumTypeExtension(_)) => {
                self.enum_type_defs.push(d)
            }
            d @ (ast::Definition::InputObjectTypeDefinition(_)
            | ast::Definition::InputObjectTypeExtension(_)) => self.input_object_type_defs.push(d),
            _ => {}
        }
    }

    fn resolve_fragments(&mut self) {
        loop {
            let resolved: Vec<_> = self
                .fragment_defs
                .iter()
                .filter(|f| !self.fragment_selections.contains_key(&f.name))
                .filter_map(|f| {
                    Some((
                        f.name.clone(),
                        self.selections_of_fragment(f)?,
                        self.nesting_depth(&f.selection_set)?,
                    ))
                })
                .collect();
            if resolved.is_empty() {
                break;
            }
            for (name, selections, depth) in resolved {
                self.fragment_selections.insert(name.clone(), selections);
                self.fragment_depths.insert(name, depth);
            }
        }
    }

    pub(crate) fn operation_roots(&self) -> Vec<(ast::OperationType, Name)> {
        let mut roots = self
            .schema_def
            .as_ref()
            .map(|schema| {
                schema
                    .root_operations
                    .iter()
                    .map(|root| (**root).clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if self.schema_def.is_none() {
            for (operation, name) in [
                (ast::OperationType::Query, "Query"),
                (ast::OperationType::Mutation, "Mutation"),
                (ast::OperationType::Subscription, "Subscription"),
            ] {
                if let Some(definition) = self.object_type_defs.iter().find(|definition| {
                    definition.type_name() == name && !definition.is_extension_definition()
                }) {
                    roots.push((operation, definition.type_name().clone()));
                }
            }
        }
        for definition in &self.definitions {
            if let ast::Definition::SchemaExtension(extension) = definition {
                roots.extend(
                    extension
                        .root_operations
                        .iter()
                        .map(|root| (**root).clone()),
                );
            }
        }
        // Mutation and subscription roots alone do not form a valid schema.
        if roots.iter().any(|(operation, _)| operation.is_query()) {
            roots
        } else {
            Vec::new()
        }
    }

    fn available_definition_kinds(&self) -> Vec<DefinitionKind> {
        let has_objects = !self.object_type_defs.is_empty();
        [
            (DefinitionKind::Scalar, true),
            (DefinitionKind::Enum, true),
            (DefinitionKind::Interface, true),
            (DefinitionKind::Object, true),
            (DefinitionKind::Union, has_objects),
            (DefinitionKind::InputObject, true),
            (DefinitionKind::Directive, true),
            (DefinitionKind::Fragment, has_objects),
            (DefinitionKind::Operation, has_objects),
        ]
        .into_iter()
        .filter_map(|(kind, available)| available.then_some(kind))
        .collect()
    }

    fn definition(&mut self, kind: DefinitionKind) -> Result<()> {
        let definition = match kind {
            DefinitionKind::Scalar => self.scalar_type_definition()?,
            DefinitionKind::Enum => self.enum_type_definition()?,
            DefinitionKind::Interface => self.interface_type_definition()?,
            DefinitionKind::Object => self.object_type_definition()?,
            DefinitionKind::Union => self.union_type_definition()?,
            DefinitionKind::InputObject => self.input_object_type_definition()?,
            DefinitionKind::Directive => self.directive_def()?.into(),
            DefinitionKind::Fragment => self.fragment_definition()?.into(),
            DefinitionKind::Operation => return self.document_operation(),
        };
        self.add_definition(definition);
        Ok(())
    }

    /// Add an operation, generating the schema definition first if this is
    /// the first operation. An anonymous operation must be the only one in a
    /// document, so an anonymous first operation is named once a second one
    /// is added.
    fn document_operation(&mut self) -> Result<()> {
        if self.operation_roots().is_empty() {
            let schema = self.schema_definition()?;
            self.add_definition(schema.into());
        }
        let require_named = !self.operation_defs.is_empty();
        let Some(operation) = self.operation_definition_in_document(require_named)? else {
            return Ok(());
        };
        if let [first] = self.operation_defs.as_slice() {
            if first.name.is_none() {
                let name = self.type_name()?;
                self.operation_defs[0].make_mut().name = Some(name);
            }
        }
        self.add_definition(operation.into());
        Ok(())
    }

    /// Returns whether the provided `Unstructured` is now empty
    pub fn input_exhausted(&self) -> bool {
        self.u.is_empty()
    }

    pub(crate) fn stack_ty(&mut self, ty: &Type) -> bool {
        if crate::ty::is_builtin(ty) {
            return false;
        }
        let type_name = ty.inner_named_type();

        if let Some(object_ty) = self
            .object_type_defs
            .iter()
            .find(|object_ty_def| object_ty_def.type_name() == type_name)
            .cloned()
        {
            self.stack.push(object_ty);
            true
        } else if let Some(itf_type) = self
            .interface_type_defs
            .iter()
            .find(|itf_type_def| itf_type_def.type_name() == type_name)
            .cloned()
        {
            self.stack.push(itf_type);
            true
        } else if self
            .union_type_defs
            .iter()
            .any(|u| u.type_name() == type_name)
        {
            self.stack.push(
                ast::UnionTypeDefinition {
                    name: type_name.clone(),
                    description: None,
                    members: self.union_members(type_name).into_iter().collect(),
                    directives: ast::DirectiveList::new(),
                }
                .into(),
            );
            true
        } else if self
            .enum_type_defs
            .iter()
            .any(|e| e.type_name() == type_name)
            || self
                .scalar_type_defs
                .iter()
                .any(|s| s.type_name() == type_name)
        {
            false
        } else {
            panic!("'{type_name:?}' is not a known output type");
        }
    }

    /// Validation requires every fragment definition to be referenced by some
    /// operation (directly or transitively). Drop fragments that never get
    /// spread to avoid producing invalid documents.
    ///
    /// We walk transitively from operations so chains like `op -> A -> B` keep
    /// both A and B, while `A -> B` with no operation referencing A drops both
    /// (B is only reachable through A, which is itself unused).
    fn prune_unused_fragments(&mut self) {
        let reachable =
            fragment::reachable_fragment_names(&self.operation_defs, &self.fragment_defs);
        self.fragment_defs.retain(|f| reachable.contains(&f.name));
        self.definitions.retain(|definition| match definition {
            ast::Definition::FragmentDefinition(fragment) => reachable.contains(&fragment.name),
            _ => true,
        });
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::documents;
    use crate::test_support::prop_assert_valid;
    use crate::DefinitionKind;
    use crate::DocumentBuilder;
    use apollo_compiler::ast;
    use apollo_compiler::name;
    use arbitrary::Unstructured;
    use proptest::prelude::*;

    #[test]
    fn conventional_mutation_and_subscription_types_still_get_a_query_root() {
        for root in ["Mutation", "Subscription"] {
            let seed = ast::Document::parse(format!("type {root} {{ a: Int }}"), "schema.graphql")
                .expect("parse seed");
            let mut u = Unstructured::new(&[]);
            let document = DocumentBuilder::with_document(&mut u, seed)
                .expect("seed builder")
                .build()
                .expect("build document");
            document
                .to_mixed_validate()
                .expect("generated schema has a query root");
        }
    }

    #[test]
    fn implicit_schema_keeps_all_conventional_roots() {
        let seed = ast::Document::parse(
            "type Query { q: Int } type Mutation { m: Int } type Subscription { s: Int }",
            "schema.graphql",
        )
        .expect("parse implicit schema");
        seed.to_schema_validate().expect("valid implicit schema");
        let mut u = Unstructured::new(&[]);
        let builder = DocumentBuilder::with_document(&mut u, seed).expect("seed builder");
        assert_eq!(
            builder.operation_roots(),
            vec![
                (ast::OperationType::Query, name!(Query)),
                (ast::OperationType::Mutation, name!(Mutation)),
                (ast::OperationType::Subscription, name!(Subscription)),
            ]
        );
        let document = builder.build().expect("build document");
        assert!(document
            .definitions
            .iter()
            .all(|definition| definition.as_schema_definition().is_none()));
        document
            .to_mixed_validate()
            .expect("valid implicit schema output");
    }

    #[test]
    fn schema_extension_can_supply_the_query_root() {
        let seed = ast::Document::parse(
            "extend schema { query: Q } type Q { q: Int } type Mutation { m: Int }",
            "schema.graphql",
        )
        .expect("parse extended implicit schema");
        seed.to_schema_validate()
            .expect("valid extended implicit schema");
        let mut u = Unstructured::new(&[]);
        let builder = DocumentBuilder::with_document(&mut u, seed).expect("seed builder");
        assert_eq!(
            builder.operation_roots(),
            vec![
                (ast::OperationType::Mutation, name!(Mutation)),
                (ast::OperationType::Query, name!(Q)),
            ]
        );
        builder
            .build()
            .expect("build document")
            .to_mixed_validate()
            .expect("valid extended schema output");
    }

    #[test]
    fn definitions_keep_generation_order() {
        let mut u = Unstructured::new(&[]);
        let mut builder = DocumentBuilder::new(&mut u);
        for kind in [
            DefinitionKind::Enum,
            DefinitionKind::Object,
            DefinitionKind::Scalar,
        ] {
            builder.definition(kind).expect("generate definition");
        }
        let document = builder.build().expect("build document");
        assert!(document.definitions[0].as_enum_type_definition().is_some());
        assert!(document.definitions[1]
            .as_object_type_definition()
            .is_some());
        assert!(document.definitions[2]
            .as_scalar_type_definition()
            .is_some());
        document.to_mixed_validate().expect("valid mixed ordering");
    }

    #[test]
    fn compiler_input_preserves_extensions_and_operation_variables() {
        let source = "schema { query: Query }
            extend schema { mutation: Mutation }
            directive @tag on SCALAR
            scalar S
            extend scalar S @tag
            type Query { f(arg: Int): O }
            extend type Query { s: S }
            type Mutation { f: O }
            interface I { id: ID }
            extend interface I { other: Int }
            type O implements I { id: ID, other: Int }
            extend type O { extra: String }
            type P { id: ID }
            union U = O
            extend union U = P
            enum E { A }
            extend enum E { B }
            input In { a: Int }
            extend input In { b: String }
            query Keep($x: Int) { f(arg: $x) { id } }";
        let document = apollo_compiler::ast::Document::parse(source, "input.graphql")
            .expect("parse compiler document");
        document.to_mixed_validate().expect("valid input");
        let expected = document.definitions.clone();
        let mut u = arbitrary::Unstructured::new(&[]);
        let generated: apollo_compiler::ast::Document =
            crate::DocumentBuilder::with_document(&mut u, document)
                .expect("seed builder")
                .build()
                .expect("build seeded document");
        assert_eq!(generated.definitions.len(), expected.len());
        assert_eq!(generated.definitions, expected);
        for (definition, original) in generated.definitions.iter().zip(&expected) {
            assert_eq!(definition.location(), original.location(), "{definition}");
        }
        generated
            .to_mixed_validate()
            .expect("valid seeded document");
    }

    proptest! {
        #[test]
        fn builds_valid_documents(
            case in documents(prop::collection::vec(any::<u8>(), 0..=16384)),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }

    // With no input to choose from, a document still gets a query root and
    // an operation selecting from it.
    #[test]
    fn exhausted_input_builds_a_minimal_valid_document() {
        let doc = DocumentBuilder::new(&mut Unstructured::new(&[]))
            .build()
            .expect("document from exhausted input");
        assert_eq!(
            doc.definitions
                .iter()
                .filter(|d| d.as_object_type_definition().is_some())
                .count(),
            1
        );
        assert!(doc
            .definitions
            .iter()
            .any(|d| d.as_schema_definition().is_some()));
        assert_eq!(
            doc.definitions
                .iter()
                .filter(|d| d.as_operation_definition().is_some())
                .count(),
            1
        );
        let doc = doc.to_string();
        let errors = apollo_compiler::ast::Document::parse(&doc, "smith.graphql")
            .expect("smith output must parse")
            .to_mixed_validate();
        assert!(errors.is_ok(), "{doc}");
    }

    // An anonymous operation must be the only one in a document.
    #[test]
    fn anonymous_operation_is_named_once_another_is_added() {
        let schema = apollo_compiler::ast::Document::parse(
            "schema { query: Query } type Query { a: Int }",
            "schema.graphql",
        )
        .expect("parse compiler document");
        let mut u = Unstructured::new(&[]);
        let mut builder = DocumentBuilder::with_document(&mut u, schema).expect("builder");
        builder.document_operation().expect("first operation");
        assert!(builder.operation_defs[0].name.is_none());
        builder.document_operation().expect("second operation");
        assert!(builder.operation_defs.iter().all(|op| op.name.is_some()));
        let document = builder
            .build()
            .expect("build document with named operations");
        assert!(document
            .definitions
            .iter()
            .filter_map(|definition| { definition.as_operation_definition() })
            .all(|operation| operation.name.is_some()));
        document
            .to_mixed_validate()
            .expect("valid named operations");
    }
}
