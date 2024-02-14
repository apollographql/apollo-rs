use apollo_compiler::ast::{Definition, Name, Type};
use apollo_compiler::schema::ExtendedType;
use arbitrary::Unstructured;

pub(crate) trait ExtendedTypeExt {
    fn ty(&self, u: &mut Unstructured) -> arbitrary::Result<Type>;
}

impl ExtendedTypeExt for ExtendedType {
    fn ty(&self, u: &mut Unstructured) -> arbitrary::Result<Type> {
        Ok(ty(u, self.name().clone())?)
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
pub(crate) enum ExtendedTypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObjectTypeDefinition,
}

impl ExtendedTypeKind {
    pub(crate) fn matches(&self, definition: &ExtendedType) -> bool {
        match (self, definition) {
            (Self::Scalar, ExtendedType::Scalar(_)) => true,
            (Self::Object, ExtendedType::Object(_)) => true,
            (Self::Interface, ExtendedType::Interface(_)) => true,
            (Self::Union, ExtendedType::Union(_)) => true,
            (Self::Enum, ExtendedType::Enum(_)) => true,
            (Self::InputObjectTypeDefinition, ExtendedType::InputObject(_)) => true,
            _ => false,
        }
    }
}
