use crate::hir::{HirNodeLocation, Name};
use ordered_float::{self, OrderedFloat};

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Float {
    inner: ordered_float::OrderedFloat<f64>,
}

impl Float {
    pub fn new(float: f64) -> Self {
        Self {
            inner: OrderedFloat(float),
        }
    }

    pub fn get(self) -> f64 {
        self.inner.0
    }

    /// If the value is in the `i32` range, convert by rounding towards zero.
    ///
    /// (This is mostly useful when matching on [`Value::Int`]
    /// where the value is known not to have a fractional part
    ///  so the rounding mode doesn’t affect the result.)
    pub fn to_i32_checked(self) -> Option<i32> {
        let float = self.inner.0;
        if float <= (i32::MAX as f64) && float >= (i32::MIN as f64) {
            Some(float as i32)
        } else {
            None
        }
    }
}

pub type DefaultValue = Value;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Value {
    Variable(Variable),

    // A value of integer syntax may be coerced to a Float input value:
    // https://spec.graphql.org/draft/#sec-Float.Input-Coercion
    // Keep a f64 here instead of i32 in order to support
    // the full range of f64 integer values for that case.
    //
    // All i32 values can be represented exactly in f64,
    // so conversion to an Int input value is still exact:
    // https://spec.graphql.org/draft/#sec-Int.Input-Coercion
    Int {
        value: Float,
        loc: HirNodeLocation,
    },
    Float {
        value: Float,
        loc: HirNodeLocation,
    },
    String {
        value: String,
        loc: HirNodeLocation,
    },
    Boolean {
        value: bool,
        loc: HirNodeLocation,
    },
    Null {
        loc: HirNodeLocation,
    },
    Enum {
        value: Name,
        loc: HirNodeLocation,
    },
    List {
        value: Vec<Value>,
        loc: HirNodeLocation,
    },
    Object {
        value: Vec<(Name, Value)>,
        loc: HirNodeLocation,
    },
}

