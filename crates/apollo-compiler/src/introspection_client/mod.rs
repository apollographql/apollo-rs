use crate::ast;
use crate::execution::GraphQLError;
use crate::execution::JsonMap;
use crate::name;
use crate::schema;
use crate::schema::Component;
use crate::schema::DirectiveDefinition;
use crate::schema::EnumType;
use crate::schema::EnumValueDefinition;
use crate::schema::ExtendedType;
use crate::schema::FieldDefinition;
use crate::schema::InputObjectType;
use crate::schema::InterfaceType;
use crate::schema::Name;
use crate::schema::ObjectType;
use crate::schema::ScalarType;
use crate::schema::UnionType;
use crate::Node;
use crate::NodeStr;
use crate::Schema;
use serde::de;
use serde::ser::SerializeMap;
use serde::Deserialize;
use serde_json_bytes::serde_json;

/// The introspection query (in GraphQL syntax) to send to a server
/// in order to get its full schema.
pub const QUERY: &str = include_str!("query.graphql");

/// The JSON serialization of the GraphQL request to send to a server
/// in order to get its full schema.
pub fn request() -> String {
    struct Request;

    impl serde::Serialize for Request {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("query", QUERY)?;
            map.end()
        }
    }

    serde_json::to_string(&Request).unwrap()
}

/// Deserialize this with something like [`serde_json::from_str`] or [`serde_json::from_slice`].
pub struct Response {
    pub schema: Option<Schema>,
    pub errors: Vec<GraphQLError>,
    pub extensions: JsonMap,
}

impl<'de> Deserialize<'de> for Response {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct ResponseJson {
            #[serde(default)]
            pub data: Option<Data>,
            #[serde(default)]
            pub errors: Vec<GraphQLError>,
            #[serde(default)]
            pub extensions: JsonMap,
        }

        #[derive(serde::Deserialize)]
        struct Data {
            pub __schema: __Schema,
        }

        let response = ResponseJson::deserialize(deserializer)?;
        Ok(Self {
            schema: response.data.map(|data| data.__schema.0),
            errors: response.errors,
            extensions: response.extensions,
        })
    }
}

struct __Schema(Schema);
struct Types<'a>(&'a mut Schema);
struct Directives<'a>(&'a mut Schema);
struct TypeRef(ast::Type);
struct TypeDef(ExtendedType);

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct FieldDef {
    name: Name,
    description: Option<NodeStr>,
    args: Vec<InputValue>,
    #[serde(rename = "type")]
    ty: TypeRef,
    isDeprecated: bool,
    deprecationReason: Option<NodeStr>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct InputValue {
    name: Name,
    description: Option<NodeStr>,
    #[serde(rename = "type")]
    ty: TypeRef,
    defaultValue: Option<String>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct EnumValue {
    name: Name,
    description: Option<NodeStr>,
    isDeprecated: bool,
    deprecationReason: Option<NodeStr>,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct DirectiveDef {
    name: Name,
    description: Option<NodeStr>,
    isRepeatable: bool,
    locations: Vec<ast::DirectiveLocation>,
    args: Vec<InputValue>,
}

#[derive(Deserialize)]
struct NamedType {
    name: Name,
}

#[derive(Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum __TypeKind {
    SCALAR,
    OBJECT,
    INTERFACE,
    UNION,
    ENUM,
    INPUT_OBJECT,
    LIST,
    NON_NULL,
}

#[derive(Deserialize)]
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
enum __TypeDefKind {
    SCALAR,
    OBJECT,
    INTERFACE,
    UNION,
    ENUM,
    INPUT_OBJECT,
}

impl<'de> Deserialize<'de> for __Schema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // __schema {
        //   queryType { name }
        //   mutationType { name }
        //   subscriptionType { name }
        //   types { ...FullType }
        //   directives {
        //     name
        //     description
        //     locations
        //     args { ...InputValue }
        //   }
        // }

        #[derive(Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field {
            queryType,
            mutationType,
            subscriptionType,
            types,
            directives,
        }

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = __Schema;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a __Schema field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut schema = Schema::new();
                while let Some(field) = map.next_key()? {
                    match field {
                        Field::queryType | Field::mutationType | Field::subscriptionType => {
                            let schema_def = schema.schema_definition.make_mut();
                            let new = map
                                .next_value::<Option<NamedType>>()?
                                .map(|n| n.name.into());
                            match field {
                                Field::queryType => schema_def.query = new,
                                Field::mutationType => schema_def.mutation = new,
                                Field::subscriptionType => schema_def.subscription = new,
                                _ => unreachable!(),
                            }
                        }
                        Field::types => map.next_value_seed(Types(&mut schema))?,
                        Field::directives => map.next_value_seed(Directives(&mut schema))?,
                    }
                }
                Ok(__Schema(schema))
            }
        }

        deserializer.deserialize_struct(
            "__Schema",
            &[
                "queryType",
                "mutationType",
                "subscriptionType",
                "types",
                "directives",
            ],
            Visitor,
        )
    }
}

