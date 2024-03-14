use arbitrary::Unstructured;
use paste::paste;

use apollo_compiler::schema::{DirectiveDefinition, ExtendedType};
use apollo_compiler::schema::{
    EnumType, InputObjectType, InterfaceType, ObjectType, ScalarType, UnionType,
};
use apollo_compiler::Node;
use apollo_compiler::Schema;

use crate::next::schema::extended_type::ExtendedTypeKind;

macro_rules! access {
    ($variant: ident, $ty: ty) => {
        paste! {
            fn [<random_ $ty:snake>](
                &self,
                u: &mut Unstructured,
            ) -> arbitrary::Result<&Node<$ty>> {
                let mut existing = self
                    .target()
                    .types.values()
                    .filter_map(|d| {
                        if let ExtendedType::$variant(definition) = d {
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
                    .types.values()
                    .filter_map(|d| {
                        if let ExtendedType::$variant(definition) = d {
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

pub(crate) trait SchemaExt {
    access!(Scalar, ScalarType);
    access!(Object, ObjectType);
    access!(Interface, InterfaceType);
    access!(Union, UnionType);
    access!(Enum, EnumType);
    access!(InputObject, InputObjectType);

    fn random_type(
        &self,
        u: &mut Unstructured,
        types: Vec<ExtendedTypeKind>,
    ) -> arbitrary::Result<&ExtendedType> {
        let definitions = self
            .target()
            .types
            .values()
            .filter(|d| types.iter().any(|t| t.matches(*d)))
            .collect::<Vec<_>>();
        Ok(u.choose(definitions.as_slice()).map_err(|e| {
            if let arbitrary::Error::EmptyChoose = e {
                panic!("no existing definitions of types {:?}", types)
            } else {
                e
            }
        })?)
    }

    fn random_directive(
        &self,
        u: &mut Unstructured,
    ) -> arbitrary::Result<&Node<DirectiveDefinition>> {
        let mut existing = self
            .target()
            .directive_definitions
            .values()
            .collect::<Vec<_>>();
        let idx = u.choose_index(existing.len()).map_err(|e| {
            if let arbitrary::Error::EmptyChoose = e {
                panic!("no existing directive definitions")
            } else {
                e
            }
        })?;
        Ok(existing.remove(idx))
    }

    fn sample_directives(
        &self,
        u: &mut Unstructured,
    ) -> arbitrary::Result<Vec<&Node<DirectiveDefinition>>> {
        let existing = self
            .target()
            .directive_definitions
            .values()
            .filter(|_| u.arbitrary().unwrap_or(false))
            .collect::<Vec<_>>();

        Ok(existing)
    }

    fn random_query_mutation_subscription(
        &self,
        u: &mut Unstructured,
    ) -> arbitrary::Result<&Node<ObjectType>> {
        Ok(*u.choose(
            &vec![
                self.target().get_object("Query"),
                self.target().get_object("Mutation"),
                self.target().get_object("Subscription"),
            ]
            .into_iter()
            .filter_map(|o| o)
            .collect::<Vec<_>>(),
        )?)
    }

    fn target(&self) -> &Schema;
}

impl SchemaExt for Schema {
    fn target(&self) -> &Schema {
        &self
    }
}
