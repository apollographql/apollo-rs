use apollo_compiler::ast::{Definition, InputObjectTypeDefinition, Name, Type};
use apollo_compiler::schema::ExtendedType;
use arbitrary::Unstructured;

pub(crate) trait DefinitionExt {
    fn ty(&self, u: &mut Unstructured) -> arbitrary::Result<Type>;
}

impl DefinitionExt for Definition {
    fn ty(&self, u: &mut Unstructured) -> arbitrary::Result<Type> {
        let name = self.name().expect("definition must have a name").clone();
        Ok(ty(u, name)?)
    }
}

impl DefinitionExt for InputObjectTypeDefinition {
    fn ty(&self, u: &mut Unstructured) -> arbitrary::Result<Type> {
        Ok(ty(u, self.name.clone())?)
    }
}

fn ty(u: &mut Unstructured, name: Name) -> arbitrary::Result<Type> {
    let mut ty = if u.arbitrary()? {
        Type::Named(name)
    } else {
        Type::NonNullNamed(name)
    };

    for _ in 0..u.int_in_range(0..=5)? {
        if u.arbitrary()? {
            ty = Type::List(Box::new(ty))
        } else {
            ty = Type::NonNullList(Box::new(ty))
        };
    }
    Ok(ty)
}

#[derive(Debug)]
pub(crate) enum DefinitionKind {
    OperationDefinition,
    FragmentDefinition,
    DirectiveDefinition,
    SchemaDefinition,
    ScalarTypeDefinition,
    ObjectTypeDefinition,
    InterfaceTypeDefinition,
    UnionTypeDefinition,
    EnumTypeDefinition,
    InputObjectTypeDefinition,
    SchemaExtension,
    ScalarTypeExtension,
    ObjectTypeExtension,
    InterfaceTypeExtension,
    UnionTypeExtension,
    EnumTypeExtension,
    InputObjectTypeExtension,
}

impl DefinitionKind {
    pub(crate) fn matches(&self, definition: &Definition) -> bool {
        match (self, definition) {
            (Self::OperationDefinition, Definition::OperationDefinition(_)) => true,
            (Self::FragmentDefinition, Definition::FragmentDefinition(_)) => true,
            (Self::DirectiveDefinition, Definition::DirectiveDefinition(_)) => true,
            (Self::SchemaDefinition, Definition::SchemaDefinition(_)) => true,
            (Self::ScalarTypeDefinition, Definition::ScalarTypeDefinition(_)) => true,
            (Self::ObjectTypeDefinition, Definition::ObjectTypeDefinition(_)) => true,
            (Self::InterfaceTypeDefinition, Definition::InterfaceTypeDefinition(_)) => true,
            (Self::UnionTypeDefinition, Definition::UnionTypeDefinition(_)) => true,
            (Self::EnumTypeDefinition, Definition::EnumTypeDefinition(_)) => true,
            (Self::InputObjectTypeDefinition, Definition::InputObjectTypeDefinition(_)) => true,
            (Self::SchemaExtension, Definition::SchemaExtension(_)) => true,
            (Self::ScalarTypeExtension, Definition::ScalarTypeExtension(_)) => true,
            (Self::ObjectTypeExtension, Definition::ObjectTypeExtension(_)) => true,
            (Self::InterfaceTypeExtension, Definition::InterfaceTypeExtension(_)) => true,
            (Self::UnionTypeExtension, Definition::UnionTypeExtension(_)) => true,
            (Self::EnumTypeExtension, Definition::EnumTypeExtension(_)) => true,
            (Self::InputObjectTypeExtension, Definition::InputObjectTypeExtension(_)) => true,
            _ => false,
        }
    }
}
