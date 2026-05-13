use crate::generators::scalar::ScalarGenerators;
use crate::random::RandomProvider;
use crate::random::ResponseError;
use apollo_compiler::executable::Field;
use apollo_compiler::Node;
use indexmap::IndexMap;
use serde_json_bytes::Value;

/// Custom generator for an object (or interface/union) type.
///
/// Registered via
/// [`ResponseBuilder::with_object_generator`][crate::ResponseBuilder::with_object_generator].
pub trait ObjectGenerator<R: RandomProvider> {
    /// Generate a value for an object type given the requested selection.
    ///
    /// The `fields` argument is pre-processed to flatten them across fragment spreads
    /// and inline fragments and group them by response key (alias if present, else field name).
    /// As such, the first entry in the vector can be treated as representative
    /// for meta information about the type of the field.
    ///
    /// The `scalars` argument exposes the builder's registered scalar generators so the
    /// implementation can produce values for individual leaf fields without duplicating
    /// scalar generation logic.
    fn generate(
        &mut self,
        rng: &mut R,
        scalars: &mut ScalarGenerators<R>,
        fields: &IndexMap<String, Vec<Node<Field>>>,
    ) -> Result<Value, ResponseError>;
}
