use crate::resolvers::ObjectValue;
use crate::resolvers::ResolveError;
use crate::resolvers::ResolveInfo;
use crate::resolvers::ResolvedValue;
use crate::response::JsonMap;
use crate::schema;
use crate::schema::ComponentName;
use crate::Node;
use std::borrow::Cow;

pub(crate) struct SchemaMetaField;

struct TypeDefResolver<'a> {
    def: &'a schema::ExtendedType,
}

/// Only used for non-null and list types. `TypeDef` is used for everything else.
pub(crate) struct TypeResolver<'a> {
    ty: Cow<'a, schema::Type>,
}

struct DirectiveResolver<'a> {
    def: &'a schema::DirectiveDefinition,
}

struct FieldResolver<'a> {
    def: &'a schema::FieldDefinition,
}

struct EnumValueResolver<'a> {
    def: &'a schema::EnumValueDefinition,
}

struct InputValueResolver<'a> {
    def: &'a schema::InputValueDefinition,
}

pub(crate) fn type_def<'a>(info: &ResolveInfo<'a>, name: &str) -> ResolvedValue<'a> {
    ResolvedValue::nullable_object(
        info.schema()
            .types
            .get(name)
            .map(|def| TypeDefResolver { def }),
    )
}

fn type_def_opt<'a>(info: &ResolveInfo<'a>, name: &Option<ComponentName>) -> ResolvedValue<'a> {
    if let Some(name) = name {
        type_def(info, name)
    } else {
        ResolvedValue::null()
    }
}

fn ty<'a>(info: &ResolveInfo<'a>, ty: &'a schema::Type) -> ResolvedValue<'a> {
    if let schema::Type::Named(name) = ty {
        type_def(info, name)
    } else {
        ResolvedValue::object(TypeResolver {
            ty: Cow::Borrowed(ty),
        })
    }
}

fn deprecation_reason<'a>(
    info: &ResolveInfo<'a>,
    opt_directive: Option<&Node<schema::Directive>>,
) -> ResolvedValue<'a> {
    ResolvedValue::leaf(
        opt_directive
            .and_then(|directive| directive.argument_by_name("reason", info.schema()).ok())
            .and_then(|arg| arg.as_str()),
    )
}

