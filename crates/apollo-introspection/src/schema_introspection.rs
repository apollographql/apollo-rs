use crate::execution::execute_query_or_mutation;
use crate::execution::get_operation;
use crate::input_coercion::VariableValues;
use crate::resolver::ResolvedValue;
use crate::response::request_error;
use crate::response::RequestErrorResponse;
use crate::response::Response;
use crate::JsonMap;
use apollo_compiler::executable::Fragment;
use apollo_compiler::executable::InlineFragment;
use apollo_compiler::executable::Operation;
use apollo_compiler::executable::OperationType;
use apollo_compiler::executable::Selection;
use apollo_compiler::executable::SelectionSet;
use apollo_compiler::schema;
use apollo_compiler::schema::Name;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use std::borrow::Cow;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::OnceLock;

/// The [schema introspection] selections that have been removed from an `Operation`.
///
/// Schema introspection selections are those for the `__schema` or `__type` meta-fields.
///
/// See [`Self::split_from`]
///
/// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
pub struct SchemaIntrospectionQuery {
    document: Option<ExecutableDocument>,
}

impl SchemaIntrospectionQuery {
    /// Remove and return [schema introspection] selections at the root of a query operation.
    ///
    /// Schema introspection selections are those for the `__schema` or `__type` meta-fields.
    ///
    /// Returns an error if schema introspection selections are found nested in a field sub-selection.
    /// This is possible (while keeping the operation valid) with an unconventional schema
    /// where the root query type is also the type of a field.
    ///
    /// May leave behind empty fragment definitions.
    /// (TODO: fix this? it means also finding and removing corresponding spreads.)
    ///
    /// [schema introspection]: https://spec.graphql.org/October2021/#sec-Schema-Introspection
    pub fn split_from(
        document: &mut ExecutableDocument,
        operation_name: Option<&str>,
    ) -> Result<Self, RequestErrorResponse> {
        let operation = get_operation(document, operation_name)?;
        if operation.operation_type != OperationType::Query {
            return Ok(Self::empty());
        }
        let mut fragments_to_visit: HashSet<_> = document.fragments.keys().collect();
        let mut fragments_to_split = HashSet::new();
        let mut at_root = false;
        let mut nested = false;
        contains_root(
            document,
            &mut fragments_to_visit,
            &mut fragments_to_split,
            &mut at_root,
            &mut nested,
            &operation.selection_set,
        );
        if nested {
            return Err(request_error(
                "Schema introspection meta-fields are only supported at the root of a query",
            ));
        }
        if !at_root {
            // This query does not select any schema introspection meta-fields
            return Ok(Self::empty());
        }
        let operation = document.get_operation_mut(operation_name).unwrap();
        let ty = operation.selection_set.ty.clone();
        let selections = collect_from(&mut operation.selection_set.selections);
        let mut introspection_document = ExecutableDocument::new();
        introspection_document.anonymous_operation = Some(Node::new(Operation {
            operation_type: OperationType::Query,
            variables: Vec::new(),          // unused by execution
            directives: Default::default(), // unused by execution
            selection_set: SelectionSet { ty, selections },
        }));
        introspection_document
            .fragments
            .extend(fragments_to_split.into_iter().map(|name| {
                let fragment = document.fragments[&name].make_mut();
                let ty = fragment.selection_set.ty.clone();
                let selections = collect_from(&mut fragment.selection_set.selections);
                let new_fragment = Fragment {
                    directives: Default::default(), // unused by execution
                    selection_set: SelectionSet { ty, selections },
                };
                (name, Node::new(new_fragment))
            }));
        let intropsection_fragments: Vec<_> = document
            .fragments
            .iter()
            .filter(|(_name, def)| def.type_condition().starts_with("__"))
            .map(|(name, _def)| name.clone())
            .collect();
        introspection_document
            .fragments
            .extend(intropsection_fragments.into_iter().map(|name| {
                let def = document.fragments.remove(&name).unwrap();
                (name, def)
            }));
        Ok(Self {
            document: Some(introspection_document),
        })
    }

    fn empty() -> Self {
        Self { document: None }
    }

    pub fn is_empty(&self) -> bool {
        self.document.is_none()
    }

