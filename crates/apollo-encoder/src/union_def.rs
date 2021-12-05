use std::fmt;

use crate::StringValue;

/// UnionDefs are an abstract type where no common fields are declared.
///
/// *UnionDefTypeDefinition*:
///     Description? **union** Name Directives? UnionDefMemberTypes?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-UnionDef).
///
/// ### Example
/// ```rust
/// use apollo_encoder::{UnionDef};
///
/// let mut union_ = UnionDef::new("Pet");
/// union_.member("Cat");
/// union_.member("Dog");
///
/// assert_eq!(
///     union_.to_string(),
/// r#"union Pet = Cat | Dog
/// "#
/// );
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct UnionDef {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: StringValue,
    // The vector of members that can be represented within this union.
    members: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct UnionDefBuilder {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<String>,
    // The vector of members that can be represented within this union.
    members: Vec<String>,
}

impl UnionDefBuilder {
    /// Create a new instance of a UnionDefBuilder.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            members: Vec::new(),
        }
    }

    /// Set the UnionDefs description.
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set a UnionDef member.
    pub fn member(mut self, member: &str) -> Self {
        self.members.push(member.to_string());
        self
    }

    /// Create a new instance of a UnionDef.
    pub fn build(self) -> UnionDef {
        UnionDef {
            name: self.name,
            description: StringValue::Top {
                source: self.description,
            },
            members: self.members,
        }
    }
}

impl fmt::Display for UnionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description)?;

        write!(f, "union {} = ", self.name)?;

        for (i, member) in self.members.iter().enumerate() {
            match i {
                0 => write!(f, "{}", member)?,
                _ => write!(f, " | {}", member)?,
            }
        }

        writeln!(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_union_with_description() {
        let union_ = {
            let mut union_ = UnionDef::new("Pet");
            union_.description("A union of all animals in a household.");
            union_.member("Cat");
            union_.member("Dog");
            union_
        };

        assert_eq!(
            union_.to_string(),
            r#""A union of all animals in a household."
union Pet = Cat | Dog
"#
        );
    }
}
