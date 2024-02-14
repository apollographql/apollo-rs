use std::collections::HashSet;

use arbitrary::Result;
use arbitrary::Unstructured;
use paste::paste;

use apollo_compiler::ast::{
    Definition, DirectiveDefinition, DirectiveLocation, Document, EnumTypeDefinition,
    EnumTypeExtension, EnumValueDefinition, FieldDefinition, FragmentDefinition,
    InputObjectTypeDefinition, InputObjectTypeExtension, InputValueDefinition,
    InterfaceTypeDefinition, InterfaceTypeExtension, Name, ObjectTypeDefinition,
    ObjectTypeExtension, OperationDefinition, OperationType, ScalarTypeDefinition,
    ScalarTypeExtension, SchemaDefinition, SchemaExtension, Type, UnionTypeDefinition,
    UnionTypeExtension, Value,
};
use apollo_compiler::executable::DirectiveList;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::{Node, NodeStr, Schema};

use crate::next::ast::definition::{DefinitionExt, DefinitionKind};
use crate::next::common::Common;
use crate::next::schema::extended_type::{ExtendedTypeExt, ExtendedTypeKind};
use crate::next::unstructured::{UnstructuredExt, UnstructuredOption};

use super::super::schema::schema::SchemaExt;
use super::directive_definition::DirectiveDefinitionIterExt;

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

pub(crate) trait DocumentExt: Common {
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

