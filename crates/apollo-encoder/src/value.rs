use std::fmt;

/// The __value type represents available values you could give as an input.
///
/// *Value*:
///     Variable | IntValue | FloatValue | StringValue | BooleanValue | NullValue | EnumValue | ListValue | ObjectValue
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#Value).
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// Name of a variable example: `varName`
    Variable(String),
    /// Int value example: `7`
    Int(i64),
    /// Float value example: `25.4`
    Float(f64),
    /// String value example: `"My string"`
    String(String),
    /// Boolean value example: `false`
    Boolean(bool),
    /// Null value example: `null`
    Null,
    /// Enum value example: `"VARIANT_EXAMPLE"`
    Enum(String),
    /// List value example: `[1, 2, 3]`
    List(Vec<Value>),
    /// Object value example: `{ first: 1, second: 2 }`
    Object(Vec<(String, Value)>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Variable(v) => write!(f, "${v}"),
            Self::Int(i) => write!(f, "{i}"),
            Self::Float(fl) => write!(f, "{fl}"),
            Self::String(s) => {
                if s.contains('"') | s.contains('\n') | s.contains('\r') {
                    write!(f, r#""""{s}""""#)
                } else {
                    write!(f, r#""{s}""#)
                }
            }
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Null => write!(f, "null"),
            Self::Enum(val) => write!(f, "{val}"),
            Self::List(list) => write!(
                f,
                "[{}]",
                list.iter()
                    .map(|elt| format!("{elt}"))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Self::Object(obj) => write!(
                f,
                "{{ {} }}",
                obj.iter()
                    .map(|(k, v)| format!("{}: {v}", String::from(k)))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
        }
    }
}

macro_rules! to_number_value {
    ($ty: path, $inner_type: path, $value_variant: ident) => {
        impl From<$ty> for Value {
            fn from(val: $ty) -> Self {
                Self::$value_variant(val as $inner_type)
            }
        }
    };
    ($({$ty: path, $inner_type: path, $value_variant: ident}),+) => {
        $(
            to_number_value!($ty, $inner_type, $value_variant);
        )+
    };
}

// Numbers
to_number_value!(
    {i64, i64, Int},
    {i32, i64, Int},
    {i16, i64, Int},
    {i8, i64, Int},
    {isize, i64, Int},
    {u64, i64, Int},
    {u32, i64, Int},
    {u16, i64, Int},
    {u8, i64, Int},
    {usize, i64, Int},
    {f64, f64, Float},
    {f32, f64, Float}
);

impl From<String> for Value {
    fn from(val: String) -> Self {
        Self::String(val)
    }
}

impl From<&str> for Value {
    fn from(val: &str) -> Self {
        Self::String(val.to_string())
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Self::Boolean(val)
    }
}
