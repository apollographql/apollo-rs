use apollo_encoder::Type_;
use arbitrary::Result as ArbitraryResult;
use once_cell::sync::Lazy;

use crate::{input_value::InputValue, name::Name, DocumentBuilder};

static BUILTIN_SCALAR_NAMES: Lazy<[Ty; 5]> = Lazy::new(|| {
    [
        Ty::Named(Name::new(String::from("Int"))),
        Ty::Named(Name::new(String::from("Float"))),
        Ty::Named(Name::new(String::from("String"))),
        Ty::Named(Name::new(String::from("Boolean"))),
        Ty::Named(Name::new(String::from("ID"))),
    ]
});

/// Convenience Type_ implementation used when creating a Field.
/// Can be a `NamedType`, a `NonNull` or a `List`.
///
/// This enum is resposible for encoding creating values such as `String!`, `[[[[String]!]!]!]!`, etc.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Ty {
    /// The Non-Null field type.
    Named(Name),
    /// The List field type.
    List(Box<Ty>),
    /// The Named field type.
    NonNull(Box<Ty>),
}

impl From<Ty> for Type_ {
    fn from(val: Ty) -> Self {
        match val {
            Ty::Named(name) => Type_::NamedType { name: name.into() },
            Ty::List(ty) => Type_::List {
                ty: Box::new((*ty).into()),
            },
            Ty::NonNull(ty) => Type_::NonNull {
                ty: Box::new((*ty).into()),
            },
        }
    }
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::Type> for Ty {
    fn from(ty: apollo_parser::ast::Type) -> Self {
        match ty {
            apollo_parser::ast::Type::NamedType(named_ty) => named_ty.into(),
            apollo_parser::ast::Type::ListType(list_type) => {
                Self::List(Box::new(list_type.ty().unwrap().into()))
            }
            apollo_parser::ast::Type::NonNullType(non_null) => {
                if let Some(named_ty) = non_null.named_type() {
                    Self::NonNull(Box::new(named_ty.into()))
                } else if let Some(list_type) = non_null.list_type() {
                    Self::NonNull(Box::new(Self::List(Box::new(
                        list_type.ty().unwrap().into(),
                    ))))
                } else {
                    panic!("a non null type must have a type")
                }
            }
        }
    }
}

#[cfg(feature = "parser-impl")]
impl From<apollo_parser::ast::NamedType> for Ty {
    fn from(ty: apollo_parser::ast::NamedType) -> Self {
        Self::Named(ty.name().unwrap().into())
    }
}

impl Ty {
    pub(crate) fn name(&self) -> &Name {
        match self {
            Ty::Named(name) => name,
            Ty::List(list) => list.name(),
            Ty::NonNull(non_null) => non_null.name(),
        }
    }

    /// Returns `true` if the ty is [`Named`].
    ///
    /// [`Named`]: Ty::Named
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named(..))
    }

    pub(crate) fn is_builtin(&self) -> bool {
        BUILTIN_SCALAR_NAMES.contains(&Ty::Named(self.name().clone()))
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `Ty`
    pub fn ty(&mut self) -> ArbitraryResult<Ty> {
        self.generate_ty(true)
    }

    /// Choose an arbitrary existing `Ty` given a slice of existing types
    pub fn choose_ty(&mut self, existing_types: &[Ty]) -> ArbitraryResult<Ty> {
        self.choose_ty_given_nullable(existing_types, true)
    }

    /// Choose an arbitrary existing named `Ty` given a slice of existing types
    pub fn choose_named_ty(&mut self, existing_types: &[Ty]) -> ArbitraryResult<Ty> {
        let used_type_names: Vec<&Ty> = existing_types
            .iter()
            .chain(BUILTIN_SCALAR_NAMES.iter())
            .collect();

        Ok(self.u.choose(&used_type_names)?.to_owned().clone())
    }

    fn choose_ty_given_nullable(
        &mut self,
        existing_types: &[Ty],
        is_nullable: bool,
    ) -> ArbitraryResult<Ty> {
        let ty: Ty = match self.u.int_in_range(0..=2usize)? {
            // Named type
            0 => {
                let used_type_names: Vec<&Ty> = existing_types
                    .iter()
                    .chain(BUILTIN_SCALAR_NAMES.iter())
                    .collect();

                self.u.choose(&used_type_names)?.to_owned().clone()
            }
            // List type
            1 => Ty::List(Box::new(
                self.choose_ty_given_nullable(existing_types, true)?,
            )),
            // Non Null type
            2 => {
                if is_nullable {
                    Ty::NonNull(Box::new(
                        self.choose_ty_given_nullable(existing_types, false)?,
                    ))
                } else {
                    self.choose_ty_given_nullable(existing_types, is_nullable)?
                }
            }
            _ => unreachable!(),
        };

        Ok(ty)
    }

    fn generate_ty(&mut self, is_nullable: bool) -> ArbitraryResult<Ty> {
        let ty = match self.u.int_in_range(0..=2usize)? {
            // Named type
            0 => Ty::Named(self.name()?),
            // List type
            1 => Ty::List(Box::new(self.generate_ty(true)?)),
            // Non Null type
            2 => {
                if is_nullable {
                    Ty::NonNull(Box::new(self.generate_ty(false)?))
                } else {
                    self.generate_ty(is_nullable)?
                }
            }
            _ => unreachable!(),
        };

        Ok(ty)
    }

    /// List all existing (already created) `Ty`
    pub(crate) fn list_existing_types(&self) -> Vec<Ty> {
        self.object_type_defs
            .iter()
            .map(|o| Ty::Named(o.name.clone()))
            .chain(
                self.enum_type_defs
                    .iter()
                    .map(|e| Ty::Named(e.name.clone())),
            )
            .collect()
    }

    /// List all existing object (already created) `Ty`
    pub(crate) fn list_existing_object_types(&self) -> Vec<Ty> {
        self.object_type_defs
            .iter()
            .map(|o| Ty::Named(o.name.clone()))
            .collect()
    }

    #[allow(dead_code)]
    pub(crate) fn generate_value_for_type(&mut self, _ty: &Ty) -> InputValue {
        todo!()
    }
}
