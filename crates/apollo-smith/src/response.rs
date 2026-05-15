use crate::generators::default_generators;
use crate::generators::Generator;
use crate::generators::Generators;
use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::executable::Field;
use apollo_compiler::executable::Selection;
use apollo_compiler::executable::SelectionSet;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Name;
use apollo_compiler::Node;
use apollo_compiler::Schema;
use indexmap::IndexMap;
use serde_json_bytes::json;
use serde_json_bytes::Map;
use serde_json_bytes::Value;

const TYPENAME: &str = "__typename";

/// Builds a GraphQL response which matches the shape of a given executable GraphQL document.
///
/// `ResponseBuilder` is generic over its randomness source via the [`RandomProvider`] trait.
/// This allows it to be used with [`arbitrary::Unstructured`] for fuzz testing, with
/// [`RandProvider`][crate::RandProvider] for standard random generation, or with any custom
/// implementation.
///
/// # Example
///
/// ```ignore
/// use apollo_smith::{ResponseBuilder, RandProvider};
///
/// let mut rng = RandProvider(rand::rng());
/// let response = ResponseBuilder::new(&mut rng, &doc, &schema).build()?;
/// ```
pub struct ResponseBuilder<'a, 'doc, 'schema, R: RandomProvider> {
    rng: &'a mut R,
    doc: &'doc Valid<ExecutableDocument>,
    schema: &'schema Valid<Schema>,
    generators: Generators<R>,
    min_list_size: usize,
    max_list_size: usize,
    null_ratio: Option<(u32, u32)>,
    operation_name: Option<&'doc str>,
}

impl<'a, 'doc, 'schema, R: RandomProvider> ResponseBuilder<'a, 'doc, 'schema, R> {
    /// Create a new `ResponseBuilder`.
    pub fn new(
        rng: &'a mut R,
        doc: &'doc Valid<ExecutableDocument>,
        schema: &'schema Valid<Schema>,
    ) -> Self {
        Self {
            rng,
            doc,
            schema,
            generators: default_generators(),
            min_list_size: 0,
            max_list_size: 5,
            null_ratio: None,
            operation_name: None,
        }
    }

    /// Register a [`Generator`] for the named GraphQL type.
    ///
    /// When the builder is about to produce a value of this type — at the root or any
    /// nested position, including each item of a list — the generator is invoked
    /// instead of the default behavior. For object, interface, and union types, the
    /// generator receives the requested fields with fragment spreads and inline
    /// fragments already flattened and grouped by response key, and its return value
    /// is used as-is (the builder does not recurse into it). For scalar types, the
    /// `fields` argument is empty and the generator's return value replaces the
    /// configured default.
    pub fn with_generator(
        mut self,
        type_name: Name,
        generator: Box<dyn Generator<R>>,
    ) -> Self {
        self.generators.insert(type_name, generator);
        self
    }

    /// Set the minimum number of items per list field. If unset, defaults to 0.
    pub fn with_min_list_size(mut self, min_size: usize) -> Self {
        self.min_list_size = min_size;
        self
    }

    /// Set the maximum number of items per list field. If unset, defaults to 5.
    pub fn with_max_list_size(mut self, max_size: usize) -> Self {
        self.max_list_size = max_size;
        self
    }

    /// Set the frequency of null values for nullable fields. If unset, fields will never be null.
    pub fn with_null_ratio(mut self, numerator: u32, denominator: u32) -> Self {
        self.null_ratio = Some((numerator, denominator));
        self
    }

    /// Set the operation name to generate a response for. If unset, uses the anonymous operation.
    /// If the operation does not exist, returns a response with `data: null`.
    pub fn with_operation_name(mut self, operation_name: Option<&'doc str>) -> Self {
        self.operation_name = operation_name;
        self
    }

    /// Builds a complete GraphQL response `Value` with a `data` key, matching the shape of `self.doc`.
    pub fn build(mut self) -> Result<Value, ResponseError> {
        if let Ok(operation) = self.doc.operations.get(self.operation_name) {
            let data = self.selection_set(&operation.selection_set)?;
            Ok(json!({ "data": data }))
        } else {
            Ok(json!({ "data": null }))
        }
    }

