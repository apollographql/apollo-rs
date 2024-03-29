use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut};

use arbitrary::Result;

use apollo_compiler::ast::{
    Argument, DirectiveDefinition, DirectiveLocation, EnumTypeDefinition, EnumValueDefinition,
    Field, FieldDefinition, FragmentDefinition, FragmentSpread, InlineFragment,
    InputObjectTypeDefinition, InputValueDefinition, InterfaceTypeDefinition, Name,
    ObjectTypeDefinition, OperationDefinition, OperationType, ScalarTypeDefinition,
    SchemaDefinition, Selection, Type, UnionTypeDefinition, Value, VariableDefinition,
};
use apollo_compiler::schema::{ExtendedType, InterfaceType};
use apollo_compiler::{ExecutableDocument, Node, NodeStr, Schema};

use crate::next::ast::directive_definition::DirectiveDefinitionIterExt;
use crate::next::executable::executable_document::ExecutableDocumentExt;
use crate::next::schema::extended_type::{ExtendedTypeExt, ExtendedTypeKind};
use crate::next::schema::object_type::ObjectTypeExt;
use crate::next::schema::schema::SchemaExt;
use crate::next::schema::Selectable;

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
        let implements =
            Self::all_transitive_interfaces(schema, schema.sample_interface_types(self)?);
        let implements_fields = Self::all_unique_fields_from_interfaces(&implements);
        let new_fields = self.arbitrary_vec(1, 5, |u| {
            Ok(Node::new(u.arbitrary_field_definition(
                schema,
                DirectiveLocation::FieldDefinition,
            )?))
        })?;
        Ok(ObjectTypeDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
            name: self.unique_name(),
            implements_interfaces: implements.iter().map(|i| i.name.clone()).collect(),
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
                Ok(Node::new(u.arbitrary_input_value_definition(
                    schema,
                    DirectiveLocation::ArgumentDefinition,
                )?))
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
                Ok(Node::new(u.arbitrary_input_value_definition(
                    schema,
                    DirectiveLocation::InputFieldDefinition,
                )?))
            })?,
        })
    }

    pub(crate) fn arbitrary_input_value_definition(
        &mut self,
        schema: &Schema,
        location: DirectiveLocation,
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
                .with_location(location)
                .try_collect(self, schema)?,
        })
    }

    pub(crate) fn arbitrary_scalar_type_definition(
        &mut self,
        schema: &Schema,
    ) -> Result<ScalarTypeDefinition> {
        Ok(ScalarTypeDefinition {
            description: self.arbitrary_optional(|u| u.arbitrary_node_str())?,
            name: self.unique_name(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Scalar)
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
        let implements =
            Self::all_transitive_interfaces(schema, schema.sample_interface_types(self)?);

        // Interfaces cannot have duplicate fields so stash them in a map
        let mut implements_fields = Self::all_unique_fields_from_interfaces(&implements);

        let new_fields = self.arbitrary_vec(1, 5, |u| {
            Ok(Node::new(u.arbitrary_field_definition(
                schema,
                DirectiveLocation::FieldDefinition,
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

    fn all_unique_fields_from_interfaces(
        interfaces: &Vec<&Node<InterfaceType>>,
    ) -> Vec<Node<FieldDefinition>> {
        let all_fields = interfaces
            .iter()
            .flat_map(|interface| interface.fields.values())
            .map(|field| (field.name.clone(), field.deref().clone()))
            .collect::<HashMap<_, _>>();
        all_fields.values().cloned().collect()
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
                Ok(Node::new(u.arbitrary_input_value_definition(
                    schema,
                    DirectiveLocation::ArgumentDefinition,
                )?))
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
        executable_document: &ExecutableDocument,
        name: Option<Name>,
    ) -> Result<OperationDefinition> {
        let object = schema.random_query_mutation_subscription(self)?;
        let directive_location = match object.name.as_ref() {
            "Query" => DirectiveLocation::Query,
            "Mutation" => DirectiveLocation::Mutation,
            "Subscription" => DirectiveLocation::Subscription,
            _ => panic!("invalid object name"),
        };
        Ok(OperationDefinition {
            operation_type: object
                .deref()
                .operation_type()
                .expect("top level operation must have type"),
            name,
            variables: vec![],
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(directive_location)
                .try_collect(self, schema)?,
            selection_set: self.arbitrary_vec(1, 5, |u| {
                Ok(u.arbitrary_selection(schema, object.deref(), executable_document)?)
            })?,
        })
    }

    fn arbitrary_variable_definition(&mut self, schema: &Schema) -> Result<VariableDefinition> {
        let ty = schema
            .random_type(self, vec![ExtendedTypeKind::InputObjectTypeDefinition])?
            .ty(self)?;
        Ok(VariableDefinition {
            name: self.unique_name(),
            default_value: self
                .arbitrary_optional(|u| Ok(Node::new(u.arbitrary_value(schema, &ty)?)))?,
            ty: Node::new(ty),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::ArgumentDefinition)
                .try_collect(self, schema)?,
        })
    }

    pub(crate) fn arbitrary_fragment_definition(
        &mut self,
        schema: &Schema,
        executable_document: &ExecutableDocument,
    ) -> Result<FragmentDefinition> {
        let type_condition = schema.random_type(
            self,
            vec![
                ExtendedTypeKind::Object,
                ExtendedTypeKind::Interface,
                ExtendedTypeKind::Union,
            ],
        )?;
        Ok(FragmentDefinition {
            name: self.unique_name(),
            type_condition: type_condition.name().clone(),
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::FragmentDefinition)
                .try_collect(self, schema)?,
            selection_set: self.arbitrary_vec(1, 5, |u| {
                Ok(u.arbitrary_selection(schema, type_condition, executable_document)?)
            })?,
        })
    }

    pub(crate) fn arbitrary_inline_fragment<'a>(
        &mut self,
        schema: &Schema,
        selectable: &impl Selectable,
        executable_document: &ExecutableDocument,
    ) -> Result<InlineFragment> {
        Ok(InlineFragment {
            type_condition: self.arbitrary_optional(|_| Ok(selectable.name().clone()))?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::InlineFragment)
                .try_collect(self, schema)?,
            selection_set: self.arbitrary_vec(1, 5, |u| {
                Ok(u.arbitrary_selection(schema, selectable, executable_document)?)
            })?,
        })
    }

    fn arbitrary_selection<'a>(
        &mut self,
        schema: &Schema,
        selectable: &impl Selectable,
        executable_document: &ExecutableDocument,
    ) -> Result<Selection> {
        // The selection must contain at least one field, fragment spread or inline fragment
        // If the type is a union then it must contain at least one inline fragment

        let selection_type = *self.choose(&vec![
            SelectionType::Field,
            SelectionType::InlineFragment,
            SelectionType::FragmentSpread,
        ])?;

        if selection_type == SelectionType::FragmentSpread {
            if let Some(fragment) = executable_document.random_fragment_of_type(self, selectable)? {
                return Ok(Selection::FragmentSpread(Node::new(FragmentSpread {
                    fragment_name: fragment.name.clone(),
                    directives: schema
                        .sample_directives(self)?
                        .into_iter()
                        .with_location(DirectiveLocation::Field)
                        .try_collect(self, schema)?,
                })));
            }
        }
        if selection_type == SelectionType::InlineFragment {
            if let Some(specialization) = selectable.random_specialization(self, schema)? {
                return Ok(Selection::InlineFragment(Node::new(
                    self.arbitrary_inline_fragment(schema, specialization, executable_document)?,
                )));
            }
        }

        let field = selectable.random_field(self)?;
        let field_ty = schema
            .types
            .get(field.ty.inner_named_type())
            .expect("type must exist");
        let selection_set = if field_ty.is_scalar() {
            vec![]
        } else {
            self.arbitrary_vec(1, 5, |u| {
                Ok(u.arbitrary_selection(schema, field_ty, executable_document)?)
            })?
        };
        Ok(Selection::Field(Node::new(Field {
            alias: self.arbitrary_optional(|u| Ok(u.unique_name()))?,
            name: field.name.clone(),
            arguments: self.arbitrary_arguments(schema, field)?,
            directives: schema
                .sample_directives(self)?
                .into_iter()
                .with_location(DirectiveLocation::Field)
                .try_collect(self, schema)?,
            selection_set,
        })))
    }
    fn arbitrary_arguments(
        &mut self,
        schema: &Schema,
        field_definition: &FieldDefinition,
    ) -> Result<Vec<Node<Argument>>> {
        let mut args = Vec::new();
        for arg in &field_definition.arguments {
            args.push(Node::new(Argument {
                name: arg.name.clone(),
                value: Node::new(self.arbitrary_value(schema, &field_definition.ty)?),
            }));
        }
        Ok(args)
    }

    fn all_transitive_interfaces<'a>(
        schema: &'a Schema,
        interfaces: Vec<&'a Node<InterfaceType>>,
    ) -> Vec<&'a Node<InterfaceType>> {
        // In graphql interfaces can extend other interfaces, but when using them you need to specify every single one in the entire type hierarchy.
        interfaces
            .into_iter()
            .flat_map(|interface| {
                std::iter::once(&interface.name).chain(
                    interface
                        .implements_interfaces
                        .iter()
                        .map(|component| &component.name),
                )
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .filter_map(|interface| schema.get_interface(interface))
            .collect()
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
enum SelectionType {
    Field,
    FragmentSpread,
    InlineFragment,
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
