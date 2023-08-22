use crate::hir::HirNodeLocation;
use crate::FileId;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Ranged;

/// Wraps a `Harc<Ranged<T>>` typically from a MIR document and adds an optional `FileId`.
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
    pub fn no_location(node: T) -> Self {
        Self {
            file_id: None,
            harc: Harc::new(Ranged::no_location(node)),
        }
    }

    pub fn no_file_id(harc: Harc<Ranged<T>>) -> Self {
        Self {
            file_id: None,
            harc,
        }
    }

    pub fn with_file_id(value: Harc<Ranged<T>>, file_id: FileId) -> Self {
        Self {
            file_id: Some(file_id),
            harc: value,
        }
    }

    pub fn same_file_id<'a, C>(&self, harc: &'a Harc<Ranged<C>>) -> LocatedBorrow<'a, C> {
        LocatedBorrow {
            file_id: self.file_id,
            harc,
        }
    }

    pub fn borrow(&self) -> LocatedBorrow<'_, T> {
        self.same_file_id(&self.harc)
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
        Some(HirNodeLocation {
            file_id: self.file_id?,
            text_range: *self.harc.location()?,
        })
    }
}
