use crate::{
    directive::DirectiveDef, enum_::EnumTypeDef, fragment::FragmentDef,
    input_object::InputObjectTypeDef, interface::InterfaceTypeDef, object::ObjectTypeDef,
    operation::OperationDef, scalar::ScalarTypeDef, schema::SchemaDef, union::UnionTypeDef,
};

/// The `__Document` type represents a GraphQL document.A GraphQL Document describes a complete file or request string operated on by a GraphQL service or client.
/// A document contains multiple definitions, either executable or representative of a GraphQL type system.
///
/// *Document*:
///     OperationDefinition*
///     FragmentDefinition*
///     SchemaDefinition*
///     ScalarTypeDefinition*
///     ObjectTypeDefinition*
///     InterfaceTypeDefinition*
///     UnionTypeDefinition*
///     EnumTypeDefinition*
///     InputObjectDefinition*
///     DirectiveDefinition*
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Document).
#[derive(Debug)]
pub struct Document {
    pub(crate) operation_definitions: Vec<OperationDef>,
    pub(crate) fragment_definitions: Vec<FragmentDef>,
    pub(crate) schema_definitions: Vec<SchemaDef>,
    // Type definitions
    pub(crate) scalar_type_definitions: Vec<ScalarTypeDef>,
    pub(crate) object_type_definitions: Vec<ObjectTypeDef>,
    pub(crate) interface_type_definitions: Vec<InterfaceTypeDef>,
    pub(crate) union_type_definitions: Vec<UnionTypeDef>,
    pub(crate) enum_type_definitions: Vec<EnumTypeDef>,
    pub(crate) input_object_type_definitions: Vec<InputObjectTypeDef>,
    pub(crate) directive_definitions: Vec<DirectiveDef>,
}

impl From<Document> for apollo_encoder::Document {
    fn from(doc: Document) -> Self {
        let mut new_doc = Self::new();
        doc.fragment_definitions
            .into_iter()
            .for_each(|fragment_def| new_doc.fragment(fragment_def.into()));
        doc.scalar_type_definitions
            .into_iter()
            .for_each(|scalar_type_def| new_doc.scalar(scalar_type_def.into()));
        doc.schema_definitions
            .into_iter()
            .for_each(|schema_def| new_doc.schema(schema_def.into()));
        doc.object_type_definitions
            .into_iter()
            .for_each(|object_type_def| new_doc.object(object_type_def.into()));
        doc.union_type_definitions
            .into_iter()
            .for_each(|union_type_def| new_doc.union(union_type_def.into()));
        doc.input_object_type_definitions
            .into_iter()
            .for_each(|input_object_type_def| new_doc.input_object(input_object_type_def.into()));
        doc.interface_type_definitions
            .into_iter()
            .for_each(|interface_type_def| new_doc.interface(interface_type_def.into()));
        doc.enum_type_definitions
            .into_iter()
            .for_each(|enum_type_def| new_doc.enum_(enum_type_def.into()));
        doc.directive_definitions
            .into_iter()
            .for_each(|directive_def| new_doc.directive(directive_def.into()));
        doc.operation_definitions
            .into_iter()
            .for_each(|operation_def| new_doc.operation(operation_def.into()));

        new_doc
    }
}

impl From<Document> for String {
    fn from(doc: Document) -> Self {
        apollo_encoder::Document::from(doc).to_string()
    }
}
