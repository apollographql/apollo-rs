//! Generator trait and registry used by [`ResponseBuilder`][crate::ResponseBuilder]
//! to produce values for GraphQL types, plus the built-in generators for the
//! standard GraphQL scalars.

use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::executable::Field;
use apollo_compiler::Name;
use apollo_compiler::Node;
use indexmap::IndexMap;
use serde_json_bytes::serde_json::Number;
use serde_json_bytes::Value;
use std::collections::HashMap;

/// A pluggable generator for the value of a named GraphQL type.
///
/// Generators are registered on a
/// [`ResponseBuilder`][crate::ResponseBuilder] under a type name via
/// [`ResponseBuilder::with_generator`][crate::ResponseBuilder::with_generator] and
/// invoked when the builder is about to produce a value of that type.
///
/// The `fields` argument is the requested selection grouped by response key (alias
/// if present, else field name); fragment spreads and inline fragments are
/// pre-flattened against the concrete type. Leaf-type generators (scalars, enums)
/// may ignore `fields`. The `generators` argument exposes the full registry so an
/// implementation can delegate to other registered generators — most often to fill
/// a leaf field via [`Generators::generate_scalar`].
pub trait Generator<R: RandomProvider> {
    fn generate(
        &mut self,
        rng: &mut R,
        generators: &mut Generators<R>,
        fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError>;

    /// Move this generator into a `Box<dyn Generator<R>>` for registration with
    /// [`ResponseBuilder::with_generator`][crate::ResponseBuilder::with_generator].
    fn boxed(self) -> Box<dyn Generator<R>>
    where
        Self: Sized + 'static,
    {
        Box::new(self)
    }
}

/// Registry of [`Generator`]s keyed by GraphQL type name.
pub struct Generators<R: RandomProvider> {
    map: HashMap<Name, Box<dyn Generator<R>>>,
}

impl<R: RandomProvider> Generators<R> {
    pub(crate) fn insert(&mut self, name: Name, generator: Box<dyn Generator<R>>) {
        self.map.insert(name, generator);
    }

    /// Dispatch to the generator registered for `type_name`, if any.
    ///
    /// Returns `None` if no generator is registered. The caller decides what to do
    /// in that case (the [`ResponseBuilder`][crate::ResponseBuilder] falls back to
    /// default field-by-field object generation for composite types, and to a
    /// default scalar generator for leaf types via [`Self::generate_scalar`]).
    ///
    /// While the dispatched generator runs, its entry is temporarily removed from
    /// the registry. A generator that recursively asks for its own registered type
    /// will see `None` on the inner call; generators for other types remain
    /// reachable as normal.
    pub fn try_generate(
        &mut self,
        type_name: &Name,
        rng: &mut R,
        fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Option<Result<Value, ResponseError>> {
        let mut generator = self.map.remove(type_name)?;
        let result = generator.generate(rng, self, fields);
        self.map.insert(type_name.clone(), generator);
        Some(result)
    }

    /// Generate a leaf value for a named scalar type using the registered
    /// generator, falling back to an alphanumeric string of length 1–10 if none
    /// is registered.
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
        let mut fallback = StringGenerator {
            min_len: 1,
            max_len: 10,
        };
        fallback.generate(rng, self, &empty)
    }
}

/// Generates a random boolean.
#[derive(Debug, Default, Clone)]
pub struct BooleanGenerator;

impl<R: RandomProvider> Generator<R> for BooleanGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
        Ok(Value::Bool(rng.gen_bool()?))
    }
}

/// Generates a random integer in the given inclusive range.
#[derive(Debug, Clone)]
pub struct IntGenerator {
    pub min: i32,
    pub max: i32,
}

impl<R: RandomProvider> Generator<R> for IntGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
        Ok(Value::Number(rng.gen_i32_range(self.min, self.max)?.into()))
    }
}

impl Default for IntGenerator {
    fn default() -> Self {
        Self { min: 0, max: 100 }
    }
}

/// Generates a random float in the given inclusive range.
#[derive(Debug, Clone)]
pub struct FloatGenerator {
    pub min: f64,
    pub max: f64,
}

impl<R: RandomProvider> Generator<R> for FloatGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
        let f = rng.gen_f64_range(self.min, self.max)?;
        let num = Number::from_f64(f)
            .ok_or_else(|| ResponseError::InvalidFormat("generated non-finite float".into()))?;
        Ok(Value::Number(num))
    }
}

impl Default for FloatGenerator {
    fn default() -> Self {
        Self {
            min: -1.0,
            max: 1.0,
        }
    }
}

/// Generates a random alphanumeric string with length in the given inclusive range.
#[derive(Debug, Clone)]
pub struct StringGenerator {
    pub min_len: usize,
    pub max_len: usize,
}

impl<R: RandomProvider> Generator<R> for StringGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
        let len = rng.gen_usize_range(self.min_len, self.max_len)?;
        let s: Result<std::string::String, _> =
            (0..len).map(|_| rng.gen_alphanumeric_char()).collect();
        Ok(Value::String(s?.into()))
    }
}

impl Default for StringGenerator {
    fn default() -> Self {
        Self {
            min_len: 1,
            max_len: 10,
        }
    }
}

/// Generates a random integer ID in the given inclusive range, serialized as a string.
#[derive(Debug, Clone)]
pub struct IdGenerator {
    pub min: i32,
    pub max: i32,
}

impl<R: RandomProvider> Generator<R> for IdGenerator {
    fn generate(
        &mut self,
        rng: &mut R,
        _generators: &mut Generators<R>,
        _fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError> {
        Ok(Value::String(
            rng.gen_i32_range(self.min, self.max)?.to_string().into(),
        ))
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self { min: 0, max: 100 }
    }
}

impl<R: RandomProvider> Default for Generators<R> {
    fn default() -> Self {
        let map: HashMap<Name, Box<dyn Generator<R>>> = [
            (
                Name::new_unchecked("Boolean"),
                BooleanGenerator::default().boxed(),
            ),
            (Name::new_unchecked("Int"), IntGenerator::default().boxed()),
            (Name::new_unchecked("ID"), IdGenerator::default().boxed()),
            (
                Name::new_unchecked("Float"),
                FloatGenerator::default().boxed(),
            ),
            (
                Name::new_unchecked("String"),
                StringGenerator::default().boxed(),
            ),
        ]
        .into_iter()
        .collect();
        Self { map }
    }
}
