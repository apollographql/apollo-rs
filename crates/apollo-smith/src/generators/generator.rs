use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::executable::Field;
use apollo_compiler::Name;
use apollo_compiler::Node;
use indexmap::IndexMap;
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
}

/// Registry of [`Generator`]s keyed by GraphQL type name.
pub struct Generators<R: RandomProvider> {
    map: HashMap<Name, Box<dyn Generator<R>>>,
}

impl<R: RandomProvider> Generators<R> {
    pub(crate) fn new(map: HashMap<Name, Box<dyn Generator<R>>>) -> Self {
        Self { map }
    }

    pub(crate) fn insert(&mut self, name: Name, generator: Box<dyn Generator<R>>) {
        self.map.insert(name, generator);
    }

    /// Dispatch to the generator registered for `type_name`, if any.
    ///
    /// Returns `None` if no generator is registered. The caller decides what to do
    /// in that case (the [`ResponseBuilder`][crate::ResponseBuilder] falls back to
    /// default field-by-field object generation for composite types, and to the
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
}
