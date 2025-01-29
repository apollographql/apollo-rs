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
    fn type_name(&self) -> &'static str {
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

impl_resolver! {
    for SchemaWithImplementersMap<'_>:

    __typename = "__Schema";

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.schema_definition.description.as_deref()))
    }

    fn types(&self_) {
        Ok(ResolvedValue::list(self_.types.iter().map(|(name, def)| {
            ResolvedValue::object(TypeDefResolver { schema: *self_, name, def })
        })))
    }

    fn directives(&self_) {
        Ok(ResolvedValue::list(self_.directive_definitions.values().map(|def| {
            ResolvedValue::object(DirectiveResolver { schema: *self_, def })
        })))
    }

    fn queryType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.query))
    }

    fn mutationType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.mutation))
    }

    fn subscriptionType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.subscription))
    }
}

impl_resolver! {
    for TypeDefResolver<'_>:

    __typename = "__Type";

    fn kind(&self_) {
        Ok(ResolvedValue::leaf(match self_.def {
            schema::ExtendedType::Scalar(_) => "SCALAR",
            schema::ExtendedType::Object(_) => "OBJECT",
            schema::ExtendedType::Interface(_) => "INTERFACE",
            schema::ExtendedType::Union(_) => "UNION",
            schema::ExtendedType::Enum(_) => "ENUM",
            schema::ExtendedType::InputObject(_) => "INPUT_OBJECT",
        }))
    }

    fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.name))
    }

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description().map(|desc| desc.as_str())))
    }

    fn fields(&self_, args) {
        let fields = match self_.def {
            schema::ExtendedType::Object(def) => &def.fields,
            schema::ExtendedType::Interface(def) => &def.fields,
            schema::ExtendedType::Scalar(_) |
            schema::ExtendedType::Union(_) |
            schema::ExtendedType::Enum(_) |
            schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
        };
        let include_deprecated = include_deprecated(args);
        Ok(ResolvedValue::list(fields
            .values()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(FieldResolver { schema: self_.schema, def })
            })
        ))
    }

    fn interfaces(&self_) {
        let implements_interfaces = match self_.def {
            schema::ExtendedType::Object(def) => &def.implements_interfaces,
            schema::ExtendedType::Interface(def) => &def.implements_interfaces,
            schema::ExtendedType::Scalar(_) |
            schema::ExtendedType::Union(_) |
            schema::ExtendedType::Enum(_) |
            schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
        };
        Ok(ResolvedValue::list(implements_interfaces.iter().filter_map(|name| {
            self_.schema.types.get(&name.name).map(|def| {
                ResolvedValue::object(TypeDefResolver { schema: self_.schema, name, def })
            })
        })))
    }

    fn possibleTypes(&self_) {
        macro_rules! types {
            ($names: expr) => {
                Ok(ResolvedValue::list($names.filter_map(move |name| {
                    self_.schema.types.get(name).map(move |def| {
                        ResolvedValue::object(TypeDefResolver { schema: self_.schema, name, def })
                    })
                })))
            }
        }
        match self_.def {
            schema::ExtendedType::Interface(_) => types!(self_.schema.implementers_of(self_.name)),
            schema::ExtendedType::Union(def) => types!(def.members.iter().map(|c| &c.name)),
            schema::ExtendedType::Object(_) |
            schema::ExtendedType::Scalar(_) |
            schema::ExtendedType::Enum(_) |
            schema::ExtendedType::InputObject(_) => Ok(ResolvedValue::null()),
        }
    }

    fn enumValues(&self_, args) {
        let schema::ExtendedType::Enum(def) = self_.def else {
            return Ok(ResolvedValue::null());
        };
        let include_deprecated = include_deprecated(args);
        Ok(ResolvedValue::list(def
            .values
            .values()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(EnumValueResolver { schema: self_.schema, def })
            })
        ))
    }

    fn inputFields(&self_, args) {
        let schema::ExtendedType::InputObject(def) = self_.def else {
            return Ok(ResolvedValue::null());
        };
        let include_deprecated = include_deprecated(args);
        Ok(ResolvedValue::list(def
            .fields
            .values()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(InputValueResolver { schema: self_.schema, def })
            })
        ))
    }

    fn ofType() {
        Ok(ResolvedValue::null())
    }

    fn specifiedByURL(&self_) {
        let schema::ExtendedType::Scalar(def) = self_.def else {
            return Ok(ResolvedValue::null())
        };
        Ok(ResolvedValue::leaf(def
            .directives.get("specifiedBy")
            .and_then(|dir| dir.specified_argument_by_name("url"))
            .and_then(|arg| arg.as_str())
        ))
    }
}

