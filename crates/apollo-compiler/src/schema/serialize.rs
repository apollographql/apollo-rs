use super::*;
use crate::ast::serialize::top_level;
use crate::ast::serialize::State;
use crate::ast::OperationType;
use std::fmt;

impl Schema {
    pub(crate) fn serialize_impl(&self, state: &mut State) -> fmt::Result {
        // TODO: avoid allocating temporary AST nodes?
        // it would ~duplicate large parts of ast/serialize.rs
        top_level(state, self.to_ast(), |state, def| def.serialize_impl(state))
    }
}

impl Schema {
    fn to_ast(&self) -> impl Iterator<Item = ast::Definition> + '_ {
        let implicit_op = |opt: &Option<ComponentStr>, ty: OperationType| match opt {
            Some(name) => name.as_str() == ty.name(),
            None => true,
        };
        let schema_extensions = self.extensions();
        let implicit_schema = self.description.is_none()
            && self.directives.is_empty()
            && schema_extensions.is_empty()
            && implicit_op(&self.query_type, OperationType::Query)
            && implicit_op(&self.mutation_type, OperationType::Mutation)
            && implicit_op(&self.subscription_type, OperationType::Subscription);
        let root_ops = |ext: Option<&ExtensionId>| -> Vec<(OperationType, Name)> {
            let root_op = |op: &Option<ComponentStr>, ty| {
                op.as_ref()
                    .filter(|name| name.origin.extension_id() == ext)
                    .map(|name| (ty, name.node.clone()))
                    .into_iter()
            };
            root_op(&self.query_type, OperationType::Query)
                .chain(root_op(&self.mutation_type, OperationType::Mutation))
                .chain(root_op(
                    &self.subscription_type,
                    OperationType::Subscription,
                ))
                .collect()
        };
        if implicit_schema {
            None
        } else {
            Some(ast::Definition::SchemaDefinition(Node::new_synthetic(
                ast::SchemaDefinition {
                    description: self.description.clone(),
                    directives: components(&self.directives, None),
                    root_operations: root_ops(None),
                },
            )))
        }
        .into_iter()
        .chain(schema_extensions.into_iter().map(move |ext| {
            ast::Definition::SchemaExtension(Node::new_synthetic(ast::SchemaExtension {
                directives: components(&self.directives, Some(ext)),
                root_operations: root_ops(Some(ext)),
            }))
        }))
        .chain(
            self.directive_definitions
                .values()
                .filter(|def| !def.is_built_in())
                .map(|def| ast::Definition::DirectiveDefinition(def.clone())),
        )
        .chain(
            self.types
                .iter()
                .filter(|(_name, def)| !def.is_built_in())
                .flat_map(|(name, def)| def.to_ast(name)),
        )
    }
}

impl Type {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        match self {
            Type::Scalar(ty) => Box::new(ty.to_ast(name)) as Box<dyn Iterator<Item = _>>,
            Type::Object(ty) => Box::new(ty.to_ast(name)) as _,
            Type::Interface(ty) => Box::new(ty.to_ast(name)) as _,
            Type::Union(ty) => Box::new(ty.to_ast(name)) as _,
            Type::Enum(ty) => Box::new(ty.to_ast(name)) as _,
            Type::InputObject(ty) => Box::new(ty.to_ast(name)) as _,
        }
    }
}

impl ScalarType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ScalarTypeDefinition(Node::new_synthetic(
            ast::ScalarTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ScalarTypeExtension(Node::new_synthetic(ast::ScalarTypeExtension {
                name: name.clone(),
                directives: components(&self.directives, Some(ext)),
            }))
        }))
    }
}

impl ObjectType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ObjectTypeDefinition(Node::new_synthetic(
            ast::ObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ObjectTypeExtension(Node::new_synthetic(ast::ObjectTypeExtension {
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                directives: components(&self.directives, Some(ext)),
                fields: components(self.fields.values(), Some(ext)),
            }))
        }))
    }
}

impl InterfaceType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InterfaceTypeDefinition(
            Node::new_synthetic(ast::InterfaceTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InterfaceTypeExtension(Node::new_synthetic(
                ast::InterfaceTypeExtension {
                    name: name.clone(),
                    implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                    directives: components(&self.directives, Some(ext)),
                    fields: components(self.fields.values(), Some(ext)),
                },
            ))
        }))
    }
}

impl UnionType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::UnionTypeDefinition(Node::new_synthetic(
            ast::UnionTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                members: names(&self.members, None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::UnionTypeExtension(Node::new_synthetic(ast::UnionTypeExtension {
                name: name.clone(),
                directives: components(&self.directives, Some(ext)),
                members: names(&self.members, Some(ext)),
            }))
        }))
    }
}

impl EnumType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::EnumTypeDefinition(Node::new_synthetic(
            ast::EnumTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                values: components(self.values.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::EnumTypeExtension(Node::new_synthetic(ast::EnumTypeExtension {
                name: name.clone(),
                directives: components(&self.directives, Some(ext)),
                values: components(self.values.values(), Some(ext)),
            }))
        }))
    }
}

impl InputObjectType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InputObjectTypeDefinition(
            Node::new_synthetic(ast::InputObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: components(&self.directives, None),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InputObjectTypeExtension(Node::new_synthetic(
                ast::InputObjectTypeExtension {
                    name: name.clone(),
                    directives: components(&self.directives, Some(ext)),
                    fields: components(self.fields.values(), Some(ext)),
                },
            ))
        }))
    }
}

fn components<'a, T: 'a>(
    components: impl IntoIterator<Item = &'a Component<T>>,
    ext: Option<&ExtensionId>,
) -> Vec<Node<T>> {
    components
        .into_iter()
        .filter(|def| def.origin.extension_id() == ext)
        .map(|def| def.node.clone())
        .collect()
}

fn names(names: &IndexMap<Name, ComponentOrigin>, ext: Option<&ExtensionId>) -> Vec<Name> {
    names
        .iter()
        .filter(|(_name, origin)| origin.extension_id() == ext)
        .map(|(name, _origin)| name.clone())
        .collect()
}

#[test]
fn test_schema_reserialize() {
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
    let (schema, _) = Schema::from_ast(&ast::Document::parse(input).document);
    assert_eq!(schema.to_string(), expected);
}
