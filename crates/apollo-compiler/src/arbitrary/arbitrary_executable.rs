use crate::arbitrary::common::abritrary_directive_list;
use crate::arbitrary::common::arbitary_name;
use crate::arbitrary::common::arbitrary_arguments;
use crate::arbitrary::common::gather_directive_definitions_by_location;
use crate::arbitrary::common::Context;
use crate::arbitrary::common::DirectiveDefinitionsByLocation;
use crate::arbitrary::entropy::Entropy;
use crate::executable;
use crate::schema;
use crate::schema::DirectiveLocation;
use crate::schema::MetaFieldDefinitions;
use crate::schema::NamedType;
use crate::validation::Valid;
use crate::Name;
use crate::Node;
use crate::Schema;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashMap;

/// Generate a executable document valid for `schema`, from bytes typically provided by a fuzzer.
///
/// The “size” of the document is roughly proportional to `arbitrary_bytes.len()`.
///
/// Ideally, enumerating all possible byte strings should yield all possible valid documents.
/// However there may be known and unknown cases that are not implemented yet.
pub fn arbitrary_valid_executable_document(
    schema: &Valid<Schema>,
    arbitrary_bytes: &[u8],
) -> Valid<executable::ExecutableDocument> {
    let mut builder = Builder {
        schema,
        compatible_types_map: gather_compatible_types(schema),
        directive_definitions_by_location: gather_directive_definitions_by_location(schema),
        entropy: Entropy::new(arbitrary_bytes),
        operation_variables: Default::default(),
        fragment_map: Default::default(),
        fragment_counter: 0,
        field_counter: 0,
    };

    let (operation_type, root_type) = builder.arbitrary_root_operation();
    let is_subscription = operation_type == executable::OperationType::Subscription;

    let operation_is_named = builder.entropy.bool();
    let name = operation_is_named.then(|| arbitary_name(&mut builder.entropy));

    let mut operation_selection_set = builder.arbitrary_selection_set_with_one_selection(root_type);
    while !builder.entropy.is_empty() {
        builder.expand_arbitrary_selection_set(&mut operation_selection_set, is_subscription);
    }
    // Make fragment ordering independent of `swap_remove` + re-`insert` operations.
    builder
        .fragment_map
        .sort_by_cached_key(|name, _| name.strip_prefix("Frag").unwrap().parse::<u32>().unwrap());

    let location = match operation_type {
        executable::OperationType::Query => DirectiveLocation::Query,
        executable::OperationType::Mutation => DirectiveLocation::Mutation,
        executable::OperationType::Subscription => DirectiveLocation::Subscription,
    };
    let doc = executable::ExecutableDocument {
        sources: Default::default(),
        operations: executable::OperationMap::from_one(executable::Operation {
            operation_type,
            name,
            directives: builder.abritrary_directive_list(location),
            variables: builder.operation_variables,
            selection_set: operation_selection_set,
        }),
        fragments: builder.fragment_map,
    };
    doc.validate(schema)
        .expect("bug in arbitrary_valid_executable_document")
}

struct Builder<'a> {
    schema: &'a Valid<Schema>,
    compatible_types_map: CompatibleTypesMap<'a>,
    directive_definitions_by_location: DirectiveDefinitionsByLocation<'a>,
    entropy: Entropy<'a>,
    fragment_map: executable::FragmentMap,
    operation_variables: Vec<Node<executable::VariableDefinition>>,
    fragment_counter: usize,
    field_counter: usize,
}

/// For all output non-leaf types in the schema, the set of type conditions that a fragment can have:
/// <https://spec.graphql.org/draft/#sec-Fragment-Spread-Is-Possible>
type CompatibleTypesMap<'schema> = HashMap<&'schema NamedType, IndexSet<&'schema NamedType>>;

impl<'a> Builder<'a> {
    fn arbitrary_root_operation(&mut self) -> (executable::OperationType, NamedType) {
        let count = self.schema.schema_definition.iter_root_operations().count();
        let index = self.entropy.index(count).expect(
            "valid schema unexpectedly lacks any root operation, should have at least a query",
        );
        let (operation_type, root_type) = self
            .schema
            .schema_definition
            .iter_root_operations()
            .nth(index)
            .unwrap();
        (operation_type, root_type.name.clone())
    }

    fn context(&mut self) -> Context<'_, 'a> {
        Context {
            schema: self.schema,
            directive_definitions_by_location: &self.directive_definitions_by_location,
            entropy: &mut self.entropy,
            variable_definitions: Some(&mut self.operation_variables),
        }
    }

    fn abritrary_directive_list(
        &mut self,
        location: DirectiveLocation,
    ) -> executable::DirectiveList {
        abritrary_directive_list(&mut self.context(), location)
    }

