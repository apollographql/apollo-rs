use paste::paste;

use apollo_compiler::ast::{
    Definition, DirectiveDefinition, Document, EnumTypeDefinition, EnumTypeExtension,
    FragmentDefinition, InputObjectTypeDefinition, InputObjectTypeExtension,
    InterfaceTypeDefinition, InterfaceTypeExtension, ObjectTypeDefinition, ObjectTypeExtension,
    OperationDefinition, ScalarTypeDefinition, ScalarTypeExtension, SchemaDefinition,
    SchemaExtension, UnionTypeDefinition, UnionTypeExtension,
};
use apollo_compiler::Node;

use crate::next::unstructured::Unstructured;

/// Macro to create accessors for definitions
macro_rules! access {
    ($ty: ty) => {
        paste! {
            fn [<random_ $ty:snake>](
                &self,
                u: &mut Unstructured,
            ) -> arbitrary::Result<&Node<$ty>> {
                let mut existing = self
                    .target()
                    .definitions
                    .iter()
                    .filter_map(|d| {
                        if let Definition::$ty(definition) = d {
                            Some(definition)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let idx = u.choose_index(existing.len()).map_err(|e|{
                    if let arbitrary::Error::EmptyChoose = e {
                        panic!("no existing definitions of type {}", stringify!($ty))
                    } else {
                        e
                    }
                })?;
                Ok(existing.remove(idx))
            }

            fn [<random_ $ty:snake _mut>](
                &mut self,
                u: &mut Unstructured,
            ) -> arbitrary::Result<&mut Node<$ty>> {
                let mut existing = self
                    .target_mut()
                    .definitions
                    .iter_mut()
                    .filter_map(|d| {
                        if let Definition::$ty(definition) = d {
                            Some(definition)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                let idx = u.choose_index(existing.len()).map_err(|e|{
                    if let arbitrary::Error::EmptyChoose = e {
                        panic!("no existing definitions of type {}", stringify!($ty))
                    } else {
                        e
                    }
                })?;
                Ok(existing.remove(idx))
            }

            fn [<sample_ $ty:snake s>](
                &self,
                u: &mut Unstructured,
            ) -> arbitrary::Result<Vec<&Node<$ty>>> {
                let existing = self
                    .target()
                    .definitions
                    .iter()
                    .filter_map(|d| {
                        if let Definition::$ty(definition) = d {
                            Some(definition)
                        } else {
                            None
                        }
                    })
                    .filter(|_| u.arbitrary().unwrap_or(false))
                    .collect::<Vec<_>>();

                Ok(existing)
            }
        }
    };
}

pub(crate) trait DocumentExt {
    access!(OperationDefinition);
    access!(FragmentDefinition);
    access!(DirectiveDefinition);
    access!(SchemaDefinition);
    access!(ScalarTypeDefinition);
    access!(ObjectTypeDefinition);
    access!(InterfaceTypeDefinition);
    access!(UnionTypeDefinition);
    access!(EnumTypeDefinition);
    access!(InputObjectTypeDefinition);
    access!(SchemaExtension);
    access!(ScalarTypeExtension);
    access!(ObjectTypeExtension);
    access!(InterfaceTypeExtension);
    access!(UnionTypeExtension);
    access!(EnumTypeExtension);
    access!(InputObjectTypeExtension);

    fn target(&self) -> &Document;
    fn target_mut(&mut self) -> &mut Document;
}

impl DocumentExt for Document {
    fn target(&self) -> &Document {
        self
    }
    fn target_mut(&mut self) -> &mut Document {
        self
    }
}