    pub fn document(&self) -> Option<&ExecutableDocument> {
        self.document.as_ref()
    }

    /// Execute this schema intropsection query in a synchronous context.
    ///
    /// Because execution uses `async` internally, this uses a single-threaded executor.
    /// If calling from another executor, call [`execute`][Self::execute] instead.
    pub fn execute_sync(
        &self,
        schema: &Schema,
        variable_values: &VariableValues,
    ) -> Result<Response, RequestErrorResponse> {
        futures::executor::block_on(self.execute(schema, variable_values))
    }

    /// Execute this schema intropsection query in an asynchronous context
    pub async fn execute(
        &self,
        schema: &Schema,
        variable_values: &VariableValues,
    ) -> Result<Response, RequestErrorResponse> {
        if let Some(document) = &self.document {
            let operation = document.anonymous_operation.as_ref().unwrap();
            let implementers_map = &OnceLock::new();
            let initial_value = &IntrospectionRoot(SchemaWithCache {
                schema,
                implementers_map,
            });
            execute_query_or_mutation(schema, document, variable_values, initial_value, operation)
                .await
        } else {
            Ok(Response {
                data: Some(JsonMap::new()),
                errors: Vec::new(),
            })
        }
    }
}

/// Returns (contains at the root, contains nested)
fn contains_root(
    document: &ExecutableDocument,
    fragments_to_visit: &mut HashSet<&Name>,
    fragments_to_split: &mut HashSet<Name>,
    at_root: &mut bool,
    nested: &mut bool,
    selection_set: &SelectionSet,
) {
    for selection in &selection_set.selections {
        match selection {
            Selection::Field(field) => {
                *at_root |= field.name == "__schema" || field.name == "__type";
                *nested |= contains(document, &field.selection_set);
            }
            Selection::FragmentSpread(spread) => {
                // Remove to break cycles
                if fragments_to_visit.remove(&spread.fragment_name) {
                    let fragment = &document.fragments[&spread.fragment_name];
                    let mut at_root_in_this_fragment = false;
                    contains_root(
                        document,
                        fragments_to_visit,
                        fragments_to_split,
                        &mut at_root_in_this_fragment,
                        nested,
                        &fragment.selection_set,
                    );
                    if at_root_in_this_fragment {
                        *at_root = true;
                        fragments_to_split.insert(spread.fragment_name.clone());
                    }
                }
            }
            Selection::InlineFragment(inline) => {
                contains_root(
                    document,
                    fragments_to_visit,
                    fragments_to_split,
                    at_root,
                    nested,
                    &inline.selection_set,
                );
            }
        }
    }
}

fn contains(document: &ExecutableDocument, selection_set: &SelectionSet) -> bool {
    selection_set
        .selections
        .iter()
        .any(|selection| match selection {
            Selection::Field(field) => {
                field.name == "__schema"
                    || field.name == "__type"
                    || contains(document, &field.selection_set)
            }
            Selection::FragmentSpread(spread) => document
                .fragments
                .get(&spread.fragment_name)
                .is_some_and(|fragment| contains(document, &fragment.selection_set)),
            Selection::InlineFragment(inline) => contains(document, &inline.selection_set),
        })
}

fn collect_from(selections: &mut Vec<Selection>) -> Vec<Selection> {
    // TODO: use Vec::extract_if when available https://github.com/rust-lang/rust/issues/43244
    let mut extracted = Vec::new();
    let mut index = 0;
    while index < selections.len() {
        let selection = &mut selections[index];
        match selection {
            Selection::Field(field) => {
                if field.name == "__schema" || field.name == "__type" {
                    extracted.push(selections.remove(index));
                    // Don’t increment `index` as `remove` shifts remaining selections
                    continue;
                }
            }
            Selection::FragmentSpread(_) => extracted.push(selection.clone()),
            Selection::InlineFragment(inline) => {
                let collected = collect_from(&mut inline.make_mut().selection_set.selections);
                if !collected.is_empty() {
                    extracted.push(Selection::InlineFragment(Node::new(InlineFragment {
                        type_condition: inline.type_condition.clone(),
                        directives: inline.directives.clone(),
                        selection_set: SelectionSet {
                            ty: inline.selection_set.ty.clone(),
                            selections: collected,
                        },
                    })))
                }
            }
        }
        index += 1;
    }
    extracted
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

struct IntrospectionRoot<'a>(SchemaWithCache<'a>);

