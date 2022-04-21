use std::fmt;

use crate::{Directive, StringValue};

/// UnionDefinitions are an abstract type where no common fields are declared.
///
/// *UnionDefTypeDefinition*:
///     Description? **union** Name Directives? UnionDefMemberTypes?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#UnionTypeDefinition).
///
/// ### Example
/// ```rust
/// use apollo_encoder::UnionDefinition;
///
/// let mut union_ = UnionDefinition::new("Pet".to_string());
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
pub struct UnionDefinition {
    // Name must return a String.
    name: String,
    // Description may return a String.
    description: Option<StringValue>,
    // The vector of members that can be represented within this union.
    members: Vec<String>,
    /// Contains all directives.
    directives: Vec<Directive>,
    extend: bool,
}

impl UnionDefinition {
    /// Create a new instance of a UnionDef.
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
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
    pub fn description(&mut self, description: String) {
        self.description = Some(StringValue::Top {
            source: description,
        });
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

impl fmt::Display for UnionDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.extend {
            write!(f, "extend ")?;
        // No description when it's a extension
        } else if let Some(description) = &self.description {
            write!(f, "{}", description)?;
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
    use pretty_assertions::assert_eq;

    #[test]
    fn it_encodes_union_with_description() {
        let mut union_ = UnionDefinition::new("Pet".to_string());
        union_.description("A union of all animals in a household.".to_string());
        union_.member("Cat".to_string());
        union_.member("Dog".to_string());

        assert_eq!(
            union_.to_string(),
            r#""A union of all animals in a household."
union Pet = Cat | Dog
"#
        );
    }

    #[test]
    fn it_encodes_union_extension() {
        let mut union_ = UnionDefinition::new("Pet".to_string());
        union_.description("A union of all animals in a household.".to_string());
        union_.member("Cat".to_string());
        union_.member("Dog".to_string());
        union_.extend();

        assert_eq!(
            union_.to_string(),
            r#"extend union Pet = Cat | Dog
"#
        );
    }
}