impl<'de> de::DeserializeSeed<'de> for Types<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a>(&'a mut Schema);

        impl<'de> de::Visitor<'de> for Visitor<'_> {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a list of types")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                while let Some(def) = seq.next_element::<TypeDef>()? {
                    self.0.types.insert(def.0.name().clone(), def.0);
                }
                Ok(())
            }
        }

        deserializer.deserialize_seq(Visitor(self.0))
    }
}

impl<'de> de::DeserializeSeed<'de> for Directives<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<'a>(&'a mut Schema);

        impl<'de> de::Visitor<'de> for Visitor<'_> {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a list of types")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                while let Some(def) = seq.next_element::<DirectiveDef>()? {
                    let def = DirectiveDefinition {
                        description: def.description,
                        name: def.name,
                        arguments: def.args.into_iter().map(Into::into).collect(),
                        repeatable: def.isRepeatable,
                        locations: def.locations,
                    };
                    self.0
                        .directive_definitions
                        .insert(def.name.clone(), def.into());
                }
                Ok(())
            }
        }

        deserializer.deserialize_seq(Visitor(self.0))
    }
}

impl<'de> Deserialize<'de> for TypeDef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // fragment FullType on __Type {
        //   kind
        //   name
        //   description
        //   fields(includeDeprecated: true) {
        //     name
        //     description
        //     args { ...InputValue }
        //     type { ...TypeRef }
        //     isDeprecated
        //     deprecationReason
        //   }
        //   inputFields { ...InputValue }
        //   interfaces { ...TypeRef }
        //   enumValues(includeDeprecated: true) {
        //     name
        //     description
        //     isDeprecated
        //     deprecationReason
        //   }
        //   possibleTypes { ...TypeRef }
        //   specifiedByURL
        // }

        #[derive(Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field {
            kind,
            name,
            description,
            fields,
            inputFields,
            interfaces,
            enumValues,
            possibleTypes,
            specifiedByURL,
        }

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = TypeDef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a __Type field")
            }

            #[allow(non_snake_case)]
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut description = None;
                let mut kind = None;
                let mut name = None;
                let mut fields = None;
                let mut inputFields = None;
                let mut interfaces = None;
                let mut enumValues = None;
                let mut possibleTypes = None;
                let mut specifiedByURL = None;
                while let Some(field) = map.next_key()? {
                    match field {
                        Field::description => description = Some(map.next_value()?),
                        Field::kind => kind = Some(map.next_value()?),
                        Field::name => name = Some(map.next_value()?),
                        Field::fields => fields = Some(map.next_value()?),
                        Field::inputFields => inputFields = Some(map.next_value()?),
                        Field::interfaces => interfaces = Some(map.next_value()?),
                        Field::enumValues => enumValues = Some(map.next_value()?),
                        Field::possibleTypes => possibleTypes = Some(map.next_value()?),
                        Field::specifiedByURL => specifiedByURL = Some(map.next_value()?),
                    }
                }
                let description =
                    description.ok_or_else(|| de::Error::missing_field("description"))?;
                let kind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let fields: Option<Vec<FieldDef>> =
                    fields.ok_or_else(|| de::Error::missing_field("fields"))?;
                let inputFields: Option<Vec<InputValue>> =
                    inputFields.ok_or_else(|| de::Error::missing_field("inputFields"))?;
                let interfaces: Option<Vec<NamedType>> =
                    interfaces.ok_or_else(|| de::Error::missing_field("interfaces"))?;
                let enumValues: Option<Vec<EnumValue>> =
                    enumValues.ok_or_else(|| de::Error::missing_field("enumValues"))?;
                let possibleTypes: Option<Vec<NamedType>> =
                    possibleTypes.ok_or_else(|| de::Error::missing_field("possibleTypes"))?;
                let specifiedByURL: Option<NodeStr> =
                    specifiedByURL.ok_or_else(|| de::Error::missing_field("specifiedByURL"))?;

                let fields = fields
                    .into_iter()
                    .flatten()
                    .map(|def| {
                        (
                            def.name.clone(),
                            Component::new(FieldDefinition {
                                description: def.description,
                                name: def.name,
                                arguments: def.args.into_iter().map(Into::into).collect(),
                                ty: def.ty.0,
                                directives: maybe_deprecated(
                                    def.isDeprecated,
                                    def.deprecationReason,
                                ),
                            }),
                        )
                    })
                    .collect();
                let implements_interfaces = interfaces
                    .into_iter()
                    .flatten()
                    .map(|interface| interface.name.into())
                    .collect();
                Ok(TypeDef(match kind {
                    __TypeDefKind::SCALAR => ScalarType {
                        description,
                        name,
                        directives: {
                            let mut directives = schema::DirectiveList::default();
                            if specifiedByURL.is_some() {
                                directives.push(
                                    opt_string_arg_directive(
                                        name!("specifiedBy"),
                                        name!("url"),
                                        specifiedByURL,
                                    )
                                    .into(),
                                )
                            }
                            directives
                        },
                    }
                    .into(),
                    __TypeDefKind::OBJECT => ObjectType {
                        description,
                        name,
                        implements_interfaces,
                        directives: Default::default(),
                        fields,
                    }
                    .into(),
                    __TypeDefKind::INTERFACE => InterfaceType {
                        description,
                        name,
                        implements_interfaces,
                        directives: Default::default(),
                        fields,
                    }
                    .into(),
                    __TypeDefKind::UNION => UnionType {
                        description,
                        name,
                        directives: Default::default(),
                        members: possibleTypes
                            .into_iter()
                            .flatten()
                            .map(|ty| ty.name.into())
                            .collect(),
                    }
                    .into(),
                    __TypeDefKind::ENUM => EnumType {
                        description,
                        name,
                        directives: Default::default(),
                        values: enumValues
                            .into_iter()
                            .flatten()
                            .map(|v| {
                                (
                                    v.name.clone(),
                                    EnumValueDefinition {
                                        description: v.description,
                                        value: v.name,
                                        directives: maybe_deprecated(
                                            v.isDeprecated,
                                            v.deprecationReason,
                                        ),
                                    }
                                    .into(),
                                )
                            })
                            .collect(),
                    }
                    .into(),
                    __TypeDefKind::INPUT_OBJECT => InputObjectType {
                        description,
                        name,
                        directives: Default::default(),
                        fields: inputFields
                            .into_iter()
                            .flatten()
                            .map(|def| {
                                (
                                    def.name.clone(),
                                    Node::<ast::InputValueDefinition>::from(def).into(),
                                )
                            })
                            .collect(),
                    }
                    .into(),
                }))
            }
        }

        deserializer.deserialize_struct(
            "__Type",
            &[
                "kind",
                "name",
                "description",
                "fields",
                "inputFields",
                "interfaces",
                "enumValues",
                "possibleTypes",
            ],
            Visitor,
        )
    }
}

