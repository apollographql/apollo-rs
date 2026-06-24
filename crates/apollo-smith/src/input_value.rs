use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::name::Name;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy)]
pub enum Constness {
    Const,
    NonConst,
}

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

impl From<InputValue> for ast::Value {
    fn from(input_value: InputValue) -> Self {
        match input_value {
            InputValue::Variable(v) => Self::Variable(v.into()),
            InputValue::Int(i) => Self::Int(i.into()),
            InputValue::Float(f) => Self::Float(f.into()),
            InputValue::String(s) => Self::String(s),
            InputValue::Boolean(b) => Self::Boolean(b),
            InputValue::Null => Self::Null,
            InputValue::Enum(enm) => Self::Enum(enm.into()),
            InputValue::List(l) => Self::List(l.into_iter().map(|v| Node::new(v.into())).collect()),
            InputValue::Object(o) => Self::Object(
                o.into_iter()
                    .map(|(n, i)| (n.into(), Node::new(i.into())))
                    .collect(),
            ),
        }
    }
}

impl TryFrom<apollo_parser::cst::DefaultValue> for InputValue {
    type Error = crate::FromError;

    fn try_from(default_val: apollo_parser::cst::DefaultValue) -> Result<Self, Self::Error> {
        default_val.value().unwrap().try_into()
    }
}

impl TryFrom<apollo_parser::cst::Value> for InputValue {
    type Error = crate::FromError;