// Only used for non-null and list types
impl_resolver! {
    for TypeResolver<'_>:

    __typename = "__Type";

    fn kind(&self_) {
        Ok(ResolvedValue::leaf(match &*self_.ty {
            schema::Type::Named(_) => unreachable!(),
            schema::Type::List(_) => "LIST",
            schema::Type::NonNullNamed(_) |
            schema::Type::NonNullList(_) => "NON_NULL",
        }))
    }

    fn ofType(&self_) {
        Ok(match &*self_.ty {
            schema::Type::Named(_) => unreachable!(),
            schema::Type::List(inner) => ty(self_.schema, inner),
            schema::Type::NonNullNamed(inner) => type_def(self_.schema, inner),
            schema::Type::NonNullList(inner) => ResolvedValue::object(Self {
                schema: self_.schema,
                ty: Cow::Owned(schema::Type::List(inner.clone()))
            }),
        })
    }

    fn name() { Ok(ResolvedValue::null()) }
    fn description() { Ok(ResolvedValue::null()) }
    fn fields() { Ok(ResolvedValue::null()) }
    fn interfaces() { Ok(ResolvedValue::null()) }
    fn possibleTypes() { Ok(ResolvedValue::null()) }
    fn enumValues() { Ok(ResolvedValue::null()) }
    fn inputFields() { Ok(ResolvedValue::null()) }
    fn specifiedByURL() { Ok(ResolvedValue::null()) }
}

impl_resolver! {
    for DirectiveResolver<'_>:

    __typename = "__Directive";

    fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_str()))
    }

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    fn args(&self_, args) {
        let include_deprecated = include_deprecated(args);
        Ok(ResolvedValue::list(self_
            .def
            .arguments
            .iter()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(InputValueResolver { schema: self_.schema, def })
            })
        ))
    }

    fn locations(&self_) {
        Ok(ResolvedValue::list(self_.def.locations.iter().map(|loc| {
            ResolvedValue::leaf(loc.name())
        })))
    }

    fn isRepeatable(&self_) {
        Ok(ResolvedValue::leaf(self_.def.repeatable))
    }
}

impl_resolver! {
    for FieldResolver<'_>:

    __typename = "__Field";

    fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_str()))
    }

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    fn args(&self_, args) {
        let include_deprecated = include_deprecated(args);
        Ok(ResolvedValue::list(self_
            .def
            .arguments
            .iter()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(InputValueResolver { schema: self_.schema, def })
            })
        ))
    }

    fn type(&self_) {
        Ok(ty(self_.schema, &self_.def.ty))
    }

    fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    fn deprecationReason(&self_) {
        Ok(deprecation_reason(&self_.schema, self_.def.directives.get("deprecated")))
    }
}

impl_resolver! {
    for EnumValueResolver<'_>:

    __typename = "__EnumValue";

    fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.value.as_str()))
    }

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    fn deprecationReason(&self_) {
        Ok(deprecation_reason(&self_.schema, self_.def.directives.get("deprecated")))
    }
}

impl_resolver! {
    for InputValueResolver<'_>:

    __typename = "__InputValue";

    fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_str()))
    }

    fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    fn type(&self_) {
        Ok(ty(self_.schema, &self_.def.ty))
    }

    fn defaultValue(&self_) {
        Ok(ResolvedValue::leaf(self_.def.default_value.as_ref().map(|val| {
            val.serialize().no_indent().to_string()
        })))
    }

    fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    fn deprecationReason(&self_) {
        Ok(deprecation_reason(&self_.schema, self_.def.directives.get("deprecated")))
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