impl ObjectValue for SchemaMetaField {
    fn type_name(&self) -> &str {
        "__Schema"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        let schema_def = &info.schema().schema_definition;
        match info.field_name() {
            "description" => Ok(ResolvedValue::leaf(schema_def.description.as_deref())),
            "types" => Ok(ResolvedValue::list(
                info.schema()
                    .types
                    .values()
                    .map(|def| ResolvedValue::object(TypeDefResolver { def })),
            )),
            "directives" => Ok(ResolvedValue::list(
                info.schema()
                    .directive_definitions
                    .values()
                    .map(|def| ResolvedValue::object(DirectiveResolver { def })),
            )),
            "queryType" => Ok(type_def_opt(info, &schema_def.query)),
            "mutationType" => Ok(type_def_opt(info, &schema_def.mutation)),
            "subscriptionType" => Ok(type_def_opt(info, &schema_def.subscription)),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl ObjectValue for TypeDefResolver<'_> {
    fn type_name(&self) -> &str {
        "__Type"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        let schema = info.schema();
        macro_rules! types {
            ($names:expr) => {
                Ok(ResolvedValue::list($names.filter_map(move |name| {
                    schema
                        .types
                        .get(name.as_str())
                        .map(move |def| ResolvedValue::object(TypeDefResolver { def }))
                })))
            };
        }
        match info.field_name() {
            "kind" => Ok(ResolvedValue::leaf(match self.def {
                schema::ExtendedType::Scalar(_) => "SCALAR",
                schema::ExtendedType::Object(_) => "OBJECT",
                schema::ExtendedType::Interface(_) => "INTERFACE",
                schema::ExtendedType::Union(_) => "UNION",
                schema::ExtendedType::Enum(_) => "ENUM",
                schema::ExtendedType::InputObject(_) => "INPUT_OBJECT",
            })),
            "name" => Ok(ResolvedValue::leaf(self.def.name().as_str())),
            "description" => Ok(ResolvedValue::leaf(
                self.def.description().map(|desc| desc.as_str()),
            )),
            "fields" => {
                let fields = match self.def {
                    schema::ExtendedType::Object(def) => &def.fields,
                    schema::ExtendedType::Interface(def) => &def.fields,
                    schema::ExtendedType::Scalar(_)
                    | schema::ExtendedType::Union(_)
                    | schema::ExtendedType::Enum(_)
                    | schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
                };
                let include_deprecated = include_deprecated(info.arguments());
                Ok(ResolvedValue::list(
                    fields
                        .values()
                        .filter(move |def| {
                            include_deprecated || def.directives.get("deprecated").is_none()
                        })
                        .map(|def| ResolvedValue::object(FieldResolver { def })),
                ))
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
                types!(implements_interfaces.iter())
            }
            "possibleTypes" => match self.def {
                schema::ExtendedType::Interface(def) => {
                    types!(info
                        .implementers_map()
                        .get(&def.name)
                        .into_iter()
                        .flat_map(|implementers| &implementers.objects))
                }
                schema::ExtendedType::Union(def) => {
                    types!(def.members.iter().map(|c| &c.name))
                }
                schema::ExtendedType::Object(_)
                | schema::ExtendedType::Scalar(_)
                | schema::ExtendedType::Enum(_)
                | schema::ExtendedType::InputObject(_) => Ok(ResolvedValue::null()),
            },
            "enumValues" => {
                let schema::ExtendedType::Enum(def) = self.def else {
                    return Ok(ResolvedValue::null());
                };
                let include_deprecated = include_deprecated(info.arguments());
                Ok(ResolvedValue::list(
                    def.values
                        .values()
                        .filter(move |def| {
                            include_deprecated || def.directives.get("deprecated").is_none()
                        })
                        .map(|def| ResolvedValue::object(EnumValueResolver { def })),
                ))
            }
            "inputFields" => {
                let schema::ExtendedType::InputObject(def) = self.def else {
                    return Ok(ResolvedValue::null());
                };
                let include_deprecated = include_deprecated(info.arguments());
                Ok(ResolvedValue::list(
                    def.fields
                        .values()
                        .filter(move |def| {
                            include_deprecated || def.directives.get("deprecated").is_none()
                        })
                        .map(|def| ResolvedValue::object(InputValueResolver { def })),
                ))
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
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

/// Only used for non-null and list types
impl ObjectValue for TypeResolver<'_> {
    fn type_name(&self) -> &str {
        "__Type"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        match info.field_name() {
            "kind" => Ok(ResolvedValue::leaf(match &*self.ty {
                schema::Type::Named(_) => unreachable!(),
                schema::Type::List(_) => "LIST",
                schema::Type::NonNullNamed(_) | schema::Type::NonNullList(_) => "NON_NULL",
            })),
            "ofType" => Ok(match &*self.ty {
                schema::Type::Named(_) => unreachable!(),
                schema::Type::List(inner) => ty(info, inner),
                schema::Type::NonNullNamed(inner) => type_def(info, inner),
                schema::Type::NonNullList(inner) => ResolvedValue::object(Self {
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
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl ObjectValue for DirectiveResolver<'_> {
    fn type_name(&self) -> &str {
        "__Directive"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        match info.field_name() {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "args" => {
                let include_deprecated = include_deprecated(info.arguments());
                Ok(ResolvedValue::list(
                    self.def
                        .arguments
                        .iter()
                        .filter(move |def| {
                            include_deprecated || def.directives.get("deprecated").is_none()
                        })
                        .map(|def| ResolvedValue::object(InputValueResolver { def })),
                ))
            }
            "locations" => Ok(ResolvedValue::list(
                self.def
                    .locations
                    .iter()
                    .map(|loc| ResolvedValue::leaf(loc.name())),
            )),
            "isRepeatable" => Ok(ResolvedValue::leaf(self.def.repeatable)),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl ObjectValue for FieldResolver<'_> {
    fn type_name(&self) -> &str {
        "__Field"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        match info.field_name() {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "args" => {
                let include_deprecated = include_deprecated(info.arguments());
                Ok(ResolvedValue::list(
                    self.def
                        .arguments
                        .iter()
                        .filter(move |def| {
                            include_deprecated || def.directives.get("deprecated").is_none()
                        })
                        .map(|def| ResolvedValue::object(InputValueResolver { def })),
                ))
            }
            "type" => Ok(ty(info, &self.def.ty)),
            "isDeprecated" => Ok(ResolvedValue::leaf(
                self.def.directives.get("deprecated").is_some(),
            )),
            "deprecationReason" => Ok(deprecation_reason(
                info,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl ObjectValue for EnumValueResolver<'_> {
    fn type_name(&self) -> &str {
        "__EnumValue"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        match info.field_name() {
            "name" => Ok(ResolvedValue::leaf(self.def.value.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "isDeprecated" => Ok(ResolvedValue::leaf(
                self.def.directives.get("deprecated").is_some(),
            )),
            "deprecationReason" => Ok(deprecation_reason(
                info,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(self.unknown_field_error(info)),
        }
    }
}

impl ObjectValue for InputValueResolver<'_> {
    fn type_name(&self) -> &str {
        "__InputValue"
    }

    fn resolve_field<'a>(
        &'a self,
        info: &ResolveInfo<'a>,
    ) -> Result<ResolvedValue<'a>, ResolveError> {
        match info.field_name() {
            "name" => Ok(ResolvedValue::leaf(self.def.name.as_str())),
            "description" => Ok(ResolvedValue::leaf(self.def.description.as_deref())),
            "type" => Ok(ty(info, &self.def.ty)),
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
                info,
                self.def.directives.get("deprecated"),
            )),
            _ => Err(self.unknown_field_error(info)),
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
