use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::Name;
use serde_json_bytes::serde_json::Number;
use serde_json_bytes::Value;
use std::collections::HashMap;

/// Custom generator for a scalar type.
///
/// Registered via
/// [`ResponseBuilder::with_scalar_generator`][crate::ResponseBuilder::with_scalar_generator].
pub trait ScalarGenerator<R: RandomProvider> {
    /// Generate a random value using the [RandomProvider].
    fn generate(&mut self, rng: &mut R) -> Result<Value, ResponseError>;
}

/// Handle that lets an [`ObjectGenerator`][super::ObjectGenerator] delegate field
/// generation back to the scalar generators configured on its owning
/// [`ResponseBuilder`][crate::ResponseBuilder].
pub struct ScalarGenerators<'a, R: RandomProvider> {
    map: &'a mut HashMap<Name, Box<dyn ScalarGenerator<R>>>,
}

impl<'a, R: RandomProvider> ScalarGenerators<'a, R> {
    pub(crate) fn new(map: &'a mut HashMap<Name, Box<dyn ScalarGenerator<R>>>) -> Self {
        Self { map }
    }

    /// Generate a value for the named scalar type using the existing configured scalar generators
    pub fn generate(&mut self, type_name: &Name, rng: &mut R) -> Result<Value, ResponseError> {
        match self.map.get_mut(type_name) {
            Some(gen) => gen.generate(rng),
            None => {
                let mut default = DefaultScalarGenerator::DEFAULT;
                default.generate(rng)
            }
        }
    }
}

/// Configurable defaults for generating scalar values.
///
/// Each variant describes how to generate a value of a particular primitive type using
/// a [`RandomProvider`]. Register custom scalar generators via
/// [`ResponseBuilder::with_scalar_generator`][crate::ResponseBuilder::with_scalar_generator].
#[derive(Debug, Clone)]
pub enum DefaultScalarGenerator {
    /// Generate a random boolean.
    Bool,
    /// Generate a random integer ID in the given inclusive range, serialized as a string
    ID { min: i32, max: i32 },
    /// Generate a random integer in the given inclusive range.
    Int { min: i32, max: i32 },
    /// Generate a random float in the given inclusive range.
    Float { min: f64, max: f64 },
    /// Generate a random alphanumeric string with length in the given inclusive range.
    String { min_len: usize, max_len: usize },
}

impl DefaultScalarGenerator {
    /// The default configuration used for unknown or custom scalars: an
    /// alphanumeric string of length 1–10.
    pub const DEFAULT: Self = Self::String {
        min_len: 1,
        max_len: 10,
    };

    pub fn boxed<R: RandomProvider>(self) -> Box<dyn ScalarGenerator<R>> {
        Box::new(self)
    }
}

impl<R: RandomProvider> ScalarGenerator<R> for DefaultScalarGenerator {
    fn generate(&mut self, rng: &mut R) -> Result<Value, ResponseError> {
        match *self {
            Self::Bool => Ok(Value::Bool(rng.gen_bool()?)),
            Self::Int { min, max } => Ok(Value::Number(rng.gen_i32_range(min, max)?.into())),
            Self::Float { min, max } => {
                let f = rng.gen_f64_range(min, max)?;
                let num = Number::from_f64(f).ok_or_else(|| {
                    ResponseError::InvalidFormat("generated non-finite float".into())
                })?;
                Ok(Value::Number(num))
            }
            Self::String { min_len, max_len } => {
                let len = rng.gen_usize_range(min_len, max_len)?;
                let s: Result<std::string::String, _> =
                    (0..len).map(|_| rng.gen_alphanumeric_char()).collect();
                Ok(Value::String(s?.into()))
            }
            Self::ID { min, max } => Ok(Value::String(
                rng.gen_i32_range(min, max)?.to_string().into(),
            )),
        }
    }
}

/// Returns the default generators for the built-in GraphQL scalar types.
pub fn default_scalar_generators<R: RandomProvider>() -> HashMap<Name, Box<dyn ScalarGenerator<R>>>
{
    [
        (
            Name::new_unchecked("Boolean"),
            DefaultScalarGenerator::Bool.boxed(),
        ),
        (
            Name::new_unchecked("Int"),
            DefaultScalarGenerator::Int { min: 0, max: 100 }.boxed(),
        ),
        (
            Name::new_unchecked("ID"),
            DefaultScalarGenerator::ID { min: 0, max: 100 }.boxed(),
        ),
        (
            Name::new_unchecked("Float"),
            DefaultScalarGenerator::Float {
                min: -1.0,
                max: 1.0,
            }
            .boxed(),
        ),
        (
            Name::new_unchecked("String"),
            DefaultScalarGenerator::String {
                min_len: 1,
                max_len: 10,
            }
            .boxed(),
        ),
    ]
    .into_iter()
    .collect()
}
