use std::collections::HashMap;

use crate::{
    description::Description,
    directive::{Directive, DirectiveLocation},
    name::Name,
    ty::Ty,
    DocumentBuilder,
};
use arbitrary::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum InputValue {
    Variable(Name),
    Int(i32),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Enum(Name),
    List(Vec<InputValue>),
    Object(Vec<(Name, InputValue)>),
}

impl From<InputValue> for apollo_encoder::Value {
    fn from(input_value: InputValue) -> Self {
        match input_value {
            InputValue::Variable(v) => Self::Variable(v.into()),
            InputValue::Int(i) => Self::Int(i),
            InputValue::Float(f) => Self::Float(f),
            InputValue::String(s) => Self::String(s),
            InputValue::Boolean(b) => Self::Boolean(b),
            InputValue::Null => Self::Null,
            InputValue::Enum(enm) => Self::Enum(enm.into()),
            InputValue::List(l) => Self::List(l.into_iter().map(Into::into).collect()),
            InputValue::Object(o) => {
                Self::Object(o.into_iter().map(|(n, i)| (n.into(), i.into())).collect())
            }
        }
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::DefaultValue> for InputValue {
    type Error = crate::FromError;

    fn try_from(default_val: apollo_parser::ast::DefaultValue) -> Result<Self, Self::Error> {
        default_val.value().unwrap().try_into()
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::Value> for InputValue {
    type Error = crate::FromError;

    fn try_from(value: apollo_parser::ast::Value) -> Result<Self, Self::Error> {
        let smith_value = match value {
            apollo_parser::ast::Value::Variable(variable) => {
                Self::Variable(variable.name().unwrap().into())
            }
            apollo_parser::ast::Value::StringValue(val) => Self::String(val.try_into().unwrap()),
            apollo_parser::ast::Value::FloatValue(val) => Self::Float(val.try_into()?),
            apollo_parser::ast::Value::IntValue(val) => Self::Int(val.try_into()?),
            apollo_parser::ast::Value::BooleanValue(val) => Self::Boolean(val.try_into()?),
            apollo_parser::ast::Value::NullValue(_val) => Self::Null,
            apollo_parser::ast::Value::EnumValue(val) => Self::Enum(val.name().unwrap().into()),
            apollo_parser::ast::Value::ListValue(val) => Self::List(
                val.values()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            apollo_parser::ast::Value::ObjectValue(val) => Self::Object(
                val.object_fields()
                    .map(|of| Ok((of.name().unwrap().into(), of.value().unwrap().try_into()?)))
                    .collect::<Result<Vec<_>, crate::FromError>>()?,
            ),
        };
        Ok(smith_value)
    }
}

impl From<InputValue> for String {
    fn from(input_val: InputValue) -> Self {
        match input_val {
            InputValue::Variable(v) => format!("${}", String::from(v)),
            InputValue::Int(i) => format!("{i}"),
            InputValue::Float(f) => format!("{f}"),
            InputValue::String(s) => s,
            InputValue::Boolean(b) => format!("{b}"),
            InputValue::Null => String::from("null"),
            InputValue::Enum(val) => val.into(),
            InputValue::List(list) => format!(
                "[{}]",
                list.into_iter()
                    .map(String::from)
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            InputValue::Object(obj) => format!(
                "{{ {} }}",
                obj.into_iter()
                    .map(|(k, v)| format!("{}: {}", String::from(k), String::from(v)))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

/// The __InputValueDef type represents field and directive arguments.
///
/// *InputValueDefinition*:
///     Description? Name **:** Type DefaultValue? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-The-__InputValue-Type).
#[derive(Debug, Clone, PartialEq)]
pub struct InputValueDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) ty: Ty,
    pub(crate) default_value: Option<InputValue>,
    pub(crate) directives: HashMap<Name, Directive>,
}

impl From<InputValueDef> for apollo_encoder::InputValueDefinition {
    fn from(input_val: InputValueDef) -> Self {
        let mut new_input_val = Self::new(input_val.name.into(), input_val.ty.into());
        if let Some(description) = input_val.description {
            new_input_val.description(description.into())
        }
        if let Some(default) = input_val.default_value {
            new_input_val.default_value(default.into())
        }
        input_val
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_input_val.directive(directive.into()));

        new_input_val
    }
}

#[cfg(feature = "parser-impl")]
impl TryFrom<apollo_parser::ast::InputValueDefinition> for InputValueDef {
    type Error = crate::FromError;

    fn try_from(
        input_val_def: apollo_parser::ast::InputValueDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            description: input_val_def.description().map(Description::from),
            name: input_val_def.name().unwrap().into(),
            ty: input_val_def.ty().unwrap().into(),
            default_value: input_val_def
                .default_value()
                .map(InputValue::try_from)
                .transpose()?,
            directives: input_val_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl From<InputValueDef> for apollo_encoder::InputField {
    fn from(input_val: InputValueDef) -> Self {
        let mut new_input_val = Self::new(input_val.name.into(), input_val.ty.into());
        if let Some(description) = input_val.description {
            new_input_val.description(description.into())
        }
        if let Some(default) = input_val.default_value {
            new_input_val.default_value(default.into())
        }
        input_val
            .directives
            .into_iter()
            .for_each(|(_, directive)| new_input_val.directive(directive.into()));

        new_input_val
    }
}

impl<'a> DocumentBuilder<'a> {
    /// Create an arbitrary `InputValue`
    pub fn input_value(&mut self) -> Result<InputValue> {
        let val = match self.u.int_in_range(0..=8usize)? {
            // Int
            0 => InputValue::Int(self.u.arbitrary()?),
            // Float
            1 => InputValue::Float(self.u.arbitrary()?),
            // String
            2 => InputValue::String(self.limited_string(40)?),
            // Boolean
            3 => InputValue::Boolean(self.u.arbitrary()?),
            // Null
            4 => InputValue::Null,
            // Enum
            5 => {
                if !self.enum_type_defs.is_empty() {
                    // TODO get rid of this clone
                    let enum_choosed = self.choose_enum()?.clone();
                    InputValue::Enum(self.arbitrary_variant(&enum_choosed)?.clone())
                } else {
                    self.input_value()?
                }
            }
            // List
            6 => {
                // FIXME: it's semantically wrong it should always be the same type inside
                InputValue::List(
                    (0..self.u.int_in_range(2..=4usize)?)
                        .map(|_| self.input_value())
                        .collect::<Result<Vec<_>>>()?,
                )
            }
            // Object
            7 => InputValue::Object(
                (0..self.u.int_in_range(2..=4usize)?)
                    .map(|_| Ok((self.name()?, self.input_value()?)))
                    .collect::<Result<Vec<_>>>()?,
            ),
            // Variable TODO: only generate valid variable name (existing variables)
            8 => InputValue::Variable(self.name()?),
            _ => unreachable!(),
        };

        Ok(val)
    }

    pub fn input_value_for_type(&mut self, ty: &Ty) -> Result<InputValue> {
        let gen_val = |doc_builder: &mut DocumentBuilder<'_>| -> Result<InputValue> {
            if ty.is_builtin() {
                match ty.name().name.as_str() {
                    "String" => Ok(InputValue::String(doc_builder.limited_string(1000)?)),
                    "Int" => Ok(InputValue::Int(doc_builder.u.arbitrary()?)),
                    "Float" => Ok(InputValue::Float(doc_builder.u.arbitrary()?)),
                    "Boolean" => Ok(InputValue::Boolean(doc_builder.u.arbitrary()?)),
                    "ID" => Ok(InputValue::Int(doc_builder.u.arbitrary()?)),
                    other => {
                        unreachable!("{} is not a builtin", other);
                    }
                }
            } else if let Some(enum_) = doc_builder
                .enum_type_defs
                .iter()
                .find(|e| &e.name == ty.name())
                .cloned()
            {
                Ok(InputValue::Enum(
                    doc_builder.arbitrary_variant(&enum_)?.clone(),
                ))
            } else if let Some(object_ty) = doc_builder
                .object_type_defs
                .iter()
                .find(|o| &o.name == ty.name())
                .cloned()
            {
                Ok(InputValue::Object(
                    object_ty
                        .fields_def
                        .iter()
                        .map(|field_def| {
                            Ok((
                                field_def.name.clone(),
                                doc_builder.input_value_for_type(&field_def.ty)?,
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?,
                ))
            } else {
                todo!()
            }
        };

        let val = match ty {
            Ty::Named(_) => gen_val(self)?,
            Ty::List(_) => {
                let nb_elt = self.u.int_in_range(1..=25usize)?;
                InputValue::List(
                    (0..nb_elt)
                        .map(|_| gen_val(self))
                        .collect::<Result<Vec<InputValue>>>()?,
                )
            }
            Ty::NonNull(_) => gen_val(self)?,
        };

        Ok(val)
    }

    /// Create an arbitrary list of `InputValueDef`
    pub fn input_values_def(&mut self) -> Result<Vec<InputValueDef>> {
        let arbitrary_iv_num = self.u.int_in_range(2..=5usize)?;
        let mut input_values = Vec::with_capacity(arbitrary_iv_num - 1);

        for i in 0..arbitrary_iv_num {
            let description = self
                .u
                .arbitrary()
                .unwrap_or(false)
                .then(|| self.description())
                .transpose()?;
            let name = self.name_with_index(i)?;
            let ty = self.choose_ty(&self.list_existing_types())?;
            // TODO: incorrect because input_values_def is called from different locations
            let directives = self.directives(DirectiveLocation::InputFieldDefinition)?;
            // TODO: FIXME: it's not correct I need to generate default value corresponding to the ty above
            let default_value = self
                .u
                .arbitrary()
                .unwrap_or(false)
                .then(|| self.input_value())
                .transpose()?;

            input_values.push(InputValueDef {
                description,
                name,
                ty,
                default_value,
                directives,
            });
        }

        Ok(input_values)
    }
    /// Create an arbitrary `InputValueDef`
    pub fn input_value_def(&mut self) -> Result<InputValueDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.name()?;
        let ty = self.choose_ty(&self.list_existing_types())?;
        // TODO: incorrect because input_values_def is called from different locations
        let directives = self.directives(DirectiveLocation::InputFieldDefinition)?;
        // TODO: FIXME: it's not correct I need to generate default value corresponding to the ty above
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value())
            .transpose()?;

        Ok(InputValueDef {
            description,
            name,
            ty,
            default_value,
            directives,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use arbitrary::Unstructured;

    use crate::{field::FieldDef, ObjectTypeDef};

    use super::*;

    #[test]
    fn test_input_value_for_type() {
        let data: Vec<u8> = (0..=5000usize)
            .into_iter()
            .map(|n| (n % 255) as u8)
            .collect();
        let mut u = Unstructured::new(&data);
        let mut document_builder = DocumentBuilder {
            u: &mut u,
            input_object_type_defs: Vec::new(),
            object_type_defs: Vec::new(),
            interface_type_defs: Vec::new(),
            union_type_defs: Vec::new(),
            enum_type_defs: Vec::new(),
            scalar_type_defs: Vec::new(),
            schema_def: None,
            directive_defs: Vec::new(),
            operation_defs: Vec::new(),
            fragment_defs: Vec::new(),
            stack: Vec::new(),
            chosen_arguments: HashMap::new(),
            chosen_aliases: HashMap::new(),
        };
        let my_nested_type = ObjectTypeDef {
            description: None,
            name: Name {
                name: String::from("my_nested_object"),
            },
            implements_interfaces: HashSet::new(),
            directives: HashMap::new(),
            fields_def: vec![FieldDef {
                description: None,
                name: Name {
                    name: String::from("value"),
                },
                arguments_definition: None,
                ty: Ty::Named(Name {
                    name: String::from("String"),
                }),
                directives: HashMap::new(),
            }],
            extend: false,
        };

        let my_object_type = ObjectTypeDef {
            description: None,
            name: Name {
                name: String::from("my_object"),
            },
            implements_interfaces: HashSet::new(),
            directives: HashMap::new(),
            fields_def: vec![FieldDef {
                description: None,
                name: Name {
                    name: String::from("first"),
                },
                arguments_definition: None,
                ty: Ty::List(Box::new(Ty::Named(Name {
                    name: String::from("my_nested_object"),
                }))),
                directives: HashMap::new(),
            }],
            extend: false,
        };
        document_builder.object_type_defs.push(my_nested_type);
        document_builder.object_type_defs.push(my_object_type);

        let my_type_to_find = Ty::List(Box::new(Ty::Named(Name {
            name: String::from("my_object"),
        })));
        document_builder.object_type_defs.iter().find(|o| {
            let res = &o.name == my_type_to_find.name();

            res
        });

        let input_val = document_builder
            .input_value_for_type(&Ty::List(Box::new(Ty::Named(Name {
                name: String::from("my_object"),
            }))))
            .unwrap();

        let input_val_str = apollo_encoder::Value::from(input_val).to_string();

        assert_eq!(
            input_val_str.as_str(),
            "[{ first: [{ value: \"womkigecaYWUSQOMKIGECA86420zxvtcaYWUSQOMKIGECA86420zxvtrpnljhfdbKIGECA86420zxvtrpnljhfdbZXVTRPN9420zxvtrpnljhfdbZXVTRPNLJHFDB97rpnljhfdbZXVTRPNLJHFDB97531_ywugbZXVTRPNLJHFDB97531_ywusqomkigeNLJHFDB97531_ywusqomkigecaYWUSQOM531_ywusqomkigecaYWUSQOMKIGECA8vqomkigecaYWUSQOMKIGECA86420zxvtcaYWUSQOMKIGECA86420zxvtrpnljhfdbKIGECA86420zxvtrpnljhfdbZXVTRPN9420zxvtrpnljhfdbZXVTRPNLJHFDB97rpnljhfdbZXVTRPNLJHFDB97531_ywugbZXVTRPNLJHFDB97531_ywusqomkigeNLJHFDB97531_ywusqomkigecaYWUSQOM531_ywusqomkigecaYWUSQOMKIGECA8vqomki\" }, { value: \"a86420zxvtrpnljhfdbZXVTRPN9420zxvtrpnljhfdbZXVTRPNLJHFDB97rpnljhfdbZXVTRPNLJHFDB97531_ywugbZXVTRPNLJHFDB97531kAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\" }] }]"
        );
    }
}
