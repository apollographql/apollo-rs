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
    pub(crate) schema_definition: Option<SchemaDef>,
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
        if let Some(schema_def) = doc.schema_definition {
            new_doc.schema(schema_def.into());
        }
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

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Document> for Document {
    type Error = crate::FromError;

    fn try_from(doc: apollo_parser::ast::Document) -> Result<Self, Self::Error> {
        let mut enum_defs = Vec::new();
        let mut object_defs = Vec::new();
        let mut schema_def = None;
        let mut directive_defs = Vec::new();
        let mut scalar_defs = Vec::new();
        let mut operation_defs = Vec::new();
        let mut interface_defs = Vec::new();
        let mut union_defs = Vec::new();
        let mut input_object_defs = Vec::new();
        let mut fragment_defs = Vec::new();

        for definition in doc.definitions() {
            match definition {
                apollo_parser::ast::Definition::EnumTypeDefinition(enum_def) => {
                    enum_defs.push(EnumTypeDef::try_from(enum_def)?);
                }
                apollo_parser::ast::Definition::EnumTypeExtension(enum_def) => {
                    enum_defs.push(EnumTypeDef::try_from(enum_def)?);
                }
                apollo_parser::ast::Definition::ObjectTypeDefinition(obj_def) => {
                    object_defs.push(ObjectTypeDef::try_from(obj_def)?);
                }
                apollo_parser::ast::Definition::ObjectTypeExtension(obj_def) => {
                    object_defs.push(ObjectTypeDef::try_from(obj_def)?);
                }
                apollo_parser::ast::Definition::SchemaDefinition(schema_definition) => {
                    schema_def = Some(SchemaDef::try_from(schema_definition)?);
                }
                apollo_parser::ast::Definition::SchemaExtension(schema_definition) => {
                    schema_def = Some(SchemaDef::try_from(schema_definition)?);
                }
                apollo_parser::ast::Definition::DirectiveDefinition(dir_def) => {
                    directive_defs.push(DirectiveDef::try_from(dir_def)?);
                }
                apollo_parser::ast::Definition::ScalarTypeDefinition(scalar_def) => {
                    scalar_defs.push(ScalarTypeDef::try_from(scalar_def)?)
                }
                apollo_parser::ast::Definition::ScalarTypeExtension(scalar_def) => {
                    scalar_defs.push(ScalarTypeDef::try_from(scalar_def)?)
                }
                apollo_parser::ast::Definition::OperationDefinition(operation_def) => {
                    operation_defs.push(OperationDef::try_from(operation_def)?)
                }
                apollo_parser::ast::Definition::InterfaceTypeDefinition(interface_def) => {
                    interface_defs.push(InterfaceTypeDef::try_from(interface_def)?)
                }
                apollo_parser::ast::Definition::InterfaceTypeExtension(interface_def) => {
                    interface_defs.push(InterfaceTypeDef::try_from(interface_def)?)
                }
                apollo_parser::ast::Definition::UnionTypeDefinition(union_def) => {
                    union_defs.push(UnionTypeDef::try_from(union_def)?)
                }
                apollo_parser::ast::Definition::UnionTypeExtension(union_def) => {
                    union_defs.push(UnionTypeDef::try_from(union_def)?)
                }
                apollo_parser::ast::Definition::InputObjectTypeDefinition(input_object_def) => {
                    input_object_defs.push(InputObjectTypeDef::try_from(input_object_def)?)
                }
                apollo_parser::ast::Definition::InputObjectTypeExtension(input_object_def) => {
                    input_object_defs.push(InputObjectTypeDef::try_from(input_object_def)?)
                }
                apollo_parser::ast::Definition::FragmentDefinition(fragment_def) => {
                    fragment_defs.push(FragmentDef::try_from(fragment_def)?)
                }
            }
        }

        Ok(Self {
            operation_definitions: operation_defs,
            fragment_definitions: fragment_defs,
            schema_definition: schema_def,
            scalar_type_definitions: scalar_defs,
            object_type_definitions: object_defs,
            interface_type_definitions: interface_defs,
            union_type_definitions: union_defs,
            enum_type_definitions: enum_defs,
            input_object_type_definitions: input_object_defs,
            directive_definitions: directive_defs,
        })
    }
}

impl From<Document> for String {
    fn from(doc: Document) -> Self {
        apollo_encoder::Document::from(doc).to_string()
    }
}