struct TypeDef<'a> {
    schema: SchemaWithCache<'a>,
    name: &'a str,
    def: &'a schema::ExtendedType,
}

/// Only used for non-null and list types. `TypeDef` is used for everything else.
struct Type<'a> {
    schema: SchemaWithCache<'a>,
    ty: Cow<'a, schema::Type>,
}

struct Directive<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::DirectiveDefinition,
}

struct Field<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::FieldDefinition,
}

struct EnumValue<'a> {
    def: &'a schema::EnumValueDefinition,
}

struct InputValue<'a> {
    schema: SchemaWithCache<'a>,
    def: &'a schema::InputValueDefinition,
}

fn type_def(schema: SchemaWithCache<'_>, name: impl AsRef<str>) -> ResolvedValue<'_> {
    ResolvedValue::opt_object(
        schema
            .types
            .get_key_value(name.as_ref())
            .map(|(name, def)| TypeDef { schema, name, def }),
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
        ResolvedValue::object(Type {
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
    for IntrospectionRoot<'_>:

    __typename = unreachable!();

    async fn __schema(&self_) {
        Ok(ResolvedValue::object(self_.0))
    }

    async fn __type(&self_, args) {
        let name = args["name"].as_str().unwrap();
        Ok(type_def(self_.0, name))
    }
}

impl_resolver! {
    for SchemaWithCache<'_>:

    __typename = "__Schema";

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.schema_definition.description.as_deref()))
    }

    async fn types(&self_) {
        Ok(ResolvedValue::list(self_.types.iter().map(|(name, def)| {
            ResolvedValue::object(TypeDef { schema: *self_, name, def })
        })))
    }

    async fn directives(&self_) {
        Ok(ResolvedValue::list(self_.directive_definitions.values().map(|def| {
            ResolvedValue::object(Directive { schema: *self_, def })
        })))
    }

    async fn queryType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.query))
    }

    async fn mutationType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.mutation))
    }

    async fn subscriptionType(&self_) {
        Ok(type_def_opt(*self_, &self_.schema_definition.subscription))
    }
}