    /// Create a selection set with one arbitrary selection
    fn arbitrary_selection_set_with_one_selection(
        &mut self,
        ty: NamedType,
    ) -> executable::SelectionSet {
        let mut selection_set = executable::SelectionSet::new(ty);
        self.arbitrary_selection_into(&mut selection_set);
        selection_set
    }

    /// Add some selections to the given non-empty selection set
    /// or some of its nested selection sets
    fn expand_arbitrary_selection_set(
        &mut self,
        selection_set: &mut executable::SelectionSet,
        is_subscription_top_level: bool,
    ) {
        // If a subscription top-level already has selections
        // we can’t add sibling ones so we must go deeper
        let go_deeper = is_subscription_top_level || self.entropy.bool();
        if !go_deeper {
            return self.arbitrary_selection_into(selection_set);
        }

        // unwrap: `expand_selection_set` is only called on non-empty selection sets
        // (initially created by `arbitrary_selection_set_with_one_selection`)
        // so `index()` returns `Some`
        let index = self.entropy.index(selection_set.selections.len()).unwrap();
        match &mut selection_set.selections[index] {
            executable::Selection::InlineFragment(inline) => {
                let selection_set_to_expand = &mut inline.make_mut().selection_set;
                self.expand_arbitrary_selection_set(
                    selection_set_to_expand,
                    is_subscription_top_level,
                );
            }
            executable::Selection::FragmentSpread(spread) => {
                // Temporarily remove the fragment definition from the map
                // so that we can mutably borrow its selection set independently of `&mut self`
                let mut fragment_def = self
                    .fragment_map
                    .swap_remove(&spread.fragment_name)
                    .expect("spread of undefined named fragment");
                let selection_set_to_expand = &mut fragment_def.make_mut().selection_set;
                self.expand_arbitrary_selection_set(
                    selection_set_to_expand,
                    is_subscription_top_level,
                );
                self.fragment_map
                    .insert(spread.fragment_name.clone(), fragment_def);
            }
            executable::Selection::Field(field) => {
                if field
                    .inner_type_def(self.schema)
                    .expect("field of undefined type")
                    .is_leaf()
                {
                    // A leaf field cannot be expanded
                    if is_subscription_top_level {
                        // There is nothing else we can expand while
                        // keeping a single response-top-level field for a subscription,
                        // so end `arbitrary_executable_document`’s loop here.
                        self.entropy.take_all();
                    } else {
                        // Give it a new sibling instead
                        self.arbitrary_selection_into(selection_set);
                    }
                } else {
                    let selection_set_to_expand = &mut field.make_mut().selection_set;
                    let nested_is_subscription_top_level = false;
                    self.expand_arbitrary_selection_set(
                        selection_set_to_expand,
                        nested_is_subscription_top_level,
                    );
                }
            }
        }
    }

    fn arbitrary_selection_into(&mut self, selection_set: &mut executable::SelectionSet) {
        match self.entropy.u8() {
            // 50% of cases
            0..=127 => self.arbitrary_field_into(selection_set),

            // 37.5% of cases
            128..=223 => self.arbitrary_inline_fragment_into(selection_set),

            // 12.5% of cases
            _ => self.arbitrary_fragment_spread_into(selection_set),
        }
    }

    fn arbitrary_field_into(&mut self, selection_set: &mut executable::SelectionSet) {
        // TODO: don’t always generate an alias, sometimes use an already-used response key while
        // ensuring https://spec.graphql.org/draft/#sec-Field-Selection-Merging
        let alias = Some(Name::try_from(format!("field{}", self.field_counter)).unwrap());
        self.field_counter += 1;

        let empty = IndexMap::new();
        let explicit_fields = match &self.schema.types[&selection_set.ty] {
            schema::ExtendedType::Interface(def) => &def.fields,
            schema::ExtendedType::Object(def) => &def.fields,
            schema::ExtendedType::Scalar(_)
            | schema::ExtendedType::Union(_)
            | schema::ExtendedType::Enum(_)
            | schema::ExtendedType::InputObject(_) => &empty,
        };
        let meta_fields = [&MetaFieldDefinitions::get().__typename];
        let field_count = meta_fields.len() + explicit_fields.len();
        // unwrap: `field_count` is always at least 1 for `__typename`
        let choice = self.entropy.index(field_count).unwrap();
        let definition = if let Some(index) = choice.checked_sub(meta_fields.len()) {
            explicit_fields[index].node.clone()
        } else {
            meta_fields[choice].node.clone()
        };
        let arguments = arbitrary_arguments(&mut self.context(), &definition.arguments);
        let ty = definition.ty.inner_named_type().clone();
        selection_set.push(executable::Field {
            alias,
            name: definition.name.clone(),
            arguments,
            directives: self.abritrary_directive_list(DirectiveLocation::Field),
            selection_set: if self.schema.types[&ty].is_leaf() {
                executable::SelectionSet::new(ty)
            } else {
                self.arbitrary_selection_set_with_one_selection(ty)
            },
            definition,
        })
    }

