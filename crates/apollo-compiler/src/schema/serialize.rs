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
        self.schema_definition
            .to_ast(&self.types)
            .chain(
                self.directive_definitions
                    .values()
                    .filter(|def| !def.is_built_in())
                    .map(|def| ast::Definition::DirectiveDefinition(def.clone())),
            )
            .chain(self.types.values().flat_map(|def| {
                let mut iter = def.to_ast();
                // skip the definition of built-in scalars but keep extensions if any
                if def.is_built_in() {
                    iter.next();
                }
                iter
            }))
    }
}

impl Node<SchemaDefinition> {
    fn to_ast(
        &self,
        types: &IndexMap<Name, ExtendedType>,
    ) -> impl Iterator<Item = ast::Definition> + '_ {
        let SchemaDefinition {
            description,
            directives,
            query,
            mutation,
            subscription,
        } = &**self;
        let extensions = self.extensions();
        let implict = description.is_none()
            && directives.is_empty()
            && extensions.is_empty()
            && [
                (query, ast::OperationType::Query),
                (mutation, ast::OperationType::Mutation),
                (subscription, ast::OperationType::Subscription),
            ]
            .into_iter()
            .all(|(root_operation, operation_type)| {
                // If there were no explict `schema` definition,
                // what implicit root operation would we get for this operation type?
                let default_type_name = operation_type.default_type_name();
                let implicit_root_operation: Option<&str> = types
                    .get(default_type_name)
                    .filter(|ty_def| ty_def.is_object())
                    .map(|_ty_def| default_type_name);
                // What we have
                let actual_root_operation = root_operation.as_ref().map(|r| r.as_str());
                // Only allow an implicit `schema` definition if they match
                actual_root_operation == implicit_root_operation
            })
            // Hack: if there is *nothing*, still emit an empty SchemaDefinition AST node
            // that carries a location, so AST-based validation can emit an error
            // with `DiagnosticData::QueryRootOperationType`.
            // This can be removed after that validation rule is ported to high-level `Schema`.
            && [query, mutation, subscription]
                .into_iter()
                .any(|op| op.is_some());
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
        if implict {
            None
        } else {
            Some(ast::Definition::SchemaDefinition(self.same_location(
                ast::SchemaDefinition {
                    description: self.description.clone(),
                    directives: ast::Directives(components(&self.directives, None)),
                    root_operations: root_ops(None),
                },
            )))
        }
        .into_iter()
        .chain(extensions.into_iter().map(move |ext| {
            ast::Definition::SchemaExtension(ext.same_location(ast::SchemaExtension {
                directives: ast::Directives(components(&self.directives, Some(ext))),
                root_operations: root_ops(Some(ext)),
            }))
        }))
    }
}

impl ExtendedType {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        match self {
            ExtendedType::Scalar(ty) => Box::new(ty.to_ast()) as Box<dyn Iterator<Item = _>>,
            ExtendedType::Object(ty) => Box::new(ty.to_ast()) as _,
            ExtendedType::Interface(ty) => Box::new(ty.to_ast()) as _,
            ExtendedType::Union(ty) => Box::new(ty.to_ast()) as _,
            ExtendedType::Enum(ty) => Box::new(ty.to_ast()) as _,
            ExtendedType::InputObject(ty) => Box::new(ty.to_ast()) as _,
        }
    }
}

impl Node<ScalarType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ScalarTypeDefinition(self.same_location(
            ast::ScalarTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ScalarTypeExtension(ext.same_location(ast::ScalarTypeExtension {
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
            }))
        }))
    }
}

impl Node<ObjectType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::ObjectTypeDefinition(self.same_location(
            ast::ObjectTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::ObjectTypeExtension(ext.same_location(ast::ObjectTypeExtension {
                name: self.name.clone(),
                implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                fields: components(self.fields.values(), Some(ext)),
            }))
        }))
    }
}

impl Node<InterfaceType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InterfaceTypeDefinition(
            self.same_location(ast::InterfaceTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                implements_interfaces: names(&self.implements_interfaces, None),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InterfaceTypeExtension(ext.same_location(
                ast::InterfaceTypeExtension {
                    name: self.name.clone(),
                    implements_interfaces: names(&self.implements_interfaces, Some(ext)),
                    directives: ast::Directives(components(&self.directives, Some(ext))),
                    fields: components(self.fields.values(), Some(ext)),
                },
            ))
        }))
    }
}

impl Node<UnionType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::UnionTypeDefinition(self.same_location(
            ast::UnionTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                members: names(&self.members, None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::UnionTypeExtension(ext.same_location(ast::UnionTypeExtension {
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                members: names(&self.members, Some(ext)),
            }))
        }))
    }
}

impl Node<EnumType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::EnumTypeDefinition(self.same_location(
            ast::EnumTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                values: components(self.values.values(), None),
            },
        )))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::EnumTypeExtension(ext.same_location(ast::EnumTypeExtension {
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, Some(ext))),
                values: components(self.values.values(), Some(ext)),
            }))
        }))
    }
}

impl Node<InputObjectType> {
    fn to_ast<'a>(&'a self) -> impl Iterator<Item = ast::Definition> + 'a {
        std::iter::once(ast::Definition::InputObjectTypeDefinition(
            self.same_location(ast::InputObjectTypeDefinition {
                description: self.description.clone(),
                name: self.name.clone(),
                directives: ast::Directives(components(&self.directives, None)),
                fields: components(self.fields.values(), None),
            }),
        ))
        .chain(self.extensions().into_iter().map(move |ext| {
            ast::Definition::InputObjectTypeExtension(ext.same_location(
                ast::InputObjectTypeExtension {
                    name: self.name.clone(),
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
