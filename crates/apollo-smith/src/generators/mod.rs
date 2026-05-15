//! Generator trait and registry used by [`ResponseBuilder`][crate::ResponseBuilder] to
//! produce values for GraphQL types.

pub mod default;
pub mod generator;

pub use default::default_generators;
pub use default::DefaultScalarGenerator;
pub use generator::Generator;
pub use generator::Generators;
