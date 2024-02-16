use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceType {
    Schema,
    Executable,
}

/// Represents a GraphQL source file.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Source {
    pub(crate) ty: SourceType,
    pub(crate) ast: Option<Arc<crate::ast::Document>>,
}

impl Source {
    pub fn source_type(&self) -> SourceType {
        self.ty
    }
}
