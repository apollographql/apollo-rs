use crate::next::ast::directive_definition::DirectiveDefinitionIterExt;
use crate::next::schema::extended_type::{ExtendedTypeExt, ExtendedTypeKind};
use crate::next::schema::schema::SchemaExt;
use apollo_compiler::ast::{
    DirectiveDefinition, DirectiveLocation, EnumTypeDefinition, EnumValueDefinition,
    FieldDefinition, InputObjectTypeDefinition, InputValueDefinition, InterfaceTypeDefinition,
    Name, ObjectTypeDefinition, OperationType, SchemaDefinition, Type, UnionTypeDefinition, Value,
};
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::{Node, NodeStr, Schema};
use arbitrary::Result;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::Ordering;

pub(crate) trait UnstructuredExt {
    fn arbitrary_vec<T, C: Fn(&mut Unstructured) -> Result<T>>(
        &mut self,
        min: usize,
        max: usize,
        callback: C,
    ) -> Result<Vec<T>>;
}
impl<'a> UnstructuredExt for Unstructured<'a> {
    fn arbitrary_vec<T, C: Fn(&mut Unstructured) -> Result<T>>(
        &mut self,
        min: usize,
        max: usize,
        callback: C,
    ) -> Result<Vec<T>> {
        let count = self.int_in_range(min..=max)?;
        let mut results = Vec::with_capacity(count);
        for _ in 0..count {
            results.push(callback(self)?);
        }
        Ok(results)
    }
}

pub(crate) trait UnstructuredOption: Sized {
    fn optional(self, u: &mut Unstructured) -> Result<Option<Self>>;
}

impl<T> UnstructuredOption for T {
    fn optional(self, u: &mut Unstructured) -> Result<Option<T>> {
        if u.arbitrary()? {
            Ok(Some(self))
        } else {
            Ok(None)
        }
    }
}

pub(crate) struct Unstructured<'a> {
    u: arbitrary::Unstructured<'a>,
    counter: usize,
}

