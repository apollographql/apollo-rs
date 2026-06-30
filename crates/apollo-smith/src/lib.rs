#![doc = include_str!("../README.md")]

pub(crate) mod argument;
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
pub(crate) mod snapshot_tests;
pub(crate) mod ty;
pub(crate) mod union;
pub(crate) mod variable;

use indexmap::IndexMap;
use std::fmt::Debug;

#[derive(Debug, Clone, thiserror::Error)]
pub enum FromError {
    #[error("parse tree is missing a node")]
    MissingNode,
    #[error("invalid i32")]
    ParseIntError(#[from] std::num::ParseIntError),
    #[error("invalid f64")]
    ParseFloatError(#[from] std::num::ParseFloatError),
    #[error("invalid boolean")]
    ParseBoolError(#[from] std::str::ParseBoolError),
}

use apollo_compiler::coordinate::TypeAttributeCoordinate;
pub use arbitrary::Result;
pub use arbitrary::Unstructured;
use argument::Argument;
pub use directive::DirectiveDef;
pub use document::Document;
pub use enum_::EnumTypeDef;
use field::FieldDef;
pub use fragment::FragmentDef;
pub use generators::BooleanGenerator;
pub use generators::FloatGenerator;
pub use generators::Generator;
pub use generators::Generators;
pub use generators::IdGenerator;
pub use generators::IntGenerator;
pub use generators::StringGenerator;
pub use input_object::InputObjectTypeDef;
pub use interface::InterfaceTypeDef;
use name::Name;
pub use object::ObjectTypeDef;
pub use operation::OperationDef;
pub use random::RandProvider;
pub use random::RandomProvider;
pub use random::ResponseError;
pub use response::ResponseBuilder;
pub use scalar::ScalarTypeDef;
pub use schema::SchemaDef;
pub use serde_json_bytes::Value;
use ty::Ty;
pub use union::UnionTypeDef;

const DEFAULT_MAX: usize = 50;

/// DocumentBuilder is a struct to build an arbitrary valid GraphQL document
///
/// ```compile_fail
/// // fuzz/fuzz_targets/my_apollo_smith_fuzz_target.rs
/// #![no_main]
///
/// use libfuzzer_sys::fuzz_target;
/// use arbitrary::Unstructured;
/// use apollo_smith::DocumentBuilder;
///
/// fuzz_target!(|input: &[u8]| {
///     let mut u = Unstructured::new(input);
///     let document = DocumentBuilder::new(&mut u).build()?;
///     let document_str = String::from(document);
///
///     // Your code here...
/// });
/// ```
pub struct DocumentBuilder<'a> {
    pub(crate) u: &'a mut Unstructured<'a>,
    pub(crate) input_object_type_defs: Vec<InputObjectTypeDef>,
    pub(crate) object_type_defs: Vec<ObjectTypeDef>,
    pub(crate) interface_type_defs: Vec<InterfaceTypeDef>,
    pub(crate) union_type_defs: Vec<UnionTypeDef>,
    pub(crate) enum_type_defs: Vec<EnumTypeDef>,
    pub(crate) scalar_type_defs: Vec<ScalarTypeDef>,
    pub(crate) schema_def: Option<SchemaDef>,
    pub(crate) directive_defs: Vec<DirectiveDef>,
    pub(crate) operation_defs: Vec<OperationDef>,
    pub(crate) fragment_defs: Vec<FragmentDef>,
    // A graph with edges representing the "implements" relationship between types
    pub(crate) implements_graph: implements_graph::ImplementsGraph,
    // A stack to set current ObjectTypeDef
    pub(crate) stack: Vec<Box<dyn StackedEntity>>,
    // Useful to keep the same arguments for a specific field on a specific type
    pub(crate) chosen_arguments: IndexMap<TypeAttributeCoordinate, Vec<Argument>>,
    // Maximum number of generated definitions per kind
    max_scalar_types: usize,
    max_enum_types: usize,
    max_interface_types: usize,
    max_object_types: usize,
    max_union_types: usize,
    max_input_object_types: usize,
    max_fragment_definitions: usize,
    max_directive_definitions: usize,
    max_operation_definitions: usize,
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
            directive_defs: Vec::new(),
            operation_defs: Vec::new(),
            fragment_defs: Vec::new(),
            scalar_type_defs: Vec::new(),
            union_type_defs: Vec::new(),
            input_object_type_defs: Vec::new(),
            implements_graph: implements_graph::ImplementsGraph::new(),
            stack: Vec::new(),
            chosen_arguments: IndexMap::new(),
            max_scalar_types: DEFAULT_MAX,
            max_enum_types: DEFAULT_MAX,
            max_interface_types: DEFAULT_MAX,
            max_object_types: DEFAULT_MAX,
            max_union_types: DEFAULT_MAX,
            max_input_object_types: DEFAULT_MAX,
            max_fragment_definitions: DEFAULT_MAX,
            max_directive_definitions: DEFAULT_MAX,
            max_operation_definitions: DEFAULT_MAX,
        }
    }

