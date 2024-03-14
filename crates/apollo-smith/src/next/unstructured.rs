use std::ops::{Deref, DerefMut};

use arbitrary::Result;

use apollo_compiler::ast::{
    Argument, DirectiveDefinition, DirectiveLocation, EnumTypeDefinition, EnumValueDefinition,
    Field, FieldDefinition, FragmentSpread, InlineFragment, InputObjectTypeDefinition,
    InputValueDefinition, InterfaceTypeDefinition, Name, ObjectTypeDefinition, OperationDefinition,
    OperationType, SchemaDefinition, Selection, Type, UnionTypeDefinition, Value,
    VariableDefinition,
};
use apollo_compiler::schema::{ExtendedType, InterfaceType, ObjectType};
use apollo_compiler::{Node, NodeStr, Schema};

use crate::next::ast::directive_definition::DirectiveDefinitionIterExt;
use crate::next::ast::DefinitionHasFields;
use crate::next::schema::extended_type::{ExtendedTypeExt, ExtendedTypeKind};
use crate::next::schema::object_type::ObjectTypeExt;
use crate::next::schema::schema::SchemaExt;
pub struct Unstructured<'a> {
    u: arbitrary::Unstructured<'a>,
    counter: usize,
}

impl Unstructured<'_> {
    pub fn new<'a>(data: &'a [u8]) -> Unstructured<'a> {
        Unstructured {
            u: arbitrary::Unstructured::new(data),
            counter: 0,
        }
    }

    pub(crate) fn arbitrary_vec<T, C: Fn(&mut Self) -> Result<T>>(
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
    pub(crate) fn arbitrary_optional<T, C: Fn(&mut Self) -> Result<T>>(
        &mut self,
        callback: C,
    ) -> Result<Option<T>> {
        if self.arbitrary()? {
            Ok(Some(callback(self)?))
        } else {
            Ok(None)
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
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
                self.arbitrary_optional(|u| {
                    Ok(Node::new((
                        OperationType::Mutation,
                        schema.random_object_type(u)?.name.clone(),
                    )))
                })?,
                self.arbitrary_optional(|u| {
                    Ok(Node::new((
                        OperationType::Subscription,
                        schema.random_object_type(u)?.name.clone(),
                    )))
                })?,
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
        let implements = schema.sample_interface_types(self)?;
        let implements_fields = Self::all_fields_from_interfaces(&implements);
        let new_fields = self.arbitrary_vec(1, 5, |u| {
            Ok(Node::new(u.arbitrary_field_definition(
                schema,
                DirectiveLocation::InputFieldDefinition,
            )?))
        })?;
        Ok(ObjectTypeDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
            fields: new_fields
                .into_iter()
                .chain(implements_fields.into_iter())
                .collect(),
        })
    }

    pub(crate) fn arbitrary_directive_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<DirectiveDefinition> {
        Ok(DirectiveDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
        let default_value =
            self.arbitrary_optional(|u| Ok(Node::new(u.arbitrary_value(schema, &ty)?)))?;

        Ok(InputValueDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
            name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Enum)
                .try_collect(self, schema)?,
            values: self.arbitrary_vec(0, 5, |u| {
                Ok(Node::new(EnumValueDefinition {
                    description: u.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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
        // All interfaces need to have all the fields from the interfaces they implement.
        let implements = schema.sample_interface_types(self)?;
        let implements_fields = Self::all_fields_from_interfaces(&implements);
        let new_fields = self.arbitrary_vec(1, 5, |u| {
            Ok(Node::new(u.arbitrary_field_definition(
                schema,
                DirectiveLocation::InputFieldDefinition,
            )?))
        })?;

        Ok(InterfaceTypeDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
            name: self.unique_name(),
            implements_interfaces: implements
                .iter()
                .map(|interface| interface.name.clone())
                .collect(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Interface)
                .try_collect(self, schema)?,
            fields: new_fields
                .into_iter()
                .chain(implements_fields.into_iter())
                .collect(),
        })
    }

    fn all_fields_from_interfaces(
        implements: &Vec<&Node<InterfaceType>>,
    ) -> Vec<Node<FieldDefinition>> {
        let implements_fields = implements
            .iter()
            .flat_map(|interface| interface.fields.values())
            .map(|field| field.deref().clone())
            .collect::<Vec<_>>();
        implements_fields
    }

    pub(crate) fn arbitrary_field_definition(
        &mut self,
        schema: &Schema,
        directive_location: DirectiveLocation,
    ) -> Result<FieldDefinition> {
        Ok(FieldDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
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

    pub(crate) fn arbitrary_value(&mut self, schema: &Schema, ty: &Type) -> Result<Value> {
        match ty {
            Type::Named(ty) => {
                if self.arbitrary()? {
                    self.arbitrary_value(schema, &Type::NonNullNamed(ty.clone()))
                } else {
                    Ok(Value::Null)
                }
            }
            Type::List(ty) => {
                if self.arbitrary()? {
                    Ok(Value::List(self.arbitrary_vec(0, 5, |u| {
                        Ok(Node::new(u.arbitrary_value(schema, ty)?))
                    })?))
                } else {
                    Ok(Value::Null)
                }
            }
            Type::NonNullNamed(ty) => match schema.types.get(ty).expect("type must exist") {
                ExtendedType::Scalar(ty) => {
                    if ty.name == "Int" {
                        Ok(Value::from(self.arbitrary::<i32>()?))
                    } else if ty.name == "Float" {
                        loop {
                            // not ideal, but graphql requires finite values is not a valid value.
                            let val = self.arbitrary::<f64>()?;
                            if val.is_finite() {
                                return Ok(Value::from(val));
                            }
                        }
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
                            Node::new(self.arbitrary_value(schema, &definition.ty)?),
                        ));
                    }
                    Ok(Value::Object(values))
                }
                ExtendedType::Enum(ty) => {
                    let values = ty
                        .values
                        .iter()
                        .map(|(_name, v)| &v.value)
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
                            Node::new(self.arbitrary_value(schema, &definition.ty)?),
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
                    Ok(Node::new(u.arbitrary_value(schema, ty)?))
                })?))
            }
        }
    }

    pub(crate) fn arbitrary_operation_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<OperationDefinition> {
        let operation = schema.random_query_mutation_subscription(self)?;

        Ok(OperationDefinition {
            operation_type: operation
                .deref()
                .operation_type()
                .expect("top level operation must have type"),
            name: self.arbitrary_optional(|u| Ok(u.unique_name()))?,
            variables: vec![],
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Field)
                .try_collect(self, schema)?,
            selection_set: self.arbitrary_vec(0, 5, |u| {
                Ok(u.arbitrary_selection(schema, operation.deref())?)
            })?,
        })
    }

    fn arbitrary_variable_definition(&mut self, schema: &Schema) -> Result<VariableDefinition> {
        let ty = schema
            .random_type(self, vec![ExtendedTypeKind::InputObjectTypeDefinition])?
            .ty(self)?;
        Ok(VariableDefinition {
            name: self.unique_name(),
            ty: Node::new(ty),
            default_value: self
                .arbitrary_optional(|u| Ok(Node::new(u.arbitrary_value(schema, &ty)?)))?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::ArgumentDefinition)
                .try_collect(self, schema)?,
        })
    }

    fn arbitrary_inline_fragment(&mut self, schema: &Schema) -> Result<InlineFragment> {
        let ty = schema.random_type(
            self,
            vec![ExtendedTypeKind::Object, ExtendedTypeKind::Interface],
        )?;

        Ok(InlineFragment {
            type_condition: self.arbitrary_optional(|u| Ok(ty.name().clone()))?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::InlineFragment)
                .try_collect(self, schema)?,
            selection_set: self.arbitrary_vec(0, 5, |u| Ok(u.arbitrary_selection(schema, ty)?))?,
        })
    }

    fn arbitrary_fragment_spread(&mut self, schema: &Schema) -> Result<FragmentSpread> {
        Ok(FragmentSpread {
            fragment_name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::FragmentSpread)
                .try_collect(self, schema)?,
        })
    }

    fn arbitrary_selection(
        &mut self,
        schema: &Schema,
        object_type: &dyn super::schema::TypeHasFields,
    ) -> Result<Selection> {
        if let Some(field) = object_type.random_field(self)? {
            match self.choose_index(3) {
                Ok(0) => Ok(Selection::Field(Node::new(self.arbitrary_field(schema)?))),
                Ok(1) => Ok(Selection::FragmentSpread(Node::new(
                    self.arbitrary_fragment_spread(schema)?,
                ))),
                Ok(2) => Ok(Selection::InlineFragment(Node::new(
                    self.arbitrary_inline_fragment(schema)?,
                ))),
                _ => unreachable!(),
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
