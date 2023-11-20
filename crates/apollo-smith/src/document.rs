use crate::{
    directive::DirectiveDef, enum_::EnumTypeDef, fragment::FragmentDef,
    input_object::InputObjectTypeDef, interface::InterfaceTypeDef, object::ObjectTypeDef,
    operation::OperationDef, scalar::ScalarTypeDef, schema::SchemaDef, union::UnionTypeDef,
};
use apollo_compiler::ast;

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
#[derive(Debug, Clone)]
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

impl From<Document> for ast::Document {
    fn from(doc: Document) -> Self {
        fn extend(
            new_doc: &mut ast::Document,
            items: impl IntoIterator<Item = impl Into<ast::Definition>>,
        ) {
            new_doc
                .definitions
                .extend(items.into_iter().map(|x| x.into()));
        }

        let Document {
            operation_definitions,
            fragment_definitions,
            schema_definition,
            scalar_type_definitions,
            object_type_definitions,
            interface_type_definitions,
            union_type_definitions,
            enum_type_definitions,
            input_object_type_definitions,
            directive_definitions,
        } = doc;
        let mut new_doc = Self::new();
        extend(&mut new_doc, operation_definitions);
        extend(&mut new_doc, fragment_definitions);
        extend(&mut new_doc, schema_definition);
        extend(&mut new_doc, scalar_type_definitions);
        extend(&mut new_doc, object_type_definitions);
        extend(&mut new_doc, interface_type_definitions);
        extend(&mut new_doc, union_type_definitions);
        extend(&mut new_doc, enum_type_definitions);
        extend(&mut new_doc, input_object_type_definitions);
        extend(&mut new_doc, directive_definitions);
        new_doc
    }
}

impl TryFrom<apollo_parser::cst::Document> for Document {
    type Error = crate::FromError;

    fn try_from(doc: apollo_parser::cst::Document) -> Result<Self, Self::Error> {
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
                apollo_parser::cst::Definition::EnumTypeDefinition(enum_def) => {
                    enum_defs.push(EnumTypeDef::try_from(enum_def)?);
                }
                apollo_parser::cst::Definition::EnumTypeExtension(enum_def) => {
                    enum_defs.push(EnumTypeDef::try_from(enum_def)?);
                }
                apollo_parser::cst::Definition::ObjectTypeDefinition(obj_def) => {
                    object_defs.push(ObjectTypeDef::try_from(obj_def)?);
                }
                apollo_parser::cst::Definition::ObjectTypeExtension(obj_def) => {
                    object_defs.push(ObjectTypeDef::try_from(obj_def)?);
                }
                apollo_parser::cst::Definition::SchemaDefinition(schema_definition) => {
                    schema_def = Some(SchemaDef::try_from(schema_definition)?);
                }
                apollo_parser::cst::Definition::SchemaExtension(schema_definition) => {
                    schema_def = Some(SchemaDef::try_from(schema_definition)?);
                }
                apollo_parser::cst::Definition::DirectiveDefinition(dir_def) => {
                    directive_defs.push(DirectiveDef::try_from(dir_def)?);
                }
                apollo_parser::cst::Definition::ScalarTypeDefinition(scalar_def) => {
                    scalar_defs.push(ScalarTypeDef::try_from(scalar_def)?)
                }
                apollo_parser::cst::Definition::ScalarTypeExtension(scalar_def) => {
                    scalar_defs.push(ScalarTypeDef::try_from(scalar_def)?)
                }
                apollo_parser::cst::Definition::OperationDefinition(operation_def) => {
                    operation_defs.push(OperationDef::try_from(operation_def)?)
                }
                apollo_parser::cst::Definition::InterfaceTypeDefinition(interface_def) => {
                    interface_defs.push(InterfaceTypeDef::try_from(interface_def)?)
                }
                apollo_parser::cst::Definition::InterfaceTypeExtension(interface_def) => {
                    interface_defs.push(InterfaceTypeDef::try_from(interface_def)?)
                }
                apollo_parser::cst::Definition::UnionTypeDefinition(union_def) => {
                    union_defs.push(UnionTypeDef::try_from(union_def)?)
                }
                apollo_parser::cst::Definition::UnionTypeExtension(union_def) => {
                    union_defs.push(UnionTypeDef::try_from(union_def)?)
                }
                apollo_parser::cst::Definition::InputObjectTypeDefinition(input_object_def) => {
                    input_object_defs.push(InputObjectTypeDef::try_from(input_object_def)?)
                }
                apollo_parser::cst::Definition::InputObjectTypeExtension(input_object_def) => {
                    input_object_defs.push(InputObjectTypeDef::try_from(input_object_def)?)
                }
                apollo_parser::cst::Definition::FragmentDefinition(fragment_def) => {
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
        ast::Document::from(doc).to_string()
    }
}
