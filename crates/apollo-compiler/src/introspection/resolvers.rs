use crate::collections::HashMap;
use crate::execution::resolver::ResolvedValue;
use crate::execution::resolver::Resolver;
use crate::execution::resolver::ResolverError;
use crate::response::JsonMap;
use crate::schema;
use crate::schema::Implementers;
use crate::schema::Name;
use crate::Node;
use crate::Schema;
use std::borrow::Cow;

#[derive(Clone, Copy)]
pub(crate) struct SchemaWithImplementersMap<'a> {
    pub(crate) schema: &'a Schema,
    pub(crate) implementers_map: &'a HashMap<Name, Implementers>,
}

impl<'a> SchemaWithImplementersMap<'a> {
    fn implementers_of(&self, interface_name: &str) -> impl Iterator<Item = &'a Name> {
        self.implementers_map
            .get(interface_name)
            .into_iter()
            .flat_map(|implementers| &implementers.objects)
    }
}

impl<'a> std::ops::Deref for SchemaWithImplementersMap<'a> {
    type Target = &'a Schema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

pub(crate) struct IntrospectionRootResolver<'a>(pub(crate) SchemaWithImplementersMap<'a>);

struct TypeDefResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    name: &'a str,
    def: &'a schema::ExtendedType,
}

/// Only used for non-null and list types. `TypeDef` is used for everything else.
pub(crate) struct TypeResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    ty: Cow<'a, schema::Type>,
}

struct DirectiveResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    def: &'a schema::DirectiveDefinition,
}

struct FieldResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    def: &'a schema::FieldDefinition,
}

struct EnumValueResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    def: &'a schema::EnumValueDefinition,
}

struct InputValueResolver<'a> {
    schema: SchemaWithImplementersMap<'a>,
    def: &'a schema::InputValueDefinition,
}

fn type_def(schema: SchemaWithImplementersMap<'_>, name: impl AsRef<str>) -> ResolvedValue<'_> {
    ResolvedValue::opt_object(
        schema
            .types
            .get_key_value(name.as_ref())
            .map(|(name, def)| TypeDefResolver { schema, name, def }),
    )
}

fn type_def_opt<'a>(
    schema: SchemaWithImplementersMap<'a>,
    name: &Option<impl AsRef<str>>,
) -> ResolvedValue<'a> {
    if let Some(name) = name.as_ref() {
        type_def(schema, name)
    } else {
        ResolvedValue::null()
    }
}

fn ty<'a>(schema: SchemaWithImplementersMap<'a>, ty: &'a schema::Type) -> ResolvedValue<'a> {
    if let schema::Type::Named(name) = ty {
        type_def(schema, name)
    } else {
        ResolvedValue::object(TypeResolver {
            schema,
            ty: Cow::Borrowed(ty),
        })
    }
}

fn deprecation_reason<'a>(
    schema: &SchemaWithImplementersMap<'_>,
    opt_directive: Option<&Node<schema::Directive>>,
) -> ResolvedValue<'a> {
    ResolvedValue::leaf(
        opt_directive
            .and_then(|directive| directive.argument_by_name("reason", schema.schema).ok())
            .and_then(|arg| arg.as_str()),
    )
}

impl Resolver for IntrospectionRootResolver<'_> {
    fn type_name(&self) -> &str {
        unreachable!()
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "__schema" => Ok(ResolvedValue::object(self.0)),
            "__type" => {
                let name = arguments["name"].as_str().unwrap();
                Ok(type_def(self.0, name))
            }
            // "__typename" is handled in `execute_selection_set` without calling here.
            // Other fields are skipped in `skip_field` below.
            _ => unreachable!(),
        }
    }

    fn skip_field(&self, field_name: &str) -> bool {
        !matches!(field_name, "__schema" | "__type" | "__typename")
    }
}

