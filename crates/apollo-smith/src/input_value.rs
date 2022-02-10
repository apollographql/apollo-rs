use crate::{description::Description, directive::Directive, name::Name, ty::Ty, DocumentBuilder};
use arbitrary::Result;

#[derive(Debug, Clone, PartialEq)]

pub enum InputValue {
    Variable(Name),
    Int(i64),
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
    pub(crate) directives: Vec<Directive>,
}

impl From<InputValueDef> for apollo_encoder::InputValueDef {
    fn from(input_val: InputValueDef) -> Self {
        let mut new_input_val = Self::new(input_val.name.into(), input_val.ty.into());
        new_input_val.description(input_val.description.map(String::from));
        new_input_val.default(input_val.default_value.map(String::from));
        input_val
            .directives
            .into_iter()
            .for_each(|directive| new_input_val.directive(directive.into()));

        new_input_val
    }
}

impl From<InputValueDef> for apollo_encoder::InputField {
    fn from(input_val: InputValueDef) -> Self {
        let mut new_input_val = Self::new(input_val.name.into(), input_val.ty.into());
        new_input_val.description(input_val.description.map(String::from));
        new_input_val.default(input_val.default_value.map(String::from));
        input_val
            .directives
            .into_iter()
            .for_each(|directive| new_input_val.directive(directive.into()));

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
            let directives = self.directives()?;
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
        let directives = self.directives()?;
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