impl<'de> Deserialize<'de> for TypeRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // fragment TypeRef on __Type {
        //   kind
        //   name
        //   ofType { ...TypeRef }
        // }

        #[derive(Deserialize)]
        #[allow(non_camel_case_types)]
        enum Field {
            kind,
            name,
            ofType,
        }

        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = TypeRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a TypeRef field")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: de::MapAccess<'de>,
            {
                let mut kind = None;
                let mut name = None;
                let mut of_type = None;
                while let Some(field) = map.next_key()? {
                    match field {
                        Field::kind => kind = Some(map.next_value()?),
                        Field::name => name = Some(map.next_value()?),
                        Field::ofType => of_type = Some(map.next_value()?),
                    }
                }
                let kind: __TypeKind = kind.ok_or_else(|| de::Error::missing_field("kind"))?;
                let name: Option<Name> = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let of_type: Option<TypeRef> =
                    of_type.ok_or_else(|| de::Error::missing_field("ofType"))?;
                Ok(TypeRef(match kind {
                    __TypeKind::SCALAR
                    | __TypeKind::OBJECT
                    | __TypeKind::INTERFACE
                    | __TypeKind::UNION
                    | __TypeKind::ENUM
                    | __TypeKind::INPUT_OBJECT => ast::Type::Named(
                        name.ok_or_else(|| de::Error::custom("missing type name"))?,
                    ),
                    __TypeKind::LIST => of_type
                        .ok_or_else(|| de::Error::custom("invalid {kind: LIST, ofType: null}"))?
                        .0
                        .list(),
                    __TypeKind::NON_NULL => of_type
                        .ok_or_else(|| de::Error::custom("invalid {kind: NON_NULL, ofType: null}"))?
                        .0
                        .non_null(),
                }))
            }
        }

        deserializer.deserialize_struct("TypeRef", &["kind", "name", "ofType"], Visitor)
    }
}

impl From<InputValue> for Node<ast::InputValueDefinition> {
    fn from(value: InputValue) -> Self {
        Node::new(ast::InputValueDefinition {
            description: value.description,
            name: value.name,
            ty: value.ty.0.into(),
            default_value: value.defaultValue.map(parse_value),
            directives: Default::default(),
        })
    }
}

fn parse_value(graphql_value: String) -> Node<ast::Value> {
    // TODO
    ast::Value::Null.into()
}

fn maybe_deprecated(
    is_deprecated: bool,
    deprecation_reason: Option<NodeStr>,
) -> ast::DirectiveList {
    let mut directives = ast::DirectiveList::default();
    if is_deprecated {
        directives.push(
            opt_string_arg_directive(name!("deprecated"), name!("reason"), deprecation_reason)
                .into(),
        )
    }
    directives
}

fn opt_string_arg_directive(
    directive_name: Name,
    arg_name: Name,
    arg_value: Option<NodeStr>,
) -> schema::Directive {
    ast::Directive {
        name: directive_name,
        arguments: arg_value
            .map(move |value| {
                ast::Argument {
                    name: arg_name,
                    value: ast::Value::String(value).into(),
                }
                .into()
            })
            .into_iter()
            .collect(),
    }
}