impl Unstructured<'_> {
    pub(crate) fn new<'a>(data: &'a [u8]) -> Unstructured<'a> {
        Unstructured {
            u: arbitrary::Unstructured::new(data),
            counter: 0,
        }
    }

    pub(crate) fn unique_name(&mut self) -> Name {
        self.counter = self.counter + 1;
        Name::new(NodeStr::new(&format!("f{}", self.counter))).expect("valid name")
    }

    pub(crate) fn arbitrary_node_str(&mut self) -> Result<NodeStr> {
        let s: String = self.arbitrary()?;
        let idx = s
            .char_indices()
            .nth(10)
            .map(|(s, _c)| s)
            .unwrap_or_else(|| s.len());
        Ok(NodeStr::new(&s[..idx]))
    }

    pub(crate) fn arbitrary_directive_locations(
        &mut self,
    ) -> arbitrary::Result<Vec<DirectiveLocation>> {
        let mut locations = Vec::new();
        for _ in 0..self.int_in_range(1..=5)? {
            locations.push(
                self.choose(&[
                    DirectiveLocation::Query,
                    DirectiveLocation::Mutation,
                    DirectiveLocation::Subscription,
                    DirectiveLocation::Field,
                    DirectiveLocation::FragmentDefinition,
                    DirectiveLocation::FragmentSpread,
                    DirectiveLocation::InlineFragment,
                    DirectiveLocation::Schema,
                    DirectiveLocation::Scalar,
                    DirectiveLocation::Object,
                    DirectiveLocation::FieldDefinition,
                    DirectiveLocation::ArgumentDefinition,
                    DirectiveLocation::Interface,
                    DirectiveLocation::Union,
                    DirectiveLocation::Enum,
                    DirectiveLocation::EnumValue,
                    DirectiveLocation::InputObject,
                    DirectiveLocation::InputFieldDefinition,
                ])?
                .clone(),
            );
        }
        Ok(locations)
    }

    pub(crate) fn arbitrary_schema_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<SchemaDefinition> {
        Ok(SchemaDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Schema)
                .try_collect(self, schema)?,
            root_operations: vec![
                Some(Node::new((
                    OperationType::Query,
                    schema.random_object_type(self)?.name.clone(),
                ))),
                Node::new((
                    OperationType::Mutation,
                    schema.random_object_type(self)?.name.clone(),
                ))
                .optional(self)?,
                Node::new((
                    OperationType::Subscription,
                    schema.random_object_type(self)?.name.clone(),
                ))
                .optional(self)?,
            ]
            .into_iter()
            .filter_map(|op| op)
            .collect(),
        })
    }

    pub(crate) fn arbitrary_object_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<ObjectTypeDefinition> {
        Ok(ObjectTypeDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            implements_interfaces: schema
                .sample_interface_types(self)?
                .iter()
                .map(|i| i.name.clone())
                .collect(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Object)
                .try_collect(self, schema)?,
            fields: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(u.arbitrary_field_definition(
                    schema,
                    DirectiveLocation::FieldDefinition,
                )?))
            })?,
        })
    }

    pub(crate) fn arbitrary_directive_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<DirectiveDefinition> {
        Ok(DirectiveDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            arguments: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(u.arbitrary_input_value_definition(schema)?))
            })?,
            repeatable: self.arbitrary()?,
            locations: self.arbitrary_directive_locations()?,
        })
    }

    pub(crate) fn arbitrary_input_object_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<InputObjectTypeDefinition> {
        Ok(InputObjectTypeDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::InputObject)
                .try_collect(self, schema)?,
            fields: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(u.arbitrary_input_value_definition(schema)?))
            })?,
        })
    }

    pub(crate) fn arbitrary_input_value_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<InputValueDefinition> {
        let ty = schema
            .random_type(
                self,
                vec![
                    ExtendedTypeKind::InputObjectTypeDefinition,
                    ExtendedTypeKind::Scalar,
                ],
            )?
            .ty(self)?;
        let default_value = self
            .arbitrary_value(&ty, schema)?
            .optional(self)?
            .map(Node::new);
        Ok(InputValueDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            ty: Node::new(ty),
            default_value,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::InputFieldDefinition)
                .try_collect(self, schema)?,
        })
    }

    pub(crate) fn arbitrary_enum_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<EnumTypeDefinition> {
        Ok(EnumTypeDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Enum)
                .try_collect(self, schema)?,
            values: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(EnumValueDefinition {
                    description: u.arbitrary_node_str()?.optional(u)?,
                    value: u.unique_name(),
                    directives: schema
                        .sample_directives(u)?
                        .into_iter()
                        .with_location(DirectiveLocation::EnumValue)
                        .try_collect(u, schema)?,
                }))
            })?,
        })
    }

    pub(crate) fn arbitrary_union_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<UnionTypeDefinition> {
        Ok(UnionTypeDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Union)
                .try_collect(self, schema)?,
            members: schema
                .sample_object_types(self)?
                .iter()
                .map(|i| i.name.clone())
                .collect(),
        })
    }

    pub(crate) fn arbitrary_interface_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<InterfaceTypeDefinition> {
        Ok(InterfaceTypeDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            implements_interfaces: schema
                .sample_interface_types(self)?
                .iter()
                .map(|interface| interface.name.clone())
                .collect(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Interface)
                .try_collect(self, schema)?,
            fields: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(u.arbitrary_field_definition(
                    schema,
                    DirectiveLocation::InputFieldDefinition,
                )?))
            })?,
        })
    }

    pub(crate) fn arbitrary_field_definition(
        &mut self,
        schema: &Schema,
        directive_location: DirectiveLocation,
    ) -> Result<FieldDefinition> {
        Ok(FieldDefinition {
            description: self.arbitrary_node_str()?.optional(self)?,
            name: self.unique_name(),
            arguments: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(u.arbitrary_input_value_definition(schema)?))
            })?,
            ty: schema
                .random_type(
                    self,
                    vec![
                        ExtendedTypeKind::Scalar,
                        ExtendedTypeKind::Object,
                        ExtendedTypeKind::Enum,
                        ExtendedTypeKind::Union,
                        ExtendedTypeKind::Interface,
                    ],
                )?
                .ty(self)?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(directive_location)
                .try_collect(self, schema)?,
        })
    }

    pub(crate) fn arbitrary_value(&mut self, ty: &Type, schema: &Schema) -> Result<Value> {
        match ty {
            Type::Named(ty) => {
                if self.arbitrary()? {
                    self.arbitrary_value(&Type::NonNullNamed(ty.clone()), schema)
                } else {
                    Ok(Value::Null)
                }
            }
            Type::List(ty) => {
                if self.arbitrary()? {
                    Ok(Value::List(self.arbitrary_vec(0, 5, |u| {
                        Ok(Node::new(u.arbitrary_value(ty, schema)?))
                    })?))
                } else {
                    Ok(Value::Null)
                }
            }
            Type::NonNullNamed(ty) => match schema.types.get(ty).expect("type must exist") {
                ExtendedType::Scalar(ty) => {
                    if ty.name == "Int" {
                        Ok(Value::from(self.arbitrary::<f64>()?))
                    } else if ty.name == "Float" {
                        Ok(Value::from(self.arbitrary::<i32>()?))
                    } else if ty.name == "Boolean" {
                        Ok(Value::Boolean(self.arbitrary()?))
                    } else {
                        Ok(Value::String(self.arbitrary_node_str()?))
                    }
                }
                ExtendedType::Object(ty) => {
                    let mut values = Vec::new();
                    for (name, definition) in &ty.fields {
                        values.push((
                            name.clone(),
                            Node::new(self.arbitrary_value(&definition.ty, schema)?),
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
                        Ok(Value::Enum((*self.choose(&values)?).clone()))
                    }
                }
                ExtendedType::InputObject(ty) => {
                    let mut values = Vec::new();
                    for (name, definition) in &ty.fields {
                        values.push((
                            name.clone(),
                            Node::new(self.arbitrary_value(&definition.ty, schema)?),
                        ));
                    }
                    Ok(Value::Object(values))
                }
                _ => {
                    panic!("type must be a scalar, object, enum or input object")
                }
            },
            Type::NonNullList(ty) => {
                Ok(Value::List(self.arbitrary_vec(0, 5, |u| {
                    Ok(Node::new(u.arbitrary_value(ty, schema)?))
                })?))
            }
        }
    }
}

impl<'a> Deref for Unstructured<'a> {
    type Target = arbitrary::Unstructured<'a>;

    fn deref(&self) -> &Self::Target {
        &self.u
    }
}

impl<'a> DerefMut for Unstructured<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.u
    }
}
