use super::executable::*;
use super::type_system::*;
use super::Component;
use super::ExtensionId;
use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::OperationType;
use apollo_parser::mir::Ranged;
use indexmap::IndexMap;

impl Schema {
    pub(super) fn to_mir(&self) -> impl Iterator<Item = mir::Definition> + '_ {
        let root_ops = |ext: Option<&ExtensionId>| -> Vec<(OperationType, Name)> {
            let root_op = |name: &Option<Name>, name_ext: &Option<ExtensionId>, ty| {
                name.as_ref()
                    .filter(|_| name_ext.as_ref() == ext)
                    .cloned()
                    .map(|name| (ty, name))
                    .into_iter()
            };
            root_op(&self.query, &self.query_extension, OperationType::Query)
                .chain(root_op(
                    &self.mutation,
                    &self.mutation_extension,
                    OperationType::Mutation,
                ))
                .chain(root_op(
                    &self.subscription,
                    &self.subscription_extension,
                    OperationType::Subscription,
                ))
                .collect()
        };
        let extensions = self.extensions();
        let implicit = self.description.is_none()
            && self.directives.is_empty()
            && extensions.is_empty()
            && [None, Some(OperationType::Query.name())].contains(&self.query.as_deref())
            && [None, Some(OperationType::Mutation.name())].contains(&self.mutation.as_deref())
            && [None, Some(OperationType::Subscription.name())]
                .contains(&self.subscription.as_deref());
        if implicit {
            None
        } else {
            Some(mir::Definition::SchemaDefinition(Harc::new(
                Ranged::no_location(mir::SchemaDefinition {
                    description: self.description.clone(),
                    directives: components(&self.directives, None),
                    root_operations: root_ops(None),
                }),
            )))
        }
        .into_iter()
        .chain(extensions.into_iter().map(move |ext| {
            mir::Definition::SchemaExtension(Harc::new(Ranged::no_location(mir::SchemaExtension {
                directives: components(&self.directives, Some(ext)),
                root_operations: root_ops(Some(ext)),
            })))
        }))
    }
}

impl ScalarType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::ScalarTypeDefinition(Harc::new(
            Ranged::no_location(mir::ScalarTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::ScalarTypeExtension(Harc::new(Ranged::no_location(
                mir::ScalarTypeExtension {
                    name: name.clone(),
                    directives: components(&self.directives, Some(ext)),
                },
            )))
        }))
    }
}

impl ObjectType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::ObjectTypeDefinition(Harc::new(
            Ranged::no_location(mir::ObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::ObjectTypeExtension(Harc::new(Ranged::no_location(
                mir::ObjectTypeExtension {
                    name: name.clone(),
                    implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                    directives: components(&self.directives, Some(ext)),
                    fields: components(self.fields.values(), Some(ext)),
                },
            )))
        }))
    }
}

impl InterfaceType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::InterfaceTypeDefinition(Harc::new(
            Ranged::no_location(mir::InterfaceTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::InterfaceTypeExtension(Harc::new(Ranged::no_location(
                mir::InterfaceTypeExtension {
                    name: name.clone(),
                    implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                    directives: components(&self.directives, Some(ext)),
                    fields: components(self.fields.values(), Some(ext)),
                },
            )))
        }))
    }
}

impl UnionType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::UnionTypeDefinition(Harc::new(
            Ranged::no_location(mir::UnionTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                members: names(&self.members, None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::UnionTypeExtension(Harc::new(Ranged::no_location(
                mir::UnionTypeExtension {
                    name: name.clone(),
                    directives: components(&self.directives, Some(ext)),
                    members: names(&self.members, Some(ext)),
                },
            )))
        }))
    }
}

impl EnumType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::EnumTypeDefinition(Harc::new(
            Ranged::no_location(mir::EnumTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                values: components(self.values.values(), None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::EnumTypeExtension(Harc::new(Ranged::no_location(
                mir::EnumTypeExtension {
                    name: name.clone(),
                    directives: components(&self.directives, Some(ext)),
                    values: components(self.values.values(), Some(ext)),
                },
            )))
        }))
    }
}

