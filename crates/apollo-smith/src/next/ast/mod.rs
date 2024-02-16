use apollo_compiler::ast::{FieldDefinition, InterfaceTypeDefinition, ObjectTypeDefinition};
use apollo_compiler::Node;

pub(crate) mod definition;
pub(crate) mod directive_definition;
pub(crate) mod document;

/// macro for accessing fields on ast elements
macro_rules! field_access {
    ($ty:ty) => {
        paste::paste! {
            pub(crate) trait [<$ty Ext>] {
                fn random_field(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<&apollo_compiler::Node<apollo_compiler::ast::FieldDefinition>> {
                    Ok(u.choose(&self.target().fields).map_err(|e| {
                        if let arbitrary::Error::EmptyChoose = e {
                            panic!("no existing fields")
                        } else {
                            e
                        }
                    })?)
                }

                fn random_field_mut(
                    &mut self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<&mut apollo_compiler::Node<apollo_compiler::ast::FieldDefinition>> {
                    let idx = u.choose_index(self.target().fields.len()).map_err(|e| {
                        if let arbitrary::Error::EmptyChoose = e {
                            panic!("no existing fields")
                        } else {
                            e
                        }
                    })?;
                    Ok(&mut self.target_mut().fields[idx])
                }

                fn sample_fields(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Vec<&apollo_compiler::Node<apollo_compiler::ast::FieldDefinition>>> {
                    let existing = self
                        .target()
                        .fields
                        .iter()
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

field_access!(ObjectTypeDefinition);
field_access!(InterfaceTypeDefinition);

pub(crate) trait HasFields {
    fn fields(&self) -> &Vec<Node<FieldDefinition>>;
    fn fields_mut(&mut self) -> &mut Vec<Node<FieldDefinition>>;
}

impl HasFields for ObjectTypeDefinition {
    fn fields(&self) -> &Vec<Node<FieldDefinition>> {
        &self.fields
    }

    fn fields_mut(&mut self) -> &mut Vec<Node<FieldDefinition>> {
        &mut self.fields
    }
}

impl HasFields for InterfaceTypeDefinition {
    fn fields(&self) -> &Vec<Node<FieldDefinition>> {
        &self.fields
    }

    fn fields_mut(&mut self) -> &mut Vec<Node<FieldDefinition>> {
        &mut self.fields
    }
}
