use std::fmt::{self, Display};

/// Convenience Type_ implementation used when creating a Field.
/// Can be a `NamedType`, a `NonNull` or a `List`.
///
/// This enum is resposible for encoding creating values such as `String!`, `[[[[String]!]!]!]!`, etc.
///
/// ### Example
/// ```rust
/// use apollo_encoder::{Type_};
///
/// let field_ty = Type_::NamedType {
///     name: "String".to_string(),
/// };
///
/// let list = Type_::List {
///     ty: Box::new(field_ty),
/// };
///
/// let non_null = Type_::NonNull { ty: Box::new(list) };
///
/// assert_eq!(non_null.to_string(), "[String]!");
/// ```
#[derive(Debug, PartialEq, Clone)]
pub enum Type_ {
    /// The Non-Null field type.
    NonNull {
        /// Null inner type.
        ty: Box<Type_>,
    },
    /// The List field type.
    List {
        /// List inner type.
        ty: Box<Type_>,
    },
    /// The Named field type.
    NamedType {
        /// NamedType type.
        name: String,
    },
}

impl Type_ {
    /// Create a new instance of Type_::NonNull.
    pub const fn non_null(ty: Box<Type_>) -> Self {
        Type_::NonNull { ty }
    }

    /// Create a new instance of Type_::List.
    pub const fn list(ty: Box<Type_>) -> Self {
        Type_::List { ty }
    }

    /// Create a new instance of Type_::NamedType.
    #[inline(always)]
    pub fn named_type(name: &str) -> Self {
        Type_::NamedType {
            name: name.to_string(),
        }
    }
}

impl Display for Type_ {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type_::List { ty } => {
                write!(f, "[{}]", ty)
            }
            Type_::NonNull { ty } => {
                write!(f, "{}!", ty)
            }
            Type_::NamedType { name } => write!(f, "{}", name),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn encodes_simple_field_value() {
        let field_ty = Type_::NamedType {
            name: "String".to_string(),
        };

        assert_eq!(field_ty.to_string(), "String");
    }

    #[test]
    fn encodes_list_field_value() {
        let field_ty = Type_::NamedType {
            name: "String".to_string(),
        };

        let list = Type_::List {
            ty: Box::new(field_ty),
        };

        assert_eq!(list.to_string(), "[String]");
    }

    #[test]
    fn encodes_non_null_list_field_value() {
        let field_ty = Type_::NamedType {
            name: "String".to_string(),
        };

        let list = Type_::List {
            ty: Box::new(field_ty),
        };

        let non_null = Type_::NonNull { ty: Box::new(list) };

        assert_eq!(non_null.to_string(), "[String]!");
    }
    #[test]
    fn encodes_non_null_list_non_null_list_field_value() {
        let field_ty = Type_::NamedType {
            name: "String".to_string(),
        };

        let list = Type_::List {
            ty: Box::new(field_ty),
        };

        let non_null = Type_::NonNull { ty: Box::new(list) };

        let list_2 = Type_::List {
            ty: Box::new(non_null),
        };

        let non_null_2 = Type_::NonNull {
            ty: Box::new(list_2),
        };

        assert_eq!(non_null_2.to_string(), "[[String]!]!");
    }
}
