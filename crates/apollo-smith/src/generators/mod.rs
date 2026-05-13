//! Generator traits and helpers used by [`ResponseBuilder`][crate::ResponseBuilder] to
//! produce values for scalar and object types.

pub mod object;
pub mod scalar;

pub use object::ObjectGenerator;
pub use scalar::default_scalar_generators;
pub use scalar::DefaultScalarGenerator;
pub use scalar::ScalarGenerator;
pub use scalar::ScalarGenerators;