    fn arbitrary_inline_fragment_into(&mut self, selection_set: &mut executable::SelectionSet) {
        let nested_type;
        let type_condition;
        let use_type_condition = self.entropy.bool();
        if use_type_condition {
            nested_type = self.abritrary_type_condition(&selection_set.ty);
            type_condition = Some(nested_type.clone());
        } else {
            nested_type = selection_set.ty.clone();
            type_condition = None;
        };
        selection_set.push(executable::InlineFragment {
            type_condition,
            directives: self.abritrary_directive_list(DirectiveLocation::InlineFragment),
            selection_set: self.arbitrary_selection_set_with_one_selection(nested_type),
        })
    }

    fn arbitrary_fragment_spread_into(&mut self, selection_set: &mut executable::SelectionSet) {
        // TODO: sometimes add spreads of existing named fragments?
        // This needs a way to track and avoid introducing cycles
        // <https://spec.graphql.org/draft/#sec-Fragment-Spreads-Must-Not-Form-Cycles>

        let fragment_name = Name::try_from(format!("Frag{}", self.fragment_counter)).unwrap();
        self.fragment_counter += 1;
        let fragment_type = self.abritrary_type_condition(&selection_set.ty);
        let fragment_def = executable::Fragment {
            name: fragment_name.clone(),
            directives: self.abritrary_directive_list(DirectiveLocation::FragmentDefinition),
            selection_set: self.arbitrary_selection_set_with_one_selection(fragment_type),
        };
        self.fragment_map
            .insert(fragment_name.clone(), fragment_def.into());
        selection_set.push(executable::FragmentSpread {
            fragment_name,
            directives: self.abritrary_directive_list(DirectiveLocation::FragmentSpread),
        });
    }

    fn abritrary_type_condition(&mut self, parent_selection_set_type: &NamedType) -> NamedType {
        let compatible_types = &self.compatible_types_map[parent_selection_set_type];
        // unwrap: `compatible_types` is non-empty
        // since every type is at least compatible with itself
        let index = self.entropy.index(compatible_types.len()).unwrap();
        compatible_types[index].clone()
    }
}