impl Resolver for SchemaWithImplementersMap<'_> {
    fn type_name(&self) -> &str {
        "__Schema"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        _arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "description" => Ok(ResolvedValue::leaf(
                self.schema_definition.description.as_deref(),
            )),
            "types" => Ok(ResolvedValue::list(self.types.iter().map(|(name, def)| {
                ResolvedValue::object(TypeDefResolver {
                    schema: *self,
                    name,
                    def,
                })
            }))),
            "directives" => Ok(ResolvedValue::list(
                self.directive_definitions
                    .values()
                    .map(|def| ResolvedValue::object(DirectiveResolver { schema: *self, def })),
            )),
            "queryType" => Ok(type_def_opt(*self, &self.schema_definition.query)),
            "mutationType" => Ok(type_def_opt(*self, &self.schema_definition.mutation)),
            "subscriptionType" => Ok(type_def_opt(*self, &self.schema_definition.subscription)),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

impl Resolver for TypeDefResolver<'_> {
    fn type_name(&self) -> &str {
        "__Type"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "kind" => Ok(ResolvedValue::leaf(match self.def {
                schema::ExtendedType::Scalar(_) => "SCALAR",
                schema::ExtendedType::Object(_) => "OBJECT",
                schema::ExtendedType::Interface(_) => "INTERFACE",
                schema::ExtendedType::Union(_) => "UNION",
                schema::ExtendedType::Enum(_) => "ENUM",
                schema::ExtendedType::InputObject(_) => "INPUT_OBJECT",
            })),
            "name" => Ok(ResolvedValue::leaf(self.name)),
            "description" => Ok(ResolvedValue::leaf(
                self.def.description().map(|desc| desc.as_str()),
            )),
            "fields" => {
                let args = arguments;
                {
                    let fields = match self.def {
                        schema::ExtendedType::Object(def) => &def.fields,
                        schema::ExtendedType::Interface(def) => &def.fields,
                        schema::ExtendedType::Scalar(_)
                        | schema::ExtendedType::Union(_)
                        | schema::ExtendedType::Enum(_)
                        | schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
                    };
                    let include_deprecated = include_deprecated(args);
                    Ok(ResolvedValue::list(
                        fields
                            .values()
                            .filter(move |def| {
                                include_deprecated || def.directives.get("deprecated").is_none()
                            })
                            .map(|def| {
                                ResolvedValue::object(FieldResolver {
                                    schema: self.schema,
                                    def,
                                })
                            }),
                    ))
                }
            }
            "interfaces" => {
                let implements_interfaces = match self.def {
                    schema::ExtendedType::Object(def) => &def.implements_interfaces,
                    schema::ExtendedType::Interface(def) => &def.implements_interfaces,
                    schema::ExtendedType::Scalar(_)
                    | schema::ExtendedType::Union(_)
                    | schema::ExtendedType::Enum(_)
                    | schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
                };
                Ok(ResolvedValue::list(
                    implements_interfaces.iter().filter_map(|name| {
                        self.schema.types.get(&name.name).map(|def| {
                            ResolvedValue::object(TypeDefResolver {
                                schema: self.schema,
                                name,
                                def,
                            })
                        })
                    }),
                ))
            }
            "possibleTypes" => {
                macro_rules! types {
                    ($names:expr) => {
                        Ok(ResolvedValue::list($names.filter_map(move |name| {
                            self.schema.types.get(name).map(move |def| {
                                ResolvedValue::object(TypeDefResolver {
                                    schema: self.schema,
                                    name,
                                    def,
                                })
                            })
                        })))
                    };
                }
                match self.def {
                    schema::ExtendedType::Interface(_) => {
                        types!(self.schema.implementers_of(self.name))
                    }
                    schema::ExtendedType::Union(def) => {
                        types!(def.members.iter().map(|c| &c.name))
                    }
                    schema::ExtendedType::Object(_)
                    | schema::ExtendedType::Scalar(_)
                    | schema::ExtendedType::Enum(_)
                    | schema::ExtendedType::InputObject(_) => Ok(ResolvedValue::null()),
                }
            }
            "enumValues" => {
                let args = arguments;
                {
                    let schema::ExtendedType::Enum(def) = self.def else {
                        return Ok(ResolvedValue::null());
                    };
                    let include_deprecated = include_deprecated(args);
                    Ok(ResolvedValue::list(
                        def.values
                            .values()
                            .filter(move |def| {
                                include_deprecated || def.directives.get("deprecated").is_none()
                            })
                            .map(|def| {
                                ResolvedValue::object(EnumValueResolver {
                                    schema: self.schema,
                                    def,
                                })
                            }),
                    ))
                }
            }
            "inputFields" => {
                let args = arguments;
                {
                    let schema::ExtendedType::InputObject(def) = self.def else {
                        return Ok(ResolvedValue::null());
                    };
                    let include_deprecated = include_deprecated(args);
                    Ok(ResolvedValue::list(
                        def.fields
                            .values()
                            .filter(move |def| {
                                include_deprecated || def.directives.get("deprecated").is_none()
                            })
                            .map(|def| {
                                ResolvedValue::object(InputValueResolver {
                                    schema: self.schema,
                                    def,
                                })
                            }),
                    ))
                }
            }
            "ofType" => Ok(ResolvedValue::null()),
            "specifiedByURL" => {
                let schema::ExtendedType::Scalar(def) = self.def else {
                    return Ok(ResolvedValue::null());
                };
                Ok(ResolvedValue::leaf(
                    def.directives
                        .get("specifiedBy")
                        .and_then(|dir| dir.specified_argument_by_name("url"))
                        .and_then(|arg| arg.as_str()),
                ))
            }
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

/// Only used for non-null and list types
impl Resolver for TypeResolver<'_> {
    fn type_name(&self) -> &str {
        "__Type"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        _arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "kind" => Ok(ResolvedValue::leaf(match &*self.ty {
                schema::Type::Named(_) => unreachable!(),
                schema::Type::List(_) => "LIST",
                schema::Type::NonNullNamed(_) | schema::Type::NonNullList(_) => "NON_NULL",
            })),
            "ofType" => Ok(match &*self.ty {
                schema::Type::Named(_) => unreachable!(),
                schema::Type::List(inner) => ty(self.schema, inner),
                schema::Type::NonNullNamed(inner) => type_def(self.schema, inner),
                schema::Type::NonNullList(inner) => ResolvedValue::object(Self {
                    schema: self.schema,
                    ty: Cow::Owned(schema::Type::List(inner.clone())),
                }),
            }),
            "name" => Ok(ResolvedValue::null()),
            "description" => Ok(ResolvedValue::null()),
            "fields" => Ok(ResolvedValue::null()),
            "interfaces" => Ok(ResolvedValue::null()),
            "possibleTypes" => Ok(ResolvedValue::null()),
            "enumValues" => Ok(ResolvedValue::null()),
            "inputFields" => Ok(ResolvedValue::null()),
            "specifiedByURL" => Ok(ResolvedValue::null()),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

impl Resolver for DirectiveResolver<'_> {
    fn type_name(&self) -> &str {
        "__Directive"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "args" => {
                let args = arguments;
                {
                    let include_deprecated = include_deprecated(args);
                    Ok(ResolvedValue::list(
                        self.def
                            .arguments
                            .iter()
                            .filter(move |def| {
                                include_deprecated || def.directives.get("deprecated").is_none()
                            })
                            .map(|def| {
                                ResolvedValue::object(InputValueResolver {
                                    schema: self.schema,
                                    def,
                                })
                            }),
                    ))
                }
            }
            "locations" => Ok(ResolvedValue::list(
                self.def
                    .locations
                    .iter()
                    .map(|loc| ResolvedValue::leaf(loc.name())),
            )),
            "isRepeatable" => Ok(ResolvedValue::leaf(self.def.repeatable)),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

impl Resolver for FieldResolver<'_> {
    fn type_name(&self) -> &str {
        "__Field"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "args" => {
                let args = arguments;
                {
                    let include_deprecated = include_deprecated(args);
                    Ok(ResolvedValue::list(
                        self.def
                            .arguments
                            .iter()
                            .filter(move |def| {
                                include_deprecated || def.directives.get("deprecated").is_none()
                            })
                            .map(|def| {
                                ResolvedValue::object(InputValueResolver {
                                    schema: self.schema,
                                    def,
                                })
                            }),
                    ))
                }
            }
            "type" => Ok(ty(self.schema, &self.def.ty)),
            "isDeprecated" => Ok(ResolvedValue::leaf(
                self.def.directives.get("deprecated").is_some(),
            )),
            "deprecationReason" => Ok(deprecation_reason(
                &self.schema,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

impl Resolver for EnumValueResolver<'_> {
    fn type_name(&self) -> &str {
        "__EnumValue"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        _arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "name" => Ok(ResolvedValue::leaf(self.def.value.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "isDeprecated" => Ok(ResolvedValue::leaf(
                self.def.directives.get("deprecated").is_some(),
            )),
            "deprecationReason" => Ok(deprecation_reason(
                &self.schema,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

impl Resolver for InputValueResolver<'_> {
    fn type_name(&self) -> &str {
        "__InputValue"
    }

    fn resolve_field<'a>(
        &'a self,
        field_name: &'a str,
        _arguments: &'a JsonMap,
    ) -> Result<ResolvedValue<'a>, ResolverError> {
        match field_name {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "type" => Ok(ty(self.schema, &self.def.ty)),
            "defaultValue" => Ok(ResolvedValue::leaf(
                self.def
                    .default_value
                    .as_ref()
                    .map(|val| val.serialize().no_indent().to_string()),
            )),
            "isDeprecated" => Ok(ResolvedValue::leaf(
                self.def.directives.get("deprecated").is_some(),
            )),
            "deprecationReason" => Ok(deprecation_reason(
                &self.schema,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(ResolverError::unknown_field(field_name, self)),
        }
    }
}

/// Although it should be non-null, the `includeDeprecated: Boolean = false` argument is nullable
fn include_deprecated(args: &JsonMap) -> bool {
    match &args["includeDeprecated"] {
        serde_json_bytes::Value::Bool(b) => *b,
        serde_json_bytes::Value::Null => false,
        _ => unreachable!(),
    }
}
