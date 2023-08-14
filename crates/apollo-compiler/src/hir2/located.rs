use crate::hir::HirNodeLocation;
use crate::FileId;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Ranged;

#[derive(Debug, Clone)]
pub struct Located<T> {
    file_id: Option<FileId>,
    harc: Harc<Ranged<T>>,
}

#[derive(Debug, Clone)]
pub struct LocatedBorrow<'a, T> {
    file_id: Option<FileId>,
    harc: &'a Harc<Ranged<T>>,
}

impl<T> std::ops::Deref for Located<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.harc.node
    }
}

impl<T> std::ops::Deref for LocatedBorrow<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.harc.node
    }
}

impl<T> Located<T> {
    pub fn no_file_id(value: Harc<Ranged<T>>) -> Self {
        Self {
            file_id: None,
            harc: value,
        }
    }

    pub fn with_file_id(value: Harc<Ranged<T>>, file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            harc: value,
        }
    }

    pub fn component<'a, C>(
        &'a self,
        f: impl FnOnce(&'a Harc<Ranged<T>>) -> &'a Harc<Ranged<C>>,
    ) -> LocatedBorrow<'a, C> {
        LocatedBorrow {
            file_id: self.file_id,
            harc: f(&self.harc),
        }
    }

    pub fn borrow(&self) -> LocatedBorrow<'_, T> {
        self.component(|harc| harc)
    }

    pub fn source_location(&self) -> Option<HirNodeLocation> {
        self.borrow().source_location()
    }
}

impl<'a, T> LocatedBorrow<'a, T> {
    pub fn to_owned(&self) -> Located<T> {
        Located {
            file_id: self.file_id,
            harc: Harc::clone(self.harc),
        }
    }

    pub fn source_location(&self) -> Option<HirNodeLocation> {
        let file_id = self.file_id?;
        let text_range = self.harc.location()?;
        Some(HirNodeLocation {
            file_id,
            offset: text_range.start().into(),
            node_len: text_range.len().into(),
        })
    }
}
