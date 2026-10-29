use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast::Type;
use apollo_compiler::ty;
use arbitrary::Result as ArbitraryResult;
use once_cell::sync::Lazy;

static BUILTIN_SCALAR_NAMES: Lazy<[Type; 5]> =
    Lazy::new(|| [ty!(Int), ty!(Float), ty!(String), ty!(Boolean), ty!(ID)]);

pub(crate) fn is_builtin(ty: &Type) -> bool {
    matches!(
        ty.inner_named_type().as_str(),
        "Int" | "Float" | "String" | "Boolean" | "ID"
    )
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `Type`
    pub fn ty(&mut self) -> ArbitraryResult<Type> {
        self.generate_ty(true)
    }

    /// Choose an arbitrary existing `Type` given a slice of existing types
    pub fn choose_ty(&mut self, existing_types: &[Type]) -> ArbitraryResult<Type> {
        self.choose_ty_given_nullable(existing_types, true)
    }

    /// Choose an arbitrary existing named `Type` given a slice of existing types
    pub fn choose_named_ty(&mut self, existing_types: &[Type]) -> ArbitraryResult<Type> {
        let used_type_names: Vec<&Type> = existing_types
            .iter()
            .chain(BUILTIN_SCALAR_NAMES.iter())
            .collect();

        Ok(self.u.choose(&used_type_names)?.to_owned().clone())
    }

    fn choose_ty_given_nullable(
        &mut self,
        existing_types: &[Type],
        is_nullable: bool,
    ) -> ArbitraryResult<Type> {
        let ty: Type = match self.u.int_in_range(0..=2usize)? {
            // Named type
            0 => {
                let used_type_names: Vec<&Type> = existing_types
                    .iter()
                    .chain(BUILTIN_SCALAR_NAMES.iter())
                    .collect();

                self.u.choose(&used_type_names)?.to_owned().clone()
            }
            // List type
            1 => Type::List(Box::new(
                self.choose_ty_given_nullable(existing_types, true)?,
            )),
            // Non Null type
            2 => {
                if is_nullable {
                    self.choose_ty_given_nullable(existing_types, false)?
                        .non_null()
                } else {
                    self.choose_ty_given_nullable(existing_types, is_nullable)?
                }
            }
            _ => unreachable!(),
        };

        Ok(ty)
    }

    fn generate_ty(&mut self, is_nullable: bool) -> ArbitraryResult<Type> {
        let ty = match self.u.int_in_range(0..=2usize)? {
            // Named type
            0 => Type::Named(self.name()?),
            // List type
            1 => Type::List(Box::new(self.generate_ty(true)?)),
            // Non Null type
            2 => {
                if is_nullable {
                    self.generate_ty(false)?.non_null()
                } else {
                    self.generate_ty(is_nullable)?
                }
            }
            _ => unreachable!(),
        };

        Ok(ty)
    }

    /// List the existing types that are valid in output positions (field
    /// types). Built-in scalars are added separately by `choose_ty`.
    pub(crate) fn list_existing_types(&self) -> Vec<Type> {
        self.object_type_defs
            .iter()
            .map(|o| Type::Named(o.type_name().clone()))
            .chain(
                self.interface_type_defs
                    .iter()
                    .map(|i| Type::Named(i.type_name().clone())),
            )
            .chain(
                self.union_type_defs
                    .iter()
                    .map(|u| Type::Named(u.type_name().clone())),
            )
            .chain(
                self.enum_type_defs
                    .iter()
                    .map(|e| Type::Named(e.type_name().clone())),
            )
            .chain(
                self.scalar_type_defs
                    .iter()
                    .map(|s| Type::Named(s.type_name().clone())),
            )
            .collect()
    }

    /// List the existing types that are valid in input positions (argument
    /// types, input-object fields, variable definitions). Per the GraphQL
    /// spec, only scalars, enums, and input objects qualify. Built-in
    /// scalars are added separately by `choose_ty`.
    pub(crate) fn list_existing_input_types(&self) -> Vec<Type> {
        self.scalar_type_defs
            .iter()
            .map(|s| Type::Named(s.type_name().clone()))
            .chain(
                self.enum_type_defs
                    .iter()
                    .map(|e| Type::Named(e.type_name().clone())),
            )
            .chain(
                self.input_object_type_defs
                    .iter()
                    .map(|io| Type::Named(io.type_name().clone())),
            )
            .collect()
    }

    /// List all existing object (already created) `Type`
    pub(crate) fn list_existing_object_types(&self) -> Vec<Type> {
        self.object_type_defs
            .iter()
            .map(|o| Type::Named(o.type_name().clone()))
            .collect()
    }
}
