use crate::executable::filtering::FilteredDocumentBuilder;
use crate::executable::Field;
use crate::executable::OperationType;
use crate::execution::engine::execute_query_or_mutation;
use crate::execution::resolver::ResolvedValue;
use crate::execution::JsonMap;
use crate::execution::RequestError;
use crate::execution::Response;
use crate::schema;
use crate::schema::Name;
use crate::validation::Valid;
use crate::ExecutableDocument;
use crate::Node;
use crate::Schema;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

/// Execution for [schema introspection]
///
/// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
#[derive(Debug)]
pub enum SchemaIntrospection {
    /// The selected operation does *not* use [schema introspection] fields.
    /// It should be executed unchanged.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    None,

    /// The selected operation *only* uses [schema introspection] fields.
    /// This provides the full response, there is nothing else to execute.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    Only(Response),

    /// The selected operation uses *both* [schema introspection] fields and other fields.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    Both {
        /// The response for schema introspection parts of the original operation.
        introspection_response: Response,

        /// Contains exactly one operation with schema introspection fields removed,
        /// and the fragment definitions it needs.
        /// The operation definition name is preserved,
        /// so either `None` or the original `Option<&str>` name request can be passed
        /// to [`ExecutableDocument::get_operation`] to obtain the one operation.
        ///
        /// This operation should be executed separately,
        /// and its response merged with `introspection_response` using [`Response::merge`].
        filtered_operation: Valid<ExecutableDocument>,
    },
}

impl SchemaIntrospection {
    /// Execute the [schema introspection] parts of an operation
    /// and return the rest to be executed separately.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    pub fn execute(
        schema: &Valid<Schema>,
        document: &Valid<ExecutableDocument>,
        operation_name: Option<&str>,
        variable_values: &Valid<JsonMap>,
    ) -> Result<Self, RequestError> {
        let operation = document.get_operation(operation_name)?;
        if operation.operation_type != OperationType::Query {
            return Ok(Self::None);
        }

        fn is_schema_introspection_meta_field(field: &Node<Field>) -> bool {
            field.name == "__schema" || field.name == "__type"
        }

        let Some(introspection_document) =
            FilteredDocumentBuilder::single_operation(schema, document, operation, |sel| {
                // Remove fields…
                sel.as_field().is_some_and(|field| {
                    // except __schema and __type meta-fields,
                    // and fields of the schema introspection schema
                    !is_schema_introspection_meta_field(field) && !field.definition.is_built_in()
                })
            })?
        else {
            return Ok(Self::None);
        };
        let non_introspection_document =
            FilteredDocumentBuilder::single_operation(schema, document, operation, |sel| {
                // Remove __schema and __type
                sel.as_field()
                    .is_some_and(is_schema_introspection_meta_field)
            })?;
        let implementers_map = &OnceLock::new();
        let initial_value = &IntrospectionRootResolver(SchemaWithCache {
            schema,
            implementers_map,
        });
        let introspection_response = execute_query_or_mutation(
            schema,
            &introspection_document,
            variable_values,
            initial_value,
            operation,
        )?;
        if let Some(filtered_operation) = non_introspection_document {
            Ok(Self::Both {
                introspection_response,
                filtered_operation,
            })
        } else {
            Ok(Self::Only(introspection_response))
        }
    }

    /// Execute the [schema introspection] parts of an operation
    /// and wrap a callback to execute the rest (if any).
    ///
    /// This calls [`execute`][Self::execute]
    /// and handles each [`SchemaIntrospection`] enum variant as suggested by their documentation.
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    pub fn execute_with(
        schema: &Valid<Schema>,
        document: &Valid<ExecutableDocument>,
        operation_name: Option<&str>,
        variable_values: &Valid<JsonMap>,
        execute_non_introspection_parts: impl FnOnce(
            &Valid<ExecutableDocument>,
        ) -> Result<Response, RequestError>,
    ) -> Result<Response, RequestError> {
        match Self::execute(schema, document, operation_name, variable_values)? {
            Self::Only(response) => Ok(response),
            Self::None => execute_non_introspection_parts(document),
            Self::Both {
                filtered_operation,
                introspection_response,
            } => {
                Ok(execute_non_introspection_parts(&filtered_operation)?
                    .merge(introspection_response))
            }
        }
    }
}

