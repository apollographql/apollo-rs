use std::fmt;

use crate::{Directive, StringValue};

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
/// let mut union_ = UnionDef::new("Pet".to_string());
/// union_.member("Cat".to_string());
/// union_.member("Dog".to_string());
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
    /// Contains all directives.
    directives: Vec<Directive>,
    extend: bool,
}

impl UnionDef {
    /// Create a new instance of a UnionDef.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: StringValue::Top { source: None },
            members: Vec::new(),
            extend: false,
            directives: Vec::new(),
        }
    }

    /// Set the union type as an extension
    pub fn extend(&mut self) {
        self.extend = true;
    }

    /// Set the UnionDefs description.
    pub fn description(&mut self, description: Option<String>) {
        self.description = StringValue::Top {
            source: description,
        };
    }

    /// Add a directive
    pub fn directive(&mut self, directive: Directive) {
        self.directives.push(directive);
    }

    /// Set a UnionDef member.
    pub fn member(&mut self, member: String) {
        self.members.push(member);
    }
}

impl fmt::Display for UnionDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        } else {
            // No description when it's a extension
            write!(f, "{}", self.description)?;
        }

        write!(f, "union {}", self.name)?;

        for directive in &self.directives {
            write!(f, " {}", directive)?;
        }

        write!(f, " =")?;

        for (i, member) in self.members.iter().enumerate() {
            match i {
                0 => write!(f, " {}", member)?,
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
        let mut union_ = UnionDef::new("Pet".to_string());
        union_.description(Some("A union of all animals in a household.".to_string()));
        union_.member("Cat".to_string());
        union_.member("Dog".to_string());

        assert_eq!(
            union_.to_string(),
            r#""A union of all animals in a household."
union Pet = Cat | Dog
"#
        );
    }
}