impl_resolver! {
    for TypeDef<'_>:

    __typename = "__Type";

    async fn kind(&self_) {
        Ok(ResolvedValue::leaf(match self_.def {
            schema::ExtendedType::Scalar(_) => "SCALAR",
            schema::ExtendedType::Object(_) => "OBJECT",
            schema::ExtendedType::Interface(_) => "INTERFACE",
            schema::ExtendedType::Union(_) => "UNION",
            schema::ExtendedType::Enum(_) => "ENUM",
            schema::ExtendedType::InputObject(_) => "INPUT_OBJECT",
        }))
    }

    async fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.name))
    }

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description().map(|desc| desc.as_str())))
    }

    async fn fields(&self_, args) {
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
                ResolvedValue::object(Field { schema: self_.schema, def })
            })
        ))
    }

    async fn interfaces(&self_) {
        let implements_interfaces = match self_.def {
            schema::ExtendedType::Object(def) => &def.implements_interfaces,
            schema::ExtendedType::Interface(def) => &def.implements_interfaces,
            schema::ExtendedType::Scalar(_) |
            schema::ExtendedType::Union(_) |
            schema::ExtendedType::Enum(_) |
            schema::ExtendedType::InputObject(_) => return Ok(ResolvedValue::null()),
        };
        Ok(ResolvedValue::list(implements_interfaces.iter().filter_map(|name| {
            self_.schema.types.get(&name.node).map(|def| {
                ResolvedValue::object(TypeDef { schema: self_.schema, name, def })
            })
        })))
    }

    async fn possibleTypes(&self_) {
        macro_rules! types {
            ($names: expr) => {
                Ok(ResolvedValue::list($names.filter_map(move |name| {
                    self_.schema.types.get(name).map(move |def| {
                        ResolvedValue::object(TypeDef { schema: self_.schema, name, def })
                    })
                })))
            }
        }
        match self_.def {
            schema::ExtendedType::Interface(_) => types!(self_.schema.implementers_of(self_.name)),
            schema::ExtendedType::Union(def) => types!(def.members.iter().map(|c| &c.node)),
            schema::ExtendedType::Object(_) |
            schema::ExtendedType::Scalar(_) |
            schema::ExtendedType::Enum(_) |
            schema::ExtendedType::InputObject(_) => Ok(ResolvedValue::null()),
        }
    }

    async fn enumValues(&self_, args) {
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
                ResolvedValue::object(EnumValue { def })
            })
        ))
    }

    async fn inputFields(&self_, args) {
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
                ResolvedValue::object(InputValue { schema: self_.schema, def })
            })
        ))
    }

    async fn ofType() {
        Ok(ResolvedValue::null())
    }

    async fn specifiedByURL(&self_) {
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
    for Type<'_>:

    __typename = "__Type";

    async fn kind(&self_) {
        Ok(ResolvedValue::leaf(match &*self_.ty {
            schema::Type::Named(_) => unreachable!(),
            schema::Type::List(_) => "LIST",
            schema::Type::NonNullNamed(_) |
            schema::Type::NonNullList(_) => "NON_NULL",
        }))
    }

    async fn ofType(&self_) {
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

    async fn name() { Ok(ResolvedValue::null()) }
    async fn description() { Ok(ResolvedValue::null()) }
    async fn fields() { Ok(ResolvedValue::null()) }
    async fn interfaces() { Ok(ResolvedValue::null()) }
    async fn possibleTypes() { Ok(ResolvedValue::null()) }
    async fn enumValues() { Ok(ResolvedValue::null()) }
    async fn inputFields() { Ok(ResolvedValue::null()) }
    async fn specifiedBy() { Ok(ResolvedValue::null()) }
}

impl_resolver! {
    for Directive<'_>:

    __typename = "__Directive";

    async fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_str()))
    }

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    async fn args(&self_, args) {
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
        Ok(ResolvedValue::list(self_
            .def
            .arguments
            .iter()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(InputValue { schema: self_.schema, def })
            })
        ))
    }

    async fn locations(&self_) {
        Ok(ResolvedValue::list(self_.def.locations.iter().map(|loc| {
            ResolvedValue::leaf(loc.name())
        })))
    }

    async fn isRepeatable(&self_) {
        Ok(ResolvedValue::leaf(self_.def.repeatable))
    }
}

impl_resolver! {
    for Field<'_>:

    __typename = "__Field";

    async fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_str()))
    }

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    async fn args(&self_, args) {
        let include_deprecated = args["includeDeprecated"].as_bool().unwrap();
        Ok(ResolvedValue::list(self_
            .def
            .arguments
            .iter()
            .filter(move |def| {
                include_deprecated || def.directives.get("deprecated").is_none()
            })
            .map(|def| {
                ResolvedValue::object(InputValue { schema: self_.schema, def })
            })
        ))
    }

    async fn type(&self_) {
        Ok(ty(self_.schema, &self_.def.ty))
    }

    async fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    async fn deprecationReason(&self_) {
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
    }
}

impl_resolver! {
    for EnumValue<'_>:

    __typename = "__EnumValue";

    async fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.value.as_str()))
    }

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    async fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    async fn deprecationReason(&self_) {
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
    }
}

impl_resolver! {
    for InputValue<'_>:

    __typename = "__InputValue";

    async fn name(&self_) {
        Ok(ResolvedValue::leaf(self_.def.name.as_ref()))
    }

    async fn description(&self_) {
        Ok(ResolvedValue::leaf(self_.def.description.as_deref()))
    }

    async fn type(&self_) {
        Ok(ty(self_.schema, &self_.def.ty))
    }

    async fn defaultValue(&self_) {
        Ok(ResolvedValue::leaf(self_.def.default_value.as_ref().map(|val| val.to_string())))
    }

    async fn isDeprecated(&self_) {
        Ok(ResolvedValue::leaf(self_.def.directives.get("deprecated").is_some()))
    }

    async fn deprecationReason(&self_) {
        Ok(deprecation_reason(self_.def.directives.get("deprecated")))
    }
}
