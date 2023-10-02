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
    pub(crate) fn to_ast(&self) -> impl Iterator<Item = ast::Definition> + '_ {
        self.root_operations
            .as_ref()
            .into_iter()
            .flat_map(|root| root.to_ast())
            .chain(
                self.directive_definitions
                    .values()
                    .filter(|def| !def.is_built_in())
                    .map(|def| ast::Definition::DirectiveDefinition(def.clone())),
            )
            .chain(self.types.iter().flat_map(|(name, def)| {
                let mut iter = def.to_ast(name);
                // skip the definition of built-in scalars but keep extensions if any
                if def.is_built_in() {
                    iter.next();
                }
                iter
            }))
    }
}

impl Node<RootOperations> {
    fn to_ast(&self) -> impl Iterator<Item = ast::Definition> + '_ {
        let root_ops = |ext: Option<&ExtensionId>| -> Vec<Node<(OperationType, Name)>> {
            let root_op = |op: &Option<ComponentStr>, ty| {
                op.as_ref()
                    .filter(|name| name.origin.extension_id() == ext)
                    .map(|name| (ty, name.node.clone()).into())
                    .into_iter()
            };
            root_op(&self.query, OperationType::Query)
                .chain(root_op(&self.mutation, OperationType::Mutation))
                .chain(root_op(&self.subscription, OperationType::Subscription))
                .collect()
        };
        std::iter::once(ast::Definition::SchemaDefinition(self.same_location(
            ast::SchemaDefinition {
                description: self.description.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                root_operations: root_ops(None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::SchemaExtension(ext.same_location(ast::SchemaExtension {
                directives: ast::Directives(components(&self.directives, Some(ext))),
                root_operations: root_ops(Some(ext)),
            }))
        }))
    }
}

impl ExtendedType {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        match self {
            ExtendedType::Scalar(ty) => Box::new(ty.to_ast(name)) as Box<dyn Iterator<Item = _>>,
            ExtendedType::Object(ty) => Box::new(ty.to_ast(name)) as _,
            ExtendedType::Interface(ty) => Box::new(ty.to_ast(name)) as _,
            ExtendedType::Union(ty) => Box::new(ty.to_ast(name)) as _,
            ExtendedType::Enum(ty) => Box::new(ty.to_ast(name)) as _,
            ExtendedType::InputObject(ty) => Box::new(ty.to_ast(name)) as _,
        }
    }
}

impl Node<ScalarType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ScalarTypeDefinition(self.same_location(
            ast::ScalarTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ScalarTypeExtension(ext.same_location(ast::ScalarTypeExtension {
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
            }))
        }))
    }
}

impl Node<ObjectType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ObjectTypeDefinition(self.same_location(
            ast::ObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ObjectTypeExtension(ext.same_location(ast::ObjectTypeExtension {
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                fields: components(self.fields.values(), Some(ext)),
            }))
        }))
    }
}

impl Node<InterfaceType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InterfaceTypeDefinition(
            self.same_location(ast::InterfaceTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InterfaceTypeExtension(ext.same_location(
                ast::InterfaceTypeExtension {
                    name: name.clone(),
                    implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                    directives: ast::Directives(components(&self.directives, Some(ext))),
                    fields: components(self.fields.values(), Some(ext)),
                },
            ))
        }))
    }
}

impl Node<UnionType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::UnionTypeDefinition(self.same_location(
            ast::UnionTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                members: names(&self.members, None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::UnionTypeExtension(ext.same_location(ast::UnionTypeExtension {
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                members: names(&self.members, Some(ext)),
            }))
        }))
    }
}

impl Node<EnumType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::EnumTypeDefinition(self.same_location(
            ast::EnumTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                values: components(self.values.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::EnumTypeExtension(ext.same_location(ast::EnumTypeExtension {
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                values: components(self.values.values(), Some(ext)),
            }))
        }))
    }
}

impl Node<InputObjectType> {
    fn to_ast<'a>(&'a self, name: &'a Name) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InputObjectTypeDefinition(
            self.same_location(ast::InputObjectTypeDefinition {
                description: self.description.clone(),
                name: name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InputObjectTypeExtension(ext.same_location(
                ast::InputObjectTypeExtension {
                    name: name.clone(),
                    directives: ast::Directives(components(&self.directives, Some(ext))),
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

fn names(names: &IndexSet<ComponentStr>, ext: Option<&ExtensionId>) -> Vec<Name> {
    names
        .iter()
        .filter(|component| component.origin.extension_id() == ext)
        .map(|component| component.node.clone())
        .collect()
}