#[derive(Clone, Copy)]
struct SchemaWithCache<'a> {
    schema: &'a Schema,
    implementers_map: &'a OnceLock<HashMap<Name, HashSet<Name>>>,
}

impl<'a> SchemaWithCache<'a> {
    fn implementers_of(&self, interface_name: &str) -> impl Iterator<Item = &'a Name> {
        self.implementers_map
            .get_or_init(|| self.schema.implementers_map())
            .get(interface_name)
            .into_iter()
            .flatten()
    }
}

impl<'a> std::ops::Deref for SchemaWithCache<'a> {
    type Target = &'a Schema;

    fn deref(&self) -> &Self::Target {
        &self.schema
    }
}

struct IntrospectionRootResolver<'a>(SchemaWithCache<'a>);

struct TypeDefResolver<'a> {
    schema: SchemaWithCache<'a>,
    name: &'a str,
    def: &'a schema::ExtendedType,
}

/// Only used for non-null and list types. `TypeDef` is used for everything else.
struct TypeResolver<'a> {
    schema: SchemaWithCache<'a>,
    ty: Cow<'a, schema::Type>,
}

struct DirectiveResolver<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::DirectiveDefinition,
}

struct FieldResolver<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::FieldDefinition,
}

struct EnumValueResolver<'a> {
    def: &'a schema::EnumValueDefinition,
}

struct InputValueResolver<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::InputValueDefinition,
}

fn type_def(schema: SchemaWithCache<'_>, name: impl AsRef<str>) -> ResolvedValue<'_> {
    ResolvedValue::opt_object(
        schema
            .types
            .get_key_value(name.as_ref())
            .map(|(name, def)| TypeDefResolver { schema, name, def }),
    )
}

fn type_def_opt<'a>(
    schema: SchemaWithCache<'a>,
    name: &Option<impl AsRef<str>>,
) -> ResolvedValue<'a> {
    if let Some(name) = name.as_ref() {
        type_def(schema, name)
    } else {
        ResolvedValue::null()
    }
}

fn ty<'a>(schema: SchemaWithCache<'a>, ty: &'a schema::Type) -> ResolvedValue<'a> {
    if let schema::Type::Named(name) = ty {
        type_def(schema, name)
    } else {
        ResolvedValue::object(TypeResolver {
            schema,
            ty: Cow::Borrowed(ty),
        })
    }
}

fn deprecation_reason(opt_directive: Option<&Node<schema::Directive>>) -> ResolvedValue<'_> {
    ResolvedValue::leaf(
        opt_directive
            .and_then(|directive| directive.argument_by_name("reason"))
            .and_then(|arg| arg.as_str()),
    )
}

impl_resolver! {
    for IntrospectionRootResolver<'_>:

    __typename = unreachable!();

    fn __schema(&self_) {
        Ok(ResolvedValue::object(self_.0))
    }

    fn __type(&self_, args) {
        let name = args["name"].as_str().unwrap();
        Ok(type_def(self_.0, name))
    }
}

impl_resolver! {
    for SchemaWithCache<'_>:

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
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
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
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
        Ok(ResolvedValue::list(def
            .values
            .values()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(EnumValueResolver { def })
            })
        ))
    }

    fn inputFields(&self_, args) {
        let schema::ExtendedType::InputObject(def) = self_.def else {
            return Ok(ResolvedValue::null());
        };
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
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
            .and_then(|dir| dir.argument_by_name("url"))
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
    fn specifiedBy() { Ok(ResolvedValue::null()) }
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
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
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
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
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
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
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
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
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
        Ok(ResolvedValue::leaf(self_.def.default_value.as_ref().map(|val| val.to_string())))
    }

    fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    fn deprecationReason(&self_) {
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
    }
}