impl Value {
    /// Returns `true` if `other` represents the same value as `self`. This is different from the
    /// `Eq` implementation as it ignores location information.
    pub fn is_same_value(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Variable(left), Value::Variable(right)) => left.name() == right.name(),
            (
                Value::Int { value: left, .. } | Value::Float { value: left, .. },
                Value::Int { value: right, .. } | Value::Float { value: right, .. },
            ) => left == right,
            (Value::String { value: left, .. }, Value::String { value: right, .. }) => {
                left == right
            }
            (Value::Boolean { value: left, .. }, Value::Boolean { value: right, .. }) => {
                left == right
            }
            (Value::Null { .. }, Value::Null { .. }) => true,
            (Value::Enum { value: left, .. }, Value::Enum { value: right, .. }) => {
                left.src() == right.src()
            }
            (Value::List { value: left, .. }, Value::List { value: right, .. })
                if left.len() == right.len() =>
            {
                left.iter()
                    .zip(right)
                    .all(|(left, right)| left.is_same_value(right))
            }
            (Value::Object { value: left, .. }, Value::Object { value: right, .. })
                if left.len() == right.len() =>
            {
                left.iter().zip(right).all(|(left, right)| {
                    left.0.src() == left.0.src() && left.1.is_same_value(&right.1)
                })
            }
            _ => false,
        }
    }

    /// Get current value's location.
    pub fn loc(&self) -> HirNodeLocation {
        match self {
            Value::Variable(var) => var.loc(),
            Value::Int { value: _, loc } => *loc,
            Value::Float { value: _, loc } => *loc,
            Value::String { value: _, loc } => *loc,
            Value::Boolean { value: _, loc } => *loc,
            Value::Null { loc } => *loc,
            Value::Enum { value: _, loc } => *loc,
            Value::List { value: _, loc } => *loc,
            Value::Object { value: _, loc } => *loc,
        }
    }

    pub fn variables(&self) -> Vec<Variable> {
        match self {
            Value::Variable(var) => vec![var.clone()],
            Value::List {
                value: values,
                loc: _loc,
            } => values.iter().flat_map(|v| v.variables()).collect(),
            Value::Object {
                value: obj,
                loc: _loc,
            } => obj.iter().flat_map(|o| o.1.variables()).collect(),
            _ => Vec::new(),
        }
    }

    pub fn kind(&self) -> &str {
        match self {
            Value::Variable { .. } => "Variable",
            Value::Int { .. } => "Int",
            Value::Float { .. } => "Float",
            Value::String { .. } => "String",
            Value::Boolean { .. } => "Boolean",
            Value::Null { .. } => "Null",
            Value::Enum { .. } => "Enum",
            Value::List { .. } => "List",
            Value::Object { .. } => "Object",
        }
    }

    /// Returns `true` if the value is [`Variable`].
    ///
    /// [`Variable`]: Value::Variable
    #[must_use]
    pub fn is_variable(&self) -> bool {
        matches!(self, Self::Variable { .. })
    }

    /// Returns `true` if the value is [`Null`].
    ///
    /// [`Null`]: Value::Null
    #[must_use]
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null { .. })
    }

    /// Returns an `i32` if the value is a number and can be represented as an i32.
    #[must_use]
    pub fn as_i32(&self) -> Option<i32> {
        i32::try_from(self).ok()
    }

    /// Returns an `f64` if the value is a number and can be represented as an f64.
    #[must_use]
    pub fn as_f64(&self) -> Option<f64> {
        f64::try_from(self).ok()
    }

    /// Returns a `str` if the value is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&'_ str> {
        match self {
            Value::String { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns true/false if the value is a boolean.
    #[must_use]
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Boolean { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// Returns the inner list if the value is a List type.
    #[must_use]
    pub fn as_list(&self) -> Option<&Vec<Value>> {
        match self {
            Value::List { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns a keys/values list if the value is an input object.
    #[must_use]
    pub fn as_object(&self) -> Option<&Vec<(Name, Value)>> {
        match self {
            Value::Object { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Returns the [`hir::Variable`] if the value is a variable reference.
    ///
    /// [`hir::Variable`]: Variable
    #[must_use]
    pub fn as_variable(&self) -> Option<&Variable> {
        match self {
            Value::Variable(var) => Some(var),
            _ => None,
        }
    }
}

/// Coerce to a `Float` input type (from either `Float` or `Int` syntax)
///
/// <https://spec.graphql.org/draft/#sec-Float.Input-Coercion>
impl TryFrom<Value> for f64 {
    type Error = FloatCoercionError;

    #[inline]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        f64::try_from(&value)
    }
}

/// Coerce to a `Float` input type (from either `Float` or `Int` syntax)
///
/// <https://spec.graphql.org/draft/#sec-Float.Input-Coercion>
impl TryFrom<&'_ Value> for f64 {
    type Error = FloatCoercionError;

    fn try_from(value: &'_ Value) -> Result<Self, Self::Error> {
        if let Value::Int { value: float, .. } | Value::Float { value: float, .. } = value {
            // FIXME: what does "a value outside the available precision" mean?
            // Should coercion fail when f64 does not have enough mantissa bits
            // to represent the source token exactly?
            Ok(float.inner.0)
        } else {
            Err(FloatCoercionError(()))
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("coercing a non-numeric value to a `Float` input value")]
pub struct FloatCoercionError(());

/// Coerce to an `Int` input type
///
/// <https://spec.graphql.org/draft/#sec-Int.Input-Coercion>
impl TryFrom<Value> for i32 {
    type Error = IntCoercionError;

    #[inline]
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        i32::try_from(&value)
    }
}

/// Coerce to an `Int` input type
///
/// <https://spec.graphql.org/draft/#sec-Int.Input-Coercion>
impl TryFrom<&'_ Value> for i32 {
    type Error = IntCoercionError;

    fn try_from(value: &'_ Value) -> Result<Self, Self::Error> {
        if let Value::Int { value: float, .. } = value {
            // The parser emitted an `ast::IntValue` instead of `ast::FloatValue`
            // so we already know `float` does not have a frational part.
            float
                .to_i32_checked()
                .ok_or(IntCoercionError::RangeOverflow)
        } else {
            Err(IntCoercionError::NotAnInteger)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum IntCoercionError {
    #[error("coercing a non-integer value to an `Int` input value")]
    NotAnInteger,
    #[error("integer input value overflows the signed 32-bit range")]
    RangeOverflow,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Variable {
    pub(crate) name: String,
    pub(crate) loc: HirNodeLocation,
}

impl Variable {
    /// Get a reference to the argument's name.
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn huge_floats() {
        let mut compiler = ApolloCompiler::new();
        compiler.add_type_system(
            "input HugeFloats {
                a: Float = 9876543210
                b: Float = 9876543210.0
                c: Float = 98765432109876543210
                d: Float = 98765432109876543210.0
            }",
            "huge_floats.graphql",
        );

        let default_values: Vec<_> = compiler
            .db
            .find_input_object_by_name("HugeFloats".into())
            .unwrap()
            .input_fields_definition
            .iter()
            .map(|field| {
                f64::try_from(field.default_value().unwrap())
                    .unwrap()
                    .to_string()
            })
            .collect();
        // The exact value is preserved, even outside of the range of i32
        assert_eq!(default_values[0], "9876543210");
        assert_eq!(default_values[1], "9876543210");
        // Beyond ~53 bits of mantissa we may lose precision,
        // but this is approximation is still in the range of finite f64 values.
        assert_eq!(default_values[2], "98765432109876540000");
        assert_eq!(default_values[3], "98765432109876540000");
    }

    #[test]
    fn values() {
        let mut compiler = ApolloCompiler::new();
        let input = r#"
            query ($arg: Int!) {
                field(
                    float: 1.234,
                    int: 1234,
                    string: "some text",
                    bool: true,
                    variable: $arg,
                )
            }
        "#;
        let id = compiler.add_executable(input, "test.graphql");
        let op = compiler.db.find_operation(id, None).unwrap();
        let field = &op.fields(&compiler.db)[0];

        let args = field.arguments();
        assert_eq!(args[0].value.as_f64(), Some(1.234));
        assert_eq!(args[0].value.as_i32(), None);
        assert_eq!(args[0].value.as_str(), None);
        assert_eq!(args[1].value.as_i32(), Some(1234));
        assert_eq!(args[1].value.as_f64(), Some(1234.0));
        assert_eq!(args[1].value.as_str(), None);
        assert_eq!(args[2].value.as_str(), Some("some text"));
        assert_eq!(args[2].value.as_bool(), None);
        assert_eq!(args[2].value.as_i32(), None);
        assert_eq!(args[3].value.as_bool(), Some(true));
        assert_eq!(args[3].value.as_f64(), None);
        assert_eq!(args[3].value.as_i32(), None);
        assert!(args[4].value.as_variable().is_some());
        assert!(args[4].value.as_bool().is_none());
        assert!(args[4].value.as_f64().is_none());
        assert!(args[4].value.as_i32().is_none());
    }
}