    fn arbitrary_schema_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<SchemaDefinition> {
        Ok(SchemaDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::Schema)
                .try_collect(u, self.target(), schema)?,
            root_operations: vec![
                Some(Node::new((
                    OperationType::Query,
                    self.random_object_type_definition(u)?.name.clone(),
                ))),
                Node::new((
                    OperationType::Mutation,
                    self.random_object_type_definition(u)?.name.clone(),
                ))
                .optional(u)?,
                Node::new((
                    OperationType::Subscription,
                    self.random_object_type_definition(u)?.name.clone(),
                ))
                .optional(u)?,
            ]
            .into_iter()
            .filter_map(|op| op)
            .collect(),
        })
    }
    fn arbitrary_definition_name(&self, u: &mut Unstructured, schema: &Schema) -> Result<Name> {
        let existing_names = self
            .target()
            .definitions
            .iter()
            .filter_map(|d| d.name())
            .collect::<HashSet<_>>();
        loop {
            let name = self.arbitrary_name(u)?;
            if !existing_names.contains(&name) {
                return Ok(name);
            }
        }
    }

    fn arbitrary_object_type_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<ObjectTypeDefinition> {
        Ok(ObjectTypeDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_definition_name(u, schema)?,
            implements_interfaces: schema
                .sample_interface_types(u)?
                .iter()
                .map(|i| i.name.clone())
                .collect(),
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::Object)
                .try_collect(u, self.target(), schema)?,
            fields: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_field_definition(
                    u,
                    schema,
                    DirectiveLocation::FieldDefinition,
                )?))
            })?,
        })
    }

    fn arbitrary_directive_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<DirectiveDefinition> {
        Ok(DirectiveDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_definition_name(u, schema)?,
            arguments: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_input_value_definition(u, schema)?))
            })?,
            repeatable: u.arbitrary()?,
            locations: self.arbitrary_directive_locations(u)?,
        })
    }

    fn arbitrary_input_object_type_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<InputObjectTypeDefinition> {
        Ok(InputObjectTypeDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_name(u)?,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::InputObject)
                .try_collect(u, self.target(), schema)?,
            fields: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_input_value_definition(u, schema)?))
            })?,
        })
    }

    fn arbitrary_input_value_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<InputValueDefinition> {
        let ty = schema
            .random_type(
                u,
                vec![
                    ExtendedTypeKind::InputObjectTypeDefinition,
                    ExtendedTypeKind::Scalar,
                ],
            )?
            .ty(u)?;
        let default_value = self
            .arbitrary_value(u, &ty, schema)?
            .optional(u)?
            .map(Node::new);
        Ok(InputValueDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.unique_name(),
            ty: Node::new(ty),
            default_value,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::InputFieldDefinition)
                .try_collect(u, self.target(), schema)?,
        })
    }

    fn arbitrary_enum_type_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<EnumTypeDefinition> {
        Ok(EnumTypeDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_definition_name(u, schema)?,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::Enum)
                .try_collect(u, self.target(), schema)?,
            values: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(EnumValueDefinition {
                    description: self.arbitrary_node_str(u)?.optional(u)?,
                    value: self.arbitrary_name(u)?,
                    directives: schema
                        .sample_directives(u)?
                        .into_iter()
                        .with_location(DirectiveLocation::EnumValue)
                        .try_collect(u, self.target(), schema)?,
                }))
            })?,
        })
    }

    fn arbitrary_union_type_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<UnionTypeDefinition> {
        Ok(UnionTypeDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_definition_name(u, schema)?,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::Union)
                .try_collect(u, self.target(), schema)?,
            members: self
                .sample_object_type_definitions(u)?
                .iter()
                .map(|i| i.name.clone())
                .collect(),
        })
    }

    fn arbitrary_interface_type_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> Result<InterfaceTypeDefinition> {
        Ok(InterfaceTypeDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.arbitrary_definition_name(u, schema)?,
            implements_interfaces: self
                .sample_interface_type_definitions(u)?
                .iter()
                .map(|interface| interface.name.clone())
                .collect(),
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(DirectiveLocation::Interface)
                .try_collect(u, self.target(), schema)?,
            fields: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_field_definition(
                    u,
                    schema,
                    DirectiveLocation::InputFieldDefinition,
                )?))
            })?,
        })
    }

    fn arbitrary_field_definition(
        &self,
        u: &mut Unstructured,
        schema: &Schema,
        directive_location: DirectiveLocation,
    ) -> Result<FieldDefinition> {
        Ok(FieldDefinition {
            description: self.arbitrary_node_str(u)?.optional(u)?,
            name: self.unique_name(),
            arguments: u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_input_value_definition(u, schema)?))
            })?,
            ty: schema
                .random_type(
                    u,
                    vec![
                        ExtendedTypeKind::Scalar,
                        ExtendedTypeKind::Object,
                        ExtendedTypeKind::Enum,
                        ExtendedTypeKind::Union,
                        ExtendedTypeKind::Interface,
                    ],
                )?
                .ty(u)?,
            directives: schema
                .sample_directives(u)?
                .into_iter()
                .with_location(directive_location)
                .try_collect(u, self.target(), schema)?,
        })
    }

    fn arbitrary_value(&self, u: &mut Unstructured, ty: &Type, schema: &Schema) -> Result<Value> {
        match ty {
            Type::Named(ty) => {
                if u.arbitrary()? {
                    self.arbitrary_value(u, &Type::NonNullNamed(ty.clone()), schema)
                } else {
                    Ok(Value::Null)
                }
            }
            Type::List(ty) => {
                if u.arbitrary()? {
                    Ok(Value::List(u.arbitrary_vec(0, 5, |u| {
                        Ok(Node::new(self.arbitrary_value(u, ty, schema)?))
                    })?))
                } else {
                    Ok(Value::Null)
                }
            }
            Type::NonNullNamed(ty) => match schema.types.get(ty).expect("type must exist") {
                ExtendedType::Scalar(ty) => {
                    if ty.name == "Int" {
                        Ok(Value::from(u.arbitrary::<f64>()?))
                    } else if ty.name == "Float" {
                        Ok(Value::from(u.arbitrary::<i32>()?))
                    } else if ty.name == "Boolean" {
                        Ok(Value::Boolean(u.arbitrary()?))
                    } else {
                        Ok(Value::String(self.arbitrary_node_str(u)?))
                    }
                }
                ExtendedType::Object(ty) => {
                    let mut values = Vec::new();
                    for (name, definition) in &ty.fields {
                        values.push((
                            name.clone(),
                            Node::new(self.arbitrary_value(u, &definition.ty, schema)?),
                        ));
                    }
                    Ok(Value::Object(values))
                }
                ExtendedType::Enum(ty) => {
                    let values = ty
                        .values
                        .iter()
                        .map(|(name, v)| &v.value)
                        .collect::<Vec<_>>();
                    if values.is_empty() {
                        panic!("enum must have at least one value")
                    } else {
                        Ok(Value::Enum((*u.choose(&values)?).clone()))
                    }
                }
                ExtendedType::InputObject(ty) => {
                    let mut values = Vec::new();
                    for (name, definition) in &ty.fields {
                        values.push((
                            name.clone(),
                            Node::new(self.arbitrary_value(u, &definition.ty, schema)?),
                        ));
                    }
                    Ok(Value::Object(values))
                }
                _ => {
                    panic!("type must be a scalar, object, enum or input object")
                }
            },
            Type::NonNullList(ty) => Ok(Value::List(u.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(self.arbitrary_value(u, ty, schema)?))
            })?)),
        }
    }

    fn random_definition(
        &self,
        u: &mut Unstructured,
        types: Vec<DefinitionKind>,
    ) -> Result<&Definition> {
        let definitions = self
            .target()
            .definitions
            .iter()
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

    fn definition(&self, name: &Name) -> Option<&Definition> {
        self.target()
            .definitions
            .iter()
            .find(|d| d.name() == Some(&name))
    }

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

impl Common for Document {}
