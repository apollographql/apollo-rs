use crate::generators::Generator;
use crate::generators::Generators;
use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::executable::Field;
use apollo_compiler::Name;
use apollo_compiler::Node;
use indexmap::IndexMap;
use serde_json_bytes::serde_json::Number;
use serde_json_bytes::Value;
use std::collections::HashMap;

/// Configurable defaults for generating scalar values.
///
/// Each variant describes how to generate a value of a particular primitive type
/// using a [`RandomProvider`]. Register custom generators via
/// [`ResponseBuilder::with_generator`][crate::ResponseBuilder::with_generator].
#[derive(Debug, Clone)]
pub enum DefaultScalarGenerator {
    /// Generate a random boolean.
    Bool,
    /// Generate a random integer ID in the given inclusive range, serialized as a string.
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

    pub fn boxed<R: RandomProvider>(self) -> Box<dyn Generator<R>> {
        Box::new(self)
    }
}

impl<R: RandomProvider> Generator<R> for DefaultScalarGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
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

impl<R: RandomProvider> Generators<R> {
    /// Generate a leaf value for a named scalar type using the registered generator,
    /// falling back to [`DefaultScalarGenerator::DEFAULT`] if none is registered.
    ///
    /// Object generators typically call this to fill scalar fields without
    /// hand-rolling generation logic for each leaf.
    pub fn generate_scalar(
        &mut self,
        type_name: &Name,
        rng: &mut R,
    ) -> Result<Value, ResponseError> {
        let empty = IndexMap::new();
        if let Some(result) = self.try_generate(type_name, rng, &empty) {
            return result;
        }
        let mut fallback = DefaultScalarGenerator::DEFAULT;
        fallback.generate(rng, self, &empty)
    }
}

/// Returns a [`Generators`] registry pre-populated with the built-in GraphQL scalar generators.
pub fn default_generators<R: RandomProvider>() -> Generators<R> {
    let map: HashMap<Name, Box<dyn Generator<R>>> = [
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
    .collect();
    Generators::new(map)
}
