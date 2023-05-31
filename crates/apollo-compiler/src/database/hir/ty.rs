use crate::{
    hir::{HirNodeLocation, TypeDefinition},
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Type {
    NonNull {
        ty: Box<Type>,
        loc: Option<HirNodeLocation>,
    },
    List {
        ty: Box<Type>,
        loc: Option<HirNodeLocation>,
    },
    Named {
        name: String,
        loc: Option<HirNodeLocation>,
    },
}

impl Type {
    /// Returns `true` if the type is [`NonNull`].
    ///
    /// [`NonNull`]: Type::NonNull
    #[must_use]
    pub fn is_non_null(&self) -> bool {
        matches!(self, Self::NonNull { .. })
    }

    /// Returns `true` if the type is [`Named`].
    ///
    /// [`Named`]: Type::Named
    #[must_use]
    pub fn is_named(&self) -> bool {
        matches!(self, Self::Named { .. })
    }

    /// Returns `true` if the type is [`List`].
    ///
    /// [`List`]: Type::List
    #[must_use]
    pub fn is_list(&self) -> bool {
        matches!(self, Self::List { .. })
    }

    /// Returns `true` if Type is either a [`ScalarTypeDefinition`],
    /// [`ObjectTypeDefinition`], [`InterfaceTypeDefinition`],
    /// [`UnionTypeDefinition`], [`EnumTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`ObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    /// [`InterfaceTypeDefinition`]: Definition::InterfaceTypeDefinition
    /// [`UnionTypeDefinition`]: Definition::UnionTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    #[must_use]
    pub fn is_output_type(&self, db: &dyn HirDatabase) -> bool {
        if let Some(ty) = self.type_def(db) {
            ty.is_output_definition()
        } else {
            false
        }
    }

    /// Returns `true` if the Type is either a [`ScalarTypeDefinition`],
    /// [`EnumTypeDefinition`], [`InputObjectTypeDefinition`].
    ///
    /// [`ScalarTypeDefinition`]: Definition::ScalarTypeDefinition
    /// [`EnumTypeDefinition`]: Definition::EnumTypeDefinition
    /// [`InputObjectTypeDefinition`]: Definition::ObjectTypeDefinition
    #[must_use]
    pub fn is_input_type(&self, db: &dyn HirDatabase) -> bool {
        if let Some(ty) = self.type_def(db) {
            ty.is_input_definition()
        } else {
            false
        }
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        match self {
            Type::NonNull { loc, .. } | Type::List { loc, .. } | Type::Named { loc, .. } => *loc,
        }
    }

    /// Get current Type's Type Definition.
    pub fn type_def(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.name())
    }

    /// Get current Type's name.
    pub fn name(&self) -> String {
        match self {
            Type::NonNull { ty, .. } | Type::List { ty, .. } => ty.name(),
            Type::Named { name, .. } => name.clone(),
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::NonNull { ty, .. } => write!(f, "{ty}!"),
            Type::List { ty, .. } => write!(f, "[{ty}]"),
            Type::Named { name, .. } => write!(f, "{name}"),
        }
    }
}
