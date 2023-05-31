use crate::hir::HirNodeLocation;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Name {
    pub(crate) src: String,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl Name {
    /// Get a reference to the name itself.
    pub fn src(&self) -> &str {
        self.src.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }
}

impl From<Name> for String {
    fn from(name: Name) -> String {
        name.src
    }
}

impl From<String> for Name {
    fn from(name: String) -> Name {
        Name {
            src: name,
            loc: None,
        }
    }
}