    /// Set the maximum number of scalar type definitions (default 50).
    pub fn max_scalar_types(mut self, max: usize) -> Self {
        self.max_scalar_types = max;
        self
    }

    /// Set the maximum number of enum type definitions (default 50).
    pub fn max_enum_types(mut self, max: usize) -> Self {
        self.max_enum_types = max;
        self
    }

    /// Set the maximum number of interface type definitions (default 50).
    pub fn max_interface_types(mut self, max: usize) -> Self {
        self.max_interface_types = max;
        self
    }

    /// Set the maximum number of object type definitions (default 50).
    pub fn max_object_types(mut self, max: usize) -> Self {
        self.max_object_types = max;
        self
    }

    /// Set the maximum number of union type definitions (default 50).
    pub fn max_union_types(mut self, max: usize) -> Self {
        self.max_union_types = max;
        self
    }

    /// Set the maximum number of input object type definitions (default 50).
    pub fn max_input_object_types(mut self, max: usize) -> Self {
        self.max_input_object_types = max;
        self
    }

    /// Set the maximum number of fragment definitions (default 50).
    pub fn max_fragment_definitions(mut self, max: usize) -> Self {
        self.max_fragment_definitions = max;
        self
    }

    /// Set the maximum number of directive definitions (default 50).
    pub fn max_directive_definitions(mut self, max: usize) -> Self {
        self.max_directive_definitions = max;
        self
    }

    /// Set the maximum number of operation definitions (default 50).
    pub fn max_operation_definitions(mut self, max: usize) -> Self {
        self.max_operation_definitions = max;
        self
    }

    /// Generate random definitions according to the configured maximums
    /// and return the resulting [`Document`].
    pub fn build(mut self) -> Result<Document> {
        for _ in 0..self.u.int_in_range(1..=self.max_scalar_types)? {
            let scalar_type_def = self.scalar_type_definition()?;
            self.scalar_type_defs.push(scalar_type_def);
        }

        for _ in 0..self.u.int_in_range(1..=self.max_enum_types)? {
            let enum_type_def = self.enum_type_definition()?;
            self.enum_type_defs.push(enum_type_def);
        }

        for _ in 0..self.u.int_in_range(1..=self.max_interface_types)? {
            let def = self.interface_type_definition()?;
            self.implements_graph.node_for(&def.name);
            for parent in &def.interfaces {
                self.implements_graph.add_edge(&def.name, parent);
            }
            self.interface_type_defs.push(def);
        }
        self.backfill_inherited_interface_fields();

        for _ in 0..self.u.int_in_range(1..=self.max_object_types)? {
            let def = self.object_type_definition()?;
            self.implements_graph.node_for(&def.name);
            for parent in &def.implements_interfaces {
                self.implements_graph.add_edge(&def.name, parent);
            }
            self.object_type_defs.push(def);
        }
        self.backfill_inherited_object_fields();

        for _ in 0..self.u.int_in_range(1..=self.max_union_types)? {
            let union_type_def = self.union_type_definition()?;
            self.union_type_defs.push(union_type_def);
        }

        for _ in 0..self.u.int_in_range(1..=self.max_input_object_types)? {
            let input_object_type_def = self.input_object_type_definition()?;
            self.input_object_type_defs.push(input_object_type_def);
        }

        for _ in 0..self.u.int_in_range(1..=self.max_fragment_definitions)? {
            let fragment_def = self.fragment_definition()?;
            self.fragment_defs.push(fragment_def);
        }

        for _ in 0..self.u.int_in_range(1..=self.max_directive_definitions)? {
            let directive_def = self.directive_def()?;
            self.directive_defs.push(directive_def);
        }

        let schema_def = self.schema_definition()?;
        self.schema_def = Some(schema_def);

        // An anonymous operation may only exist as the sole operation
        // in a document, so any time we're producing more than one,
        // every operation must be named.
        let num_ops = self.u.int_in_range(1..=self.max_operation_definitions)?;
        let require_named = num_ops > 1;
        for _ in 0..num_ops {
            let operation_def = self.operation_definition_in_document(require_named)?;
            if let Some(operation_def) = operation_def {
                self.operation_defs.push(operation_def);
            }
        }

        self.prune_unused_fragments();

        Ok(Document {
            schema_definition: self.schema_def,
            object_type_definitions: self.object_type_defs,
            interface_type_definitions: self.interface_type_defs,
            enum_type_definitions: self.enum_type_defs,
            directive_definitions: self.directive_defs,
            operation_definitions: self.operation_defs,
            fragment_definitions: self.fragment_defs,
            scalar_type_definitions: self.scalar_type_defs,
            union_type_definitions: self.union_type_defs,
            input_object_type_definitions: self.input_object_type_defs,
        })
    }