    /// Builds just the data portion of the response (without the `{ "data": ... }` wrapper).
    ///
    /// This is useful if you need to manipulate the response data before wrapping it into JSON.
    pub fn build_data(&mut self) -> Result<Value, ResponseError> {
        if let Ok(operation) = self.doc.operations.get(self.operation_name) {
            self.selection_set(&operation.selection_set)
        } else {
            Ok(Value::Null)
        }
    }

    /// Collect fields from a selection set, grouping by response key (alias or field name).
    ///
    /// Inline fragments and fragment spreads are flattened into the result, but only when
    /// their type condition applies to `concrete_type`. Fields sharing a response key are
    /// merged into a single entry.
    fn collect_fields(
        &self,
        selection_set: &SelectionSet,
        concrete_type: &Name,
    ) -> IndexMap<String, Vec<Node<Field>>> {
        let mut collected: IndexMap<String, Vec<Node<Field>>> = IndexMap::new();

        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    let key = field.alias.as_ref().unwrap_or(&field.name).to_string();
                    collected.entry(key).or_default().push(field.clone());
                }
                Selection::FragmentSpread(fragment) => {
                    if let Some(fragment_def) = self.doc.fragments.get(&fragment.fragment_name) {
                        if self.type_condition_matches(fragment_def.type_condition(), concrete_type)
                        {
                            for (key, mut fields) in
                                self.collect_fields(&fragment_def.selection_set, concrete_type)
                            {
                                collected.entry(key).or_default().append(&mut fields);
                            }
                        }
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    let matches = match &inline_fragment.type_condition {
                        None => true,
                        Some(cond) => self.type_condition_matches(cond, concrete_type),
                    };
                    if matches {
                        for (key, mut fields) in
                            self.collect_fields(&inline_fragment.selection_set, concrete_type)
                        {
                            collected.entry(key).or_default().append(&mut fields);
                        }
                    }
                }
            }
        }

        collected
    }

    /// Resolve the concrete object type that will be produced for `ty`.
    ///
    /// For unions, a random member is chosen. For interfaces, a random implementing
    /// object type is chosen. For object (or any other) types, `ty` is returned as-is.
    fn concrete_type<'s>(&mut self, ty: &'s Name) -> Result<&'s Name, ResponseError>
    where
        'schema: 's,
    {
        match self.schema.types.get(ty) {
            Some(ExtendedType::Union(union_ty)) => {
                let idx = self.rng.choose_index(union_ty.members.len())?;
                let member = union_ty
                    .members
                    .get_index(idx)
                    .expect("choose_index returned valid index");
                Ok(&member.name)
            }
            Some(ExtendedType::Interface(_)) => {
                let count = self
                    .schema
                    .types
                    .values()
                    .filter(|t| {
                        matches!(t, ExtendedType::Object(obj) if obj.implements_interfaces.contains(ty))
                    })
                    .count();
                if count == 0 {
                    return Ok(ty);
                }
                let idx = self.rng.choose_index(count)?;
                let chosen = self
                    .schema
                    .types
                    .iter()
                    .filter_map(|(name, t)| match t {
                        ExtendedType::Object(obj) if obj.implements_interfaces.contains(ty) => {
                            Some(name)
                        }
                        _ => None,
                    })
                    .nth(idx)
                    .expect("idx came from counting the same filter");
                Ok(chosen)
            }
            _ => Ok(ty),
        }
    }

    /// Whether a fragment with type condition `cond` should contribute fields when the
    /// object being generated has concrete type `concrete`.
    fn type_condition_matches(&self, cond: &Name, concrete: &Name) -> bool {
        if cond == concrete {
            return true;
        }
        match self.schema.types.get(cond) {
            Some(ExtendedType::Interface(_)) => matches!(
                self.schema.types.get(concrete),
                Some(ExtendedType::Object(obj)) if obj.implements_interfaces.contains(cond)
            ),
            Some(ExtendedType::Union(union_ty)) => {
                union_ty.members.iter().any(|m| m.name == *concrete)
            }
            _ => false,
        }
    }

    fn selection_set(&mut self, selection_set: &SelectionSet) -> Result<Value, ResponseError> {
        let concrete = self.concrete_type(&selection_set.ty)?;
        let grouped_fields = self.collect_fields(selection_set, concrete);

        if let Some(result) =
            self.generators
                .try_generate(&selection_set.ty, self.rng, &grouped_fields)
        {
            return result;
        }

        let mut result = Map::new();

        for (key, fields) in grouped_fields {
            // The first field is representative for schema-defined metadata (type, nullability, etc.)
            let meta_field = &fields[0];

            let val = if meta_field.name == TYPENAME {
                Value::String(concrete.to_string().into())
            } else if !meta_field.ty().is_non_null() && self.should_be_null()? {
                Value::Null
            } else {
                self.generate_field_value(&fields, meta_field)?
            };

            result.insert(key, val);
        }

        Ok(Value::Object(result))
    }

    /// Generate the value for a (possibly merged) field group.
    fn generate_field_value(
        &mut self,
        fields: &[Node<Field>],
        meta_field: &Node<Field>,
    ) -> Result<Value, ResponseError> {
        let has_selection_set = !meta_field.selection_set.is_empty();
        let is_list = meta_field.ty().is_list();

        if has_selection_set {
            // Merge sub-selections from all occurrences of this field
            let mut merged_selections = Vec::new();
            for field in fields {
                merged_selections.extend_from_slice(&field.selection_set.selections);
            }
            let full_selection_set = SelectionSet {
                ty: meta_field.selection_set.ty.clone(),
                selections: merged_selections,
            };

            if is_list {
                self.repeated_selection_set(&full_selection_set)
            } else {
                self.selection_set(&full_selection_set)
            }
        } else if is_list {
            self.repeated_leaf_field(meta_field.ty().inner_named_type())
        } else {
            self.leaf_field(meta_field.ty().inner_named_type())
        }
    }

    fn repeated_selection_set(
        &mut self,
        selection_set: &SelectionSet,
    ) -> Result<Value, ResponseError> {
        let num_values = self.arbitrary_len()?;
        let mut values = Vec::with_capacity(num_values);
        for _ in 0..num_values {
            values.push(self.selection_set(selection_set)?);
        }
        Ok(Value::Array(values))
    }

    fn leaf_field(&mut self, type_name: &Name) -> Result<Value, ResponseError> {
        let extended_ty = self
            .schema
            .types
            .get(type_name)
            .expect("validated schema should contain the type");
        match extended_ty {
            ExtendedType::Enum(enum_ty) => {
                let idx = self.rng.choose_index(enum_ty.values.len())?;
                let enum_value = enum_ty
                    .values
                    .values()
                    .nth(idx)
                    .expect("choose_index returned valid index");
                Ok(Value::String(enum_value.value.to_string().into()))
            }
            ExtendedType::Scalar(scalar) => {
                self.generators.generate_scalar(&scalar.name, self.rng)
            }
            _ => unreachable!("A field with an empty selection set must be a scalar or enum type"),
        }
    }

    fn repeated_leaf_field(&mut self, type_name: &Name) -> Result<Value, ResponseError> {
        let num_values = self.arbitrary_len()?;
        let mut values = Vec::with_capacity(num_values);
        for _ in 0..num_values {
            values.push(self.leaf_field(type_name)?);
        }
        Ok(Value::Array(values))
    }

    fn arbitrary_len(&mut self) -> Result<usize, ResponseError> {
        self.rng
            .gen_usize_range(self.min_list_size, self.max_list_size)
    }

    fn should_be_null(&mut self) -> Result<bool, ResponseError> {
        if let Some((numerator, denominator)) = self.null_ratio {
            self.rng.ratio(numerator, denominator)
        } else {
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RandProvider;

    /// Helper: parse schema + query, build a response with a seeded RNG.
    fn build_response(schema_sdl: &str, query: &str) -> Value {
        let schema = Schema::parse_and_validate(schema_sdl, "schema.graphql").unwrap();
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();
        let mut rng = RandProvider(rand::rng());
        ResponseBuilder::new(&mut rng, &doc, &schema)
            .with_null_ratio(0, 1) // never null, so we can assert field presence
            .with_min_list_size(1)
            .build()
            .unwrap()
    }

    const SIMPLE_SCHEMA: &str = r#"
        type Query {
            user(id: ID!): User
            posts: [Post!]!
        }
        type User {
            id: ID!
            name: String!
            email: String!
            address: Address!
            is_active: Boolean!
            distance: Float!
        }
        type Address {
            city: String!
            state: String!
        }
        type Post {
            id: ID!
            title: String!
            author: User!
            views: Int!
        }
    "#;

    const UNION_SCHEMA: &str = r#"
        type Query {
            user(id: ID!): User
        }
        type User {
            id: ID!
            name: String!
            content: [Content!]!
        }
        type Post {
            title: String!
            views: Int!
        }
        type Article {
            title: String!
            citations: [String!]!
        }
        union Content = Post | Article
    "#;

    const INTERFACE_SCHEMA: &str = r#"
        type Query {
            user(id: ID!): User
        }
        type User {
            id: ID!
            name: String!
            content: [Content!]!
        }
        interface Content {
            title: String!
        }
        type Post implements Content {
            title: String!
            views: Int!
        }
        type Article implements Content {
            title: String!
            citations: [String!]!
        }
    "#;

    #[test]
    fn basic_response_shape() {
        let response = build_response(
            SIMPLE_SCHEMA,
            "query { user(id: \"1\") { id name email is_active distance } }",
        );
        let data = response.get("data").expect("missing data");
        let user = data.get("user").expect("missing user");
        assert!(user.get("id").is_some());
        assert!(user.get("name").is_some());
        assert!(user.get("email").is_some());
        assert!(user.get("is_active").unwrap().is_boolean());
        assert!(user.get("distance").unwrap().is_number());
    }

    #[test]
    fn nested_objects() {
        let response = build_response(
            SIMPLE_SCHEMA,
            "query { user(id: \"1\") { id address { city state } } }",
        );
        let user = response.get("data").unwrap().get("user").unwrap();
        let address = user.get("address").expect("missing address");
        assert!(address.get("city").is_some());
        assert!(address.get("state").is_some());
    }

    #[test]
    fn list_fields() {
        let response = build_response(
            SIMPLE_SCHEMA,
            "query { posts { id title author { name } views } }",
        );
        let posts = response
            .get("data")
            .unwrap()
            .get("posts")
            .unwrap()
            .as_array()
            .expect("posts should be an array");
        assert!(!posts.is_empty(), "min_list_size is 1");
        for post in posts {
            assert!(post.get("id").is_some());
            assert!(post.get("title").is_some());
            assert!(post.get("views").is_some());
            assert!(post.get("author").unwrap().get("name").is_some());
        }
    }

    #[test]
    fn alias_support() {
        let response = build_response(
            SIMPLE_SCHEMA,
            r#"query { user(id: "1") { userId: id fullName: name } }"#,
        );
        let user = response.get("data").unwrap().get("user").unwrap();
        assert!(
            user.get("userId").is_some(),
            "alias 'userId' should be the key"
        );
        assert!(
            user.get("fullName").is_some(),
            "alias 'fullName' should be the key"
        );
        // The original field names should NOT appear
        assert!(user.get("id").is_none());
        assert!(user.get("name").is_none());
    }

    #[test]
    fn inline_fragment() {
        let response = build_response(
            SIMPLE_SCHEMA,
            r#"query { user(id: "1") { id ... on User { name email } } }"#,
        );
        let user = response.get("data").unwrap().get("user").unwrap();
        assert!(user.get("id").is_some());
        assert!(
            user.get("name").is_some(),
            "inline fragment field should be present"
        );
        assert!(
            user.get("email").is_some(),
            "inline fragment field should be present"
        );
    }

    #[test]
    fn fragment_spread() {
        let response = build_response(
            SIMPLE_SCHEMA,
            r#"
            fragment UserDetails on User {
                name
                email
                address { city state }
            }
            query { user(id: "1") { id ...UserDetails } }
            "#,
        );
        let user = response.get("data").unwrap().get("user").unwrap();
        assert!(user.get("id").is_some());
        assert!(
            user.get("name").is_some(),
            "fragment field should be present"
        );
        assert!(
            user.get("email").is_some(),
            "fragment field should be present"
        );
        let address = user
            .get("address")
            .expect("fragment nested object should be present");
        assert!(address.get("city").is_some());
        assert!(address.get("state").is_some());
    }

    #[test]
    fn union_typename_picks_member() {
        let schema = Schema::parse_and_validate(UNION_SCHEMA, "schema.graphql").unwrap();
        let query = r#"
            query {
                user(id: "1") {
                    content {
                        __typename
                        ... on Post { title views }
                        ... on Article { title citations }
                    }
                }
            }
        "#;
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

        let mut seen_post = false;
        let mut seen_article = false;

        // Run multiple times to increase confidence that we see both members
        for _ in 0..100 {
            let mut rng = RandProvider(rand::rng());
            let response = ResponseBuilder::new(&mut rng, &doc, &schema)
                .with_null_ratio(0, 1)
                .with_min_list_size(1)
                .build()
                .unwrap();

            let content = response
                .get("data")
                .unwrap()
                .get("user")
                .unwrap()
                .get("content")
                .unwrap()
                .as_array()
                .unwrap();

            for item in content {
                let typename = item.get("__typename").unwrap().as_str().unwrap();
                assert_ne!(typename, "Content", "__typename must not be the union name");
                assert!(
                    typename == "Post" || typename == "Article",
                    "unexpected __typename: {typename}"
                );
                // Each item should only have the fields from the inline fragment whose
                // type condition matches the chosen __typename.
                match typename {
                    "Post" => {
                        assert!(item.get("title").is_some(), "Post should have title");
                        assert!(item.get("views").is_some(), "Post should have views");
                        assert!(
                            item.get("citations").is_none(),
                            "Post should not have citations"
                        );
                        seen_post = true;
                    }
                    "Article" => {
                        assert!(item.get("title").is_some(), "Article should have title");
                        assert!(
                            item.get("citations").is_some(),
                            "Article should have citations"
                        );
                        assert!(item.get("views").is_none(), "Article should not have views");
                        seen_article = true
                    }
                    _ => unreachable!(),
                }
            }
        }

        assert!(seen_post, "should have seen Post at least once in 100 runs");
        assert!(
            seen_article,
            "should have seen Article at least once in 100 runs"
        );
    }

    #[test]
    fn interface_typename_picks_implementer() {
        let schema = Schema::parse_and_validate(INTERFACE_SCHEMA, "schema.graphql").unwrap();
        // `title` is on the interface and is requested unconditionally; the inline
        // fragments contribute fields specific to each implementer.
        let query = r#"
            query {
                user(id: "1") {
                    content {
                        __typename
                        title
                        ... on Post { views }
                        ... on Article { citations }
                    }
                }
            }
        "#;
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

        let mut seen_post = false;
        let mut seen_article = false;

        for _ in 0..100 {
            let mut rng = RandProvider(rand::rng());
            let response = ResponseBuilder::new(&mut rng, &doc, &schema)
                .with_null_ratio(0, 1)
                .with_min_list_size(1)
                .build()
                .unwrap();

            let content = response
                .get("data")
                .unwrap()
                .get("user")
                .unwrap()
                .get("content")
                .unwrap()
                .as_array()
                .unwrap();

            for item in content {
                let typename = item.get("__typename").unwrap().as_str().unwrap();
                assert_ne!(
                    typename, "Content",
                    "__typename must not be the interface name"
                );
                assert!(
                    typename == "Post" || typename == "Article",
                    "unexpected __typename: {typename}"
                );
                // `title` is selected at the interface level, so it should be present
                // regardless of which concrete type was chosen.
                assert!(
                    item.get("title").is_some(),
                    "title should always be present"
                );
                match typename {
                    "Post" => {
                        assert!(item.get("views").is_some(), "Post should have views");
                        assert!(
                            item.get("citations").is_none(),
                            "Post should not have citations"
                        );
                        seen_post = true;
                    }
                    "Article" => {
                        assert!(
                            item.get("citations").is_some(),
                            "Article should have citations"
                        );
                        assert!(item.get("views").is_none(), "Article should not have views");
                        seen_article = true;
                    }
                    _ => unreachable!(),
                }
            }
        }

        assert!(seen_post, "should have seen Post at least once in 100 runs");
        assert!(
            seen_article,
            "should have seen Article at least once in 100 runs"
        );
    }

    #[test]
    fn custom_object_override() {
        let schema_with_service = r#"
            type Query {
                _service: _Service!
            }
            type _Service {
                sdl: String!
            }
        "#;
        let schema = Schema::parse_and_validate(schema_with_service, "schema.graphql").unwrap();
        let query = "query { _service { sdl } }";
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

        struct SDLGenerator {
            sdl: String,
        }

        impl<R: RandomProvider> Generator<R> for SDLGenerator {
            fn generate(
                &mut self,
                _rng: &mut R,
                _generators: &mut Generators<R>,
                fields: &IndexMap<String, Vec<Node<Field>>>,
            ) -> Result<Value, ResponseError> {
                let mut service_obj = Map::new();
                for (key, group) in fields {
                    if group[0].name == "sdl" {
                        service_obj.insert(key.clone(), Value::String(self.sdl.clone().into()));
                    }
                }
                Ok(Value::Object(service_obj))
            }
        }

        let custom_sdl = "type Query { hello: String }";
        let mut rng = RandProvider(rand::rng());
        let response = ResponseBuilder::new(&mut rng, &doc, &schema)
            .with_generator(
                Name::new_unchecked("_Service"),
                Box::new(SDLGenerator {
                    sdl: custom_sdl.to_owned(),
                }) as Box<dyn Generator<_>>,
            )
            .build()
            .unwrap();

        let sdl = response
            .get("data")
            .unwrap()
            .get("_service")
            .unwrap()
            .get("sdl")
            .unwrap()
            .as_str()
            .unwrap();

        assert_eq!(sdl, custom_sdl);
    }

    #[test]
    fn custom_scalar_override() {
        let schema_sdl = r#"
            scalar UUID
            type Query {
                id: UUID!
            }
        "#;
        let schema = Schema::parse_and_validate(schema_sdl, "schema.graphql").unwrap();
        let query = "query { id }";
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

        struct ConstantGenerator(&'static str);

        impl<R: RandomProvider> Generator<R> for ConstantGenerator {
            fn generate(
                &mut self,
                _rng: &mut R,
                _generators: &mut Generators<R>,
                _fields: &IndexMap<String, Vec<Node<Field>>>,
            ) -> Result<Value, ResponseError> {
                Ok(Value::String(self.0.into()))
            }
        }

        let mut rng = RandProvider(rand::rng());
        let response = ResponseBuilder::new(&mut rng, &doc, &schema)
            .with_generator(
                Name::new_unchecked("UUID"),
                Box::new(ConstantGenerator("00000000-0000-0000-0000-000000000000"))
                    as Box<dyn Generator<_>>,
            )
            .build()
            .unwrap();

        let id = response
            .get("data")
            .unwrap()
            .get("id")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(id, "00000000-0000-0000-0000-000000000000");
    }

    #[test]
    fn object_generator_delegates_to_scalar_generators() {
        // An ObjectGenerator that doesn't want to hand-roll every leaf field can
        // delegate to the builder's registered scalar generators. Here we register a
        // custom `ID` generator and assert the object generator reuses it.
        let schema = Schema::parse_and_validate(SIMPLE_SCHEMA, "schema.graphql").unwrap();
        let query = r#"query { user(id: "1") { id name } }"#;
        let doc = ExecutableDocument::parse_and_validate(&schema, query, "query.graphql").unwrap();

        struct UserGenerator;
        impl<R: RandomProvider> Generator<R> for UserGenerator {
            fn generate(
                &mut self,
                rng: &mut R,
                generators: &mut Generators<R>,
                fields: &IndexMap<String, Vec<Node<Field>>>,
            ) -> Result<Value, ResponseError> {
                let mut obj = Map::new();
                for (key, group) in fields {
                    let scalar_name = group[0].ty().inner_named_type();
                    obj.insert(key.clone(), generators.generate_scalar(scalar_name, rng)?);
                }
                Ok(Value::Object(obj))
            }
        }

        struct ConstantGenerator(&'static str);
        impl<R: RandomProvider> Generator<R> for ConstantGenerator {
            fn generate(
                &mut self,
                _rng: &mut R,
                _generators: &mut Generators<R>,
                _fields: &IndexMap<String, Vec<Node<Field>>>,
            ) -> Result<Value, ResponseError> {
                Ok(Value::String(self.0.into()))
            }
        }

        let mut rng = RandProvider(rand::rng());
        let response = ResponseBuilder::new(&mut rng, &doc, &schema)
            .with_generator(
                Name::new_unchecked("ID"),
                Box::new(ConstantGenerator("user-id")) as Box<dyn Generator<_>>,
            )
            .with_generator(
                Name::new_unchecked("User"),
                Box::new(UserGenerator) as Box<dyn Generator<_>>,
            )
            .build()
            .unwrap();

        let user = response.get("data").unwrap().get("user").unwrap();
        assert_eq!(user.get("id").unwrap().as_str().unwrap(), "user-id");
        // `name` is a String — no custom scalar registered, so it falls back to the
        // default alphanumeric generator (length 1–10).
        let name = user.get("name").unwrap().as_str().unwrap();
        assert!((1..=10).contains(&name.len()));
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric()));
    }
}