/// For all output non-leaf types in the schema, the set of type conditions that a fragment can have:
/// <https://spec.graphql.org/draft/#sec-Fragment-Spread-Is-Possible>
// Clippy false positive: https://github.com/rust-lang/rust-clippy/issues/12908
#[allow(clippy::needless_lifetimes)]
fn gather_compatible_types<'schema>(schema: &'schema Valid<Schema>) -> CompatibleTypesMap<'schema> {
    // key: interface type name, value: types that implement this interface
    let implementers_map = schema.implementers_map();

    // key: object type name, values: unions this object is a member of
    let mut unions_map = HashMap::<&NamedType, Vec<&NamedType>>::new();
    for (name, type_def) in &schema.types {
        if let schema::ExtendedType::Union(def) = type_def {
            for member in &def.members {
                unions_map.entry(member).or_default().push(name)
            }
        }
    }

    schema
        .types
        .iter()
        .filter(|(_name, type_def)| type_def.is_output_type() && !type_def.is_leaf())
        .map(|(name, type_def)| {
            let mut compatible_types = IndexSet::new();
            // Any type is compatible with itself
            compatible_types.insert(name);

            let mut add_object_type_and_its_supertypes = |object: &'schema schema::ObjectType| {
                compatible_types.insert(&object.name);
                if let Some(unions) = unions_map.get(&object.name) {
                    compatible_types.extend(unions);
                }
                compatible_types.extend(
                    object
                        .implements_interfaces
                        .iter()
                        .map(|interface| &interface.name),
                );
            };
            match type_def {
                schema::ExtendedType::Scalar(_) | schema::ExtendedType::Enum(_) => {}
                schema::ExtendedType::Object(object) => add_object_type_and_its_supertypes(object),
                schema::ExtendedType::Interface(_) => {
                    if let Some(implementers) = implementers_map.get(name) {
                        for object_name in &implementers.objects {
                            let object = schema
                                .get_object(object_name)
                                .expect("implementers_map refers to undefined object");
                            add_object_type_and_its_supertypes(object)
                        }
                    }
                }
                schema::ExtendedType::Union(union_) => {
                    for object_name in &union_.members {
                        let object = schema
                            .get_object(object_name)
                            .expect("union has undefined object type as a member");
                        add_object_type_and_its_supertypes(object)
                    }
                }
                schema::ExtendedType::InputObject(_) => unreachable!(),
            }
            (name, compatible_types)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::arbitrary_valid_executable_document;
    use super::gather_compatible_types;
    use crate::arbitrary::common::tests::arbitrary_bytes;
    use crate::Schema;
    use expect_test::expect;
    use std::fmt::Write;

    fn format_compatible_types(schema: &str) -> String {
        let schema = Schema::parse_and_validate(schema, "").unwrap();
        let mut formatted = String::new();
        for (name, others) in gather_compatible_types(&schema)
            .into_iter()
            // For deterministic ordering:
            .map(|(k, v)| (k, v.into_iter().collect::<std::collections::BTreeSet<_>>()))
            .collect::<std::collections::BTreeMap<_, _>>()
        {
            writeln!(&mut formatted, "{name}: {others:?}",).unwrap();
        }
        formatted
    }

    #[test]
    fn compatible_types() {
        let expected = expect![[r#"
            Alien: {"Alien", "HumanOrAlien", "Sentient"}
            Cat: {"Cat", "CatOrDog", "Pet"}
            CatOrDog: {"Cat", "CatOrDog", "Dog", "DogOrHuman", "Pet"}
            Dog: {"CatOrDog", "Dog", "DogOrHuman", "Pet"}
            DogOrHuman: {"CatOrDog", "Dog", "DogOrHuman", "Human", "HumanOrAlien", "Pet", "Sentient"}
            Human: {"DogOrHuman", "Human", "HumanOrAlien", "Sentient"}
            HumanOrAlien: {"Alien", "DogOrHuman", "Human", "HumanOrAlien", "Sentient"}
            Pet: {"Cat", "CatOrDog", "Dog", "DogOrHuman", "Pet"}
            Query: {"Query"}
            Sentient: {"Alien", "DogOrHuman", "Human", "HumanOrAlien", "Sentient"}
            __Directive: {"__Directive"}
            __EnumValue: {"__EnumValue"}
            __Field: {"__Field"}
            __InputValue: {"__InputValue"}
            __Schema: {"__Schema"}
            __Type: {"__Type"}
        "#]];
        let schema = include_str!("../../examples/documents/schema.graphql");
        expected.assert_eq(&format_compatible_types(schema));
    }

    #[test]
    fn executable_document() {
        let schema = include_str!("../../benches/testdata/supergraph.graphql");
        let schema = Schema::parse_and_validate(schema, "").unwrap();

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(0, 0));
        expect![[r#"
            {
              field0: __typename
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(1, 1));
        expect![[r#"
            mutation {
              field0: __typename
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(2, 2));
        expect![[r#"
            query A {
              field0: __typename
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(3, 4));
        expect![[r#"
            {
              ... on Query {
                field0: __typename
              }
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(4, 8));
        expect![[r#"
            mutation H {
              field0: __typename
              field1: __typename
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(4, 16));
        expect![[r#"
            mutation H {
              field0: __typename
              field1: login(username: "NIm", password: "A") {
                field2: __typename
              }
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(5, 16));
        expect![[r#"
            mutation X {
              ...Frag0
              field1: updateReview(review: {id: 0}) {
                field2: __typename
              }
            }

            fragment Frag0 on Mutation {
              ... {
                field0: __typename
              }
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(6, 16));
        expect![[r#"
            {
              field0: product(upc: "vTQ") @include(if: false) {
                field1: __typename
              }
            }
        "#]]
        .assert_eq(&doc.to_string());

        let doc = arbitrary_valid_executable_document(&schema, &arbitrary_bytes(6, 100));
        expect![[r#"
            query($var0: String!, $var1: Int, $var2: Boolean! = false) {
              field0: product(upc: "vTQ") @include(if: false) {
                ... on Product {
                  field1: name @transform(from: $var0)
                  ... on Furniture {
                    ... {
                      ...Frag0
                    }
                  }
                }
                field5: __typename
              }
              field2: topCars(first: $var1) {
                ... on Thing {
                  field3: __typename
                }
                field6: retailPrice
              }
              field7: me {
                ...Frag1
              }
            }

            fragment Frag0 on Product {
              ... {
                ... {
                  ... @skip(if: $var2) {
                    ... {
                      field4: __typename
                    }
                  }
                }
              }
            }

            fragment Frag1 on User {
              ... on User @skip(if: false) {
                field8: __typename
              }
            }
        "#]]
        .assert_eq(&doc.to_string());

        // Generate a bunch more just to check generation completes
        // without panic or stack overflow, and returns something valid
        for seed in 1000..2000 {
            arbitrary_valid_executable_document(&schema, &arbitrary_bytes(seed, 100));
        }
    }
}
