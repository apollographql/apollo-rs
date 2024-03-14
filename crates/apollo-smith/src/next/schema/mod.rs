use std::sync::OnceLock;
use apollo_compiler::ast::{FieldDefinition, Name};
use apollo_compiler::schema::{Component, ExtendedType, InterfaceType, ObjectType};
use indexmap::IndexMap;
use crate::next::Unstructured;

pub(crate) mod extended_type;

pub(crate) mod object_type;
pub(crate) mod schema;

/// macro for accessing fields on schema elements
macro_rules! field_access {
    ($ty:ty) => {
        paste::paste! {
            pub(crate) trait [<$ty Ext>] {
                fn random_field(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Option<&apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let mut fields = self.target().fields.values().collect::<Vec<_>>();
                    match u.choose_index(fields.len()) {
                        Ok(idx)=> Ok(Some(fields.remove(idx))),
                        Err(arbitrary::Error::EmptyChoose)=> Ok(None),
                        Err(e)=> Err(e)
                    }
                }

                fn random_field_mut(
                    &mut self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Option<&mut apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let mut fields = self.target_mut().fields.values_mut().collect::<Vec<_>>();
                    match u.choose_index(fields.len()) {
                        Ok(idx)=> Ok(Some(fields.remove(idx))),
                        Err(arbitrary::Error::EmptyChoose)=> Ok(None),
                        Err(e)=> Err(e)
                    }

                }

                fn sample_fields(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Vec<&apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let existing = self
                        .target()
                        .fields
                        .values()
                        .filter(|_| u.arbitrary().unwrap_or(false))
                        .collect::<Vec<_>>();

                    Ok(existing)
                }
                fn target(&self) -> &$ty;
                fn target_mut(&mut self) -> &mut $ty;
            }

            impl [<$ty Ext>] for $ty {
                fn target(&self) -> &$ty {
                    self
                }
                fn target_mut(&mut self) -> &mut $ty {
                    self
                }
            }
        }
    };
}

field_access!(ObjectType);
field_access!(InterfaceType);

pub(crate) trait TypeHasFields {
    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>>;
    fn random_field(&self, u: &mut Unstructured) -> arbitrary::Result<&Component<FieldDefinition>> {
        // Types always have at least one field
        let fields = self.fields().values().collect::<Vec<_>>();
        Ok(fields[u.choose_index(fields.len())?])
    }

}

impl TypeHasFields for ObjectType {
    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        &self.fields
    }
}

impl TypeHasFields for InterfaceType {
    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        &self.fields
    }
}

impl TypeHasFields for ExtendedType {
    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        static EMPTY: OnceLock<IndexMap<Name, Component<FieldDefinition>>> = OnceLock::new();
        match self {
            ExtendedType::Object(t) => t.fields(),
            ExtendedType::Interface(t) => t.fields(),
            _ => &EMPTY.get_or_init(||Default::default()),
        }
    }
}