impl InputObjectType {
    pub(super) fn to_mir<'a>(
        &'a self,
        name: &'a Name,
    ) -> impl Iterator<Item = mir::Definition> + 'a {
        std::iter::once(mir::Definition::InputObjectTypeDefinition(Harc::new(
            Ranged::no_location(mir::InputObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            }),
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            mir::Definition::InputObjectTypeExtension(Harc::new(Ranged::no_location(
                mir::InputObjectTypeExtension {
                    name: name.clone(),
                    directives: components(&self.directives, Some(ext)),
                    fields: components(self.fields.values(), Some(ext)),
                },
            )))
        }))
    }
}

fn components<'a, T: 'a>(
    components: impl IntoIterator<Item = &'a Component<T>>,
    ext: Option<&ExtensionId>,
) -> Vec<Harc<Ranged<T>>> {
    components
        .into_iter()
        .filter(|def| def.extension_id() == ext)
        .map(|def| def.ranged().clone())
        .collect()
}

fn names(names: &IndexMap<Name, Option<ExtensionId>>, ext: Option<&ExtensionId>) -> Vec<Name> {
    names
        .iter()
        .filter(|(_k, v)| v.as_ref() == ext)
        .map(|(k, _v)| k.clone())
        .collect()
}

impl Operation {
    pub(super) fn to_mir(&self, name: Option<&Name>) -> mir::Definition {
        mir::Definition::OperationDefinition(Harc::new(Ranged::no_location(
            mir::OperationDefinition {
                operation_type: self.operation_type,
                name: name.cloned(),
                variables: self.variables.clone(),
                directives: self.directives.clone(),
                selection_set: self.selection_set.to_mir(),
            },
        )))
    }
}

impl Fragment {
    pub(super) fn to_mir(&self, name: &Name) -> mir::Definition {
        mir::Definition::FragmentDefinition(Harc::new(Ranged::no_location(
            mir::FragmentDefinition {
                name: name.clone(),
                type_condition: self.type_condition.clone(),
                directives: self.directives.clone(),
                selection_set: self.selection_set.to_mir(),
            },
        )))
    }
}

impl SelectionSet {
    fn to_mir(&self) -> Vec<mir::Selection> {
        self.selections()
            .iter()
            .map(|selection| match selection {
                Selection::Field(field) => {
                    mir::Selection::Field(Harc::new(Ranged::no_location(mir::Field {
                        alias: field.alias.clone(),
                        name: field.name.clone(),
                        arguments: field.arguments.clone(),
                        directives: field.directives.clone(),
                        selection_set: field.selection_set.to_mir(),
                    })))
                }
                Selection::FragmentSpread(fragment_spread) => mir::Selection::FragmentSpread(
                    Harc::new(Ranged::no_location(mir::FragmentSpread {
                        fragment_name: fragment_spread.fragment_name.clone(),
                        directives: fragment_spread.directives.clone(),
                    })),
                ),
                Selection::InlineFragment(inline_fragment) => mir::Selection::InlineFragment(
                    Harc::new(Ranged::no_location(mir::InlineFragment {
                        type_condition: inline_fragment.type_condition.clone(),
                        directives: inline_fragment.directives.clone(),
                        selection_set: inline_fragment.selection_set.to_mir(),
                    })),
                ),
            })
            .collect()
    }
}

#[test]
fn test_type_system_reserialization() {
    use crate::HirDatabase;

    let input = r#"
        extend type Query {
            withArg(arg: Boolean): String @deprecated,
        }

        type Query {
            int: Int,
        }

        extend type implements Inter

        interface Inter {
            string: String
        }

        extend type Query @customDirective;

        extend type Query {
            string: String,
        }

        directive @customDirective on OBJECT;
    "#;
    // Order is mostly not preserved
    let expected = r#"directive @customDirective on OBJECT

type Query {
  int: Int
}

extend type Query {
  withArg(arg: Boolean): String @deprecated
}

extend type Query {
  string: String
}

interface Inter {
  string: String
}
"#;
    let mut compiler = crate::ApolloCompiler::new();
    compiler.add_type_system(input, "");
    let type_system = compiler.db.type_system2();
    assert_eq!(type_system.to_string(), expected);
}