    fn try_from(value: apollo_parser::cst::Value) -> Result<Self, Self::Error> {
        let smith_value = match value {
            apollo_parser::cst::Value::Variable(variable) => {
                Self::Variable(variable.name().unwrap().into())
            }
            apollo_parser::cst::Value::StringValue(val) => Self::String(val.into()),
            apollo_parser::cst::Value::FloatValue(val) => Self::Float(val.try_into()?),
            apollo_parser::cst::Value::IntValue(val) => Self::Int(val.try_into()?),
            apollo_parser::cst::Value::BooleanValue(val) => Self::Boolean(val.try_into()?),
            apollo_parser::cst::Value::NullValue(_val) => Self::Null,
            apollo_parser::cst::Value::EnumValue(val) => Self::Enum(val.name().unwrap().into()),
            apollo_parser::cst::Value::ListValue(val) => Self::List(
                val.values()
                    .map(Self::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            apollo_parser::cst::Value::ObjectValue(val) => Self::Object(
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
    pub(crate) directives: IndexMap<Name, Directive>,
}

impl From<InputValueDef> for ast::InputValueDefinition {
    fn from(x: InputValueDef) -> Self {
        Self {
            description: x.description.map(Into::into),
            name: x.name.into(),
            ty: Node::new(x.ty.into()),
            default_value: x.default_value.map(|x| Node::new(x.into())),
            directives: Directive::to_ast(x.directives),
        }
    }
}

impl TryFrom<apollo_parser::cst::InputValueDefinition> for InputValueDef {
    type Error = crate::FromError;

    fn try_from(
        input_val_def: apollo_parser::cst::InputValueDefinition,
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

impl DocumentBuilder<'_> {
    /// Create an arbitrary `InputValue`
    pub fn input_value(&mut self, constness: Constness) -> ArbitraryResult<InputValue> {
        let index = match constness {
            Constness::Const => self.u.int_in_range(0..=7usize)?,
            Constness::NonConst => self.u.int_in_range(0..=8usize)?,
        };
        let val = match index {
            // Int
            0 => InputValue::Int(self.u.arbitrary()?),
            // Float
            1 => InputValue::Float(self.finite_f64()?),
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
                    self.input_value(constness)?
                }
            }
            // List
            6 => {
                // FIXME: it's semantically wrong it should always be the same type inside
                InputValue::List(
                    (0..self.u.int_in_range(2..=4usize)?)
                        .map(|_| self.input_value(constness))
                        .collect::<ArbitraryResult<Vec<_>>>()?,
                )
            }
            // Object
            7 => InputValue::Object(
                (0..self.u.int_in_range(2..=4usize)?)
                    .map(|_| Ok((self.name()?, self.input_value(constness)?)))
                    .collect::<ArbitraryResult<Vec<_>>>()?,
            ),
            // Variable TODO: only generate valid variable name (existing variables)
            8 => InputValue::Variable(self.name()?),
            _ => unreachable!(),
        };

        Ok(val)
    }

    pub fn input_value_for_type(&mut self, ty: &Ty) -> ArbitraryResult<InputValue> {
        let gen_val = |doc_builder: &mut DocumentBuilder<'_>| -> ArbitraryResult<InputValue> {
            if ty.is_builtin() {
                match ty.name().name.as_str() {
                    "String" => Ok(InputValue::String(doc_builder.limited_string(1000)?)),
                    "Int" => Ok(InputValue::Int(doc_builder.u.arbitrary()?)),
                    "Float" => Ok(InputValue::Float(doc_builder.finite_f64()?)),
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
            } else if let Some(input_object_ty) = doc_builder
                .input_object_type_defs
                .iter()
                .find(|io| &io.name == ty.name())
                .cloned()
            {
                Ok(InputValue::Object(
                    input_object_ty
                        .fields
                        .iter()
                        .map(|field_def| {
                            Ok((
                                field_def.name.clone(),
                                doc_builder.input_value_for_type(&field_def.ty)?,
                            ))
                        })
                        .collect::<ArbitraryResult<Vec<_>>>()?,
                ))
            } else if doc_builder
                .scalar_type_defs
                .iter()
                .any(|s| &s.name == ty.name())
            {
                // Custom scalars accept any literal value; generate an Int to be entropy-efficient
                Ok(InputValue::Int(doc_builder.u.arbitrary()?))
            } else {
                panic!("Type {} is not a valid input type", ty.name().name);
            }
        };

        let val = match ty {
            Ty::Named(_) => gen_val(self)?,
            Ty::List(_) => {
                let nb_elt = self.u.int_in_range(1..=25usize)?;
                InputValue::List(
                    (0..nb_elt)
                        .map(|_| gen_val(self))
                        .collect::<ArbitraryResult<Vec<InputValue>>>()?,
                )
            }
            Ty::NonNull(_) => gen_val(self)?,
        };

        Ok(val)
    }

    /// Create an arbitrary list of `InputValueDef`. The caller passes
    /// `directive_location` so directives applied to each value are
    /// filtered against the right context — `InputFieldDefinition` for
    /// input-object fields, `ArgumentDefinition` for argument lists on
    /// fields and directives.
    pub fn input_values_def(
        &mut self,
        directive_location: DirectiveLocation,
        exclude: &[&Name],
    ) -> ArbitraryResult<Vec<InputValueDef>> {
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
            let ty = self.choose_ty(&self.list_existing_input_types())?;
            let directives = self.directives(directive_location)?;
            // TODO: FIXME: it's not correct I need to generate default value corresponding to the ty above
            let default_value = self
                .u
                .arbitrary()
                .unwrap_or(false)
                .then(|| self.input_value(Constness::Const))
                .transpose()?;

            if !exclude.contains(&&name) {
                input_values.push(InputValueDef {
                    description,
                    name,
                    ty,
                    default_value,
                    directives,
                });
            }
        }

        Ok(input_values)
    }
    /// Create an arbitrary `InputValueDef`. The caller passes
    /// `directive_location` for the same reason as
    /// [`input_values_def`](Self::input_values_def).
    pub fn input_value_def(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<InputValueDef> {
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = self.name()?;
        let ty = self.choose_ty(&self.list_existing_input_types())?;
        let directives = self.directives(directive_location)?;
        // TODO: FIXME: it's not correct I need to generate default value corresponding to the ty above
        let default_value = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.input_value(Constness::Const))
            .transpose()?;

        Ok(InputValueDef {
            description,
            name,
            ty,
            default_value,
            directives,
        })
    }

    fn finite_f64(&mut self) -> arbitrary::Result<f64> {
        loop {
            let val: f64 = self.u.arbitrary()?;
            if val.is_finite() {
                return Ok(val);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::InputObjectTypeDef;
    use arbitrary::Unstructured;
    use indexmap::IndexMap;

    #[test]
    fn test_input_value_for_type() {
        let data: Vec<u8> = (0..=5000usize).map(|n| (n % 255) as u8).collect();
        let mut u = Unstructured::new(&data);
        let mut document_builder = DocumentBuilder::new(&mut u);
        let my_nested_type = InputObjectTypeDef {
            description: None,
            name: Name {
                name: String::from("my_nested_object"),
            },
            directives: IndexMap::new(),
            fields: vec![InputValueDef {
                description: None,
                name: Name {
                    name: String::from("value"),
                },
                ty: Ty::Named(Name {
                    name: String::from("String"),
                }),
                default_value: None,
                directives: IndexMap::new(),
            }],
            extend: false,
        };

        let my_object_type = InputObjectTypeDef {
            description: None,
            name: Name {
                name: String::from("my_object"),
            },
            directives: IndexMap::new(),
            fields: vec![InputValueDef {
                description: None,
                name: Name {
                    name: String::from("first"),
                },
                ty: Ty::List(Box::new(Ty::Named(Name {
                    name: String::from("my_nested_object"),
                }))),
                default_value: None,
                directives: IndexMap::new(),
            }],
            extend: false,
        };
        document_builder.input_object_type_defs.push(my_nested_type);
        document_builder.input_object_type_defs.push(my_object_type);

        let input_val = document_builder
            .input_value_for_type(&Ty::List(Box::new(Ty::Named(Name {
                name: String::from("my_object"),
            }))))
            .unwrap();

        let input_val_str = apollo_compiler::ast::Value::from(input_val)
            .serialize()
            .no_indent()
            .to_string();

        assert_eq!(
            input_val_str.as_str(),
            "[{first: [{value: \"EFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCABCDEFGHIJ\"}, {value: \"MNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_0123456789ABCDEFGHIJK\"}]}]"
        );
    }
}
