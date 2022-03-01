#![doc = include_str!("../README.md")]

pub(crate) mod argument;
pub(crate) mod description;
pub(crate) mod directive;
pub(crate) mod document;
pub(crate) mod enum_;
pub(crate) mod field;
pub(crate) mod fragment;
pub(crate) mod input_object;
pub(crate) mod input_value;
pub(crate) mod interface;
pub(crate) mod name;
pub(crate) mod object;
pub(crate) mod operation;
pub(crate) mod scalar;
pub(crate) mod schema;
pub(crate) mod selection_set;
pub(crate) mod ty;
pub(crate) mod union;
pub(crate) mod variable;

use std::fmt::Debug;

use arbitrary::Unstructured;

pub use arbitrary::Result;
pub use directive::DirectiveDef;
pub use document::Document;
pub use enum_::EnumTypeDef;
pub use fragment::FragmentDef;
pub use input_object::InputObjectTypeDef;
pub use interface::InterfaceTypeDef;
use name::Name;
pub use object::ObjectTypeDef;
pub use operation::OperationDef;
pub use scalar::ScalarTypeDef;
pub use schema::SchemaDef;
use ty::Ty;
pub use union::UnionTypeDef;

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
///     let gql_doc = DocumentBuilder::new(&mut u)?;
///     let document = gql_doc.finish();
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
    pub(crate) schema_defs: Vec<SchemaDef>,
    pub(crate) directive_defs: Vec<DirectiveDef>,
    pub(crate) operation_defs: Vec<OperationDef>,
    pub(crate) fragment_defs: Vec<FragmentDef>,
    // A stack to set current TypeDef
    pub(crate) stack: Vec<TypeDefinition>,
}

impl<'a> Debug for DocumentBuilder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DocumentBuilder")
            .field("input_object_type_defs", &self.input_object_type_defs)
            .field("object_type_defs", &self.object_type_defs)
            .field("interface_type_defs", &self.interface_type_defs)
            .field("union_type_defs", &self.union_type_defs)
            .field("enum_type_defs", &self.enum_type_defs)
            .field("scalar_type_defs", &self.scalar_type_defs)
            .field("schema_defs", &self.schema_defs)
            .field("directive_defs", &self.directive_defs)
            .field("operation_defs", &self.operation_defs)
            .field("fragment_defs", &self.fragment_defs)
            .finish()
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an instance of `DocumentBuilder`
    pub fn new(u: &'a mut Unstructured<'a>) -> Result<Self> {
        let mut builder = Self {
            u,
            object_type_defs: Vec::new(),
            interface_type_defs: Vec::new(),
            enum_type_defs: Vec::new(),
            schema_defs: Vec::new(),
            directive_defs: Vec::new(),
            operation_defs: Vec::new(),
            fragment_defs: Vec::new(),
            scalar_type_defs: Vec::new(),
            union_type_defs: Vec::new(),
            input_object_type_defs: Vec::new(),
            stack: Vec::new(),
        };

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let fragment_def = builder.fragment_definition()?;
            builder.fragment_defs.push(fragment_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let scalar_type_def = builder.scalar_type_definition()?;
            builder.scalar_type_defs.push(scalar_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let enum_type_def = builder.enum_type_definition()?;
            builder.enum_type_defs.push(enum_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let interface_type_def = builder.interface_type_definition()?;
            builder.interface_type_defs.push(interface_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let object_type_def = builder.object_type_definition()?;
            builder.object_type_defs.push(object_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let union_type_def = builder.union_type_definition()?;
            builder.union_type_defs.push(union_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let input_object_type_def = builder.input_object_type_definition()?;
            builder.input_object_type_defs.push(input_object_type_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let schema_def = builder.schema_definition()?;
            builder.schema_defs.push(schema_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let directive_def = builder.directive_def()?;
            builder.directive_defs.push(directive_def);
        }

        for _ in 0..builder.u.int_in_range(1..=50)? {
            let operation_def = builder.operation_definition()?;
            builder.operation_defs.push(operation_def);
        }

        Ok(builder)
    }

    /// Create an instance of `DocumentBuilder` given a `Document` to be able to call
    /// methods on DocumentBuilder and generate valid entities like for example an operation
    pub fn with_document(u: &'a mut Unstructured<'a>, document: Document) -> Result<Self> {
        let builder = Self {
            u,
            object_type_defs: document.object_type_definitions,
            interface_type_defs: document.interface_type_definitions,
            enum_type_defs: document.enum_type_definitions,
            schema_defs: document.schema_definitions,
            directive_defs: document.directive_definitions,
            operation_defs: document.operation_definitions,
            fragment_defs: document.fragment_definitions,
            scalar_type_defs: document.scalar_type_definitions,
            union_type_defs: document.union_type_definitions,
            input_object_type_defs: document.input_object_type_definitions,
            stack: Vec::new(),
        };

        Ok(builder)
    }

    /// Convert a `DocumentBuilder` into a GraphQL `Document`
    pub fn finish(self) -> Document {
        Document {
            schema_definitions: self.schema_defs,
            object_type_definitions: self.object_type_defs,
            interface_type_definitions: self.interface_type_defs,
            enum_type_definitions: self.enum_type_defs,
            directive_definitions: self.directive_defs,
            operation_definitions: self.operation_defs,
            fragment_definitions: self.fragment_defs,
            scalar_type_definitions: self.scalar_type_defs,
            union_type_definitions: self.union_type_defs,
            input_object_type_definitions: self.input_object_type_defs,
        }
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
            self.stack.push(TypeDefinition::Object(object_ty));
            true
        } else if let Some(_enum_ty) = self
            .enum_type_defs
            .iter()
            .find(|object_ty_def| &object_ty_def.name == type_name)
            .cloned()
        {
            false
        } else {
            todo!("need to implement for union, scalar, ...")
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TypeDefinition {
    Enum(EnumTypeDef),
    Object(ObjectTypeDef),
}

impl TypeDefinition {
    pub(crate) fn as_object(&self) -> Option<&ObjectTypeDef> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub(crate) fn need_selection_set(&self) -> bool {
        match self {
            TypeDefinition::Enum(_) => false,
            TypeDefinition::Object(_) => true,
        }
    }
}