    /// Create an instance of `DocumentBuilder` given a `Document` to be able to call
    /// methods on DocumentBuilder and generate valid entities like for example an operation
    pub fn with_document(u: &'a mut Unstructured<'a>, document: Document) -> Result<Self> {
        let mut implements_graph = implements_graph::ImplementsGraph::new();
        for itf in &document.interface_type_definitions {
            implements_graph.node_for(&itf.name);
            for parent in &itf.interfaces {
                implements_graph.add_edge(&itf.name, parent);
            }
        }
        for obj in &document.object_type_definitions {
            implements_graph.node_for(&obj.name);
            for parent in &obj.implements_interfaces {
                implements_graph.add_edge(&obj.name, parent);
            }
        }
        let builder = Self {
            u,
            object_type_defs: document.object_type_definitions,
            interface_type_defs: document.interface_type_definitions,
            enum_type_defs: document.enum_type_definitions,
            schema_def: document.schema_definition,
            directive_defs: document.directive_definitions,
            operation_defs: document.operation_definitions,
            fragment_defs: document.fragment_definitions,
            scalar_type_defs: document.scalar_type_definitions,
            union_type_defs: document.union_type_definitions,
            input_object_type_defs: document.input_object_type_definitions,
            implements_graph,
            stack: Vec::new(),
            chosen_arguments: IndexMap::new(),
            max_scalar_types: DEFAULT_MAX,
            max_enum_types: DEFAULT_MAX,
            max_interface_types: DEFAULT_MAX,
            max_object_types: DEFAULT_MAX,
            max_union_types: DEFAULT_MAX,
            max_input_object_types: DEFAULT_MAX,
            max_fragment_definitions: DEFAULT_MAX,
            max_directive_definitions: DEFAULT_MAX,
            max_operation_definitions: DEFAULT_MAX,
        };

        Ok(builder)
    }

    /// Returns whether the provided `Unstructured` is now empty
    pub fn input_exhausted(&self) -> bool {
        self.u.is_empty()
    }

    pub(crate) fn stack_ty(&mut self, ty: &Ty) -> bool {
        if ty.is_builtin() {
            return false;
        }
        let type_name = ty.name();

        if let Some(object_ty) = self
            .object_type_defs
            .iter()
            .find(|object_ty_def| &object_ty_def.name == type_name)
            .cloned()
        {
            self.stack.push(Box::new(object_ty));
            true
        } else if let Some(itf_type) = self
            .interface_type_defs
            .iter()
            .find(|itf_type_def| &itf_type_def.name == type_name)
            .cloned()
        {
            self.stack.push(Box::new(itf_type));
            true
        } else if let Some(_enum_ty) = self
            .enum_type_defs
            .iter()
            .find(|object_ty_def| &object_ty_def.name == type_name)
            .cloned()
        {
            false
        } else {
            todo!("'{:?}' need to implement for union, scalar, ...", type_name);
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
    }
}

pub(crate) trait StackedEntity {
    fn name(&self) -> &Name;
    fn fields_def(&self) -> &[FieldDef];
}
