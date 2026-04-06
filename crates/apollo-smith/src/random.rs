use apollo_compiler::Name;
use arbitrary::Unstructured;
use rand::RngExt;
use serde_json_bytes::serde_json::Number;
use serde_json_bytes::Value;
use std::collections::HashMap;

/// Error type for response generation.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    /// The randomness source was exhausted or produced invalid data.
    #[error("randomness source exhausted or produced invalid data")]
    Exhausted,
    /// The randomness source produced data that could not be converted to the expected format.
    #[error("invalid format: {0}")]
    InvalidFormat(String),
}

/// Abstraction over a source of randomness for response generation.
///
/// Implementations are provided for [`Unstructured`] (for fuzz testing) and
/// for any type implementing [`rand::Rng`] via the [`RandProvider`] newtype.
pub trait RandomProvider {
    /// Generate a random boolean.
    fn gen_bool(&mut self) -> Result<bool, ResponseError>;

    /// Generate a random `i32` within the inclusive range `[min, max]`.
    fn gen_i32_range(&mut self, min: i32, max: i32) -> Result<i32, ResponseError>;

    /// Generate a random `usize` within the inclusive range `[min, max]`.
    fn gen_usize_range(&mut self, min: usize, max: usize) -> Result<usize, ResponseError>;

    /// Generate a random `f64` within the inclusive range `[min, max]`.
    fn gen_f64_range(&mut self, min: f64, max: f64) -> Result<f64, ResponseError>;

    /// Generate a random alphanumeric character (`[0-9a-zA-Z]`).
    fn gen_alphanumeric_char(&mut self) -> Result<char, ResponseError>;

    /// Choose a random index in `0..len`. Returns an error if `len == 0`.
    fn choose_index(&mut self, len: usize) -> Result<usize, ResponseError>;

    /// Return `true` with probability `numerator / denominator`.
    fn ratio(&mut self, numerator: u32, denominator: u32) -> Result<bool, ResponseError>;
}

impl RandomProvider for Unstructured<'_> {
    fn gen_bool(&mut self) -> Result<bool, ResponseError> {
        self.arbitrary::<bool>()
            .map_err(|_| ResponseError::Exhausted)
    }

    fn gen_i32_range(&mut self, min: i32, max: i32) -> Result<i32, ResponseError> {
        self.int_in_range(min..=max)
            .map_err(|_| ResponseError::Exhausted)
    }

    fn gen_usize_range(&mut self, min: usize, max: usize) -> Result<usize, ResponseError> {
        self.int_in_range(min..=max)
            .map_err(|_| ResponseError::Exhausted)
    }

    fn gen_f64_range(&mut self, min: f64, max: f64) -> Result<f64, ResponseError> {
        // Unstructured doesn't support float ranges, so we generate a raw f64
        // and map it into [min, max].
        let raw: u32 = self.arbitrary().map_err(|_| ResponseError::Exhausted)?;
        let fraction = (raw as f64) / (u32::MAX as f64); // [0.0, 1.0]
        Ok(min + fraction * (max - min))
    }

    fn gen_alphanumeric_char(&mut self) -> Result<char, ResponseError> {
        let idx: u8 = self
            .int_in_range(0..=61)
            .map_err(|_| ResponseError::Exhausted)?;
        Ok(match idx {
            0..=9 => (b'0' + idx) as char,
            10..=35 => (b'a' + idx - 10) as char,
            _ => (b'A' + idx - 36) as char,
        })
    }

    fn choose_index(&mut self, len: usize) -> Result<usize, ResponseError> {
        if len == 0 {
            return Err(ResponseError::InvalidFormat(
                "cannot choose from empty collection".into(),
            ));
        }
        self.int_in_range(0..=(len - 1))
            .map_err(|_| ResponseError::Exhausted)
    }

    fn ratio(&mut self, numerator: u32, denominator: u32) -> Result<bool, ResponseError> {
        // Unstructured::ratio takes u8, so we scale down if needed.
        // For ratios that fit in u8, use directly; otherwise approximate.
        let (n, d) = if numerator <= u8::MAX as u32 && denominator <= u8::MAX as u32 {
            (numerator as u8, denominator as u8)
        } else {
            // Scale to fit in u8
            let scale = denominator.max(1) as f64 / 255.0;
            let n = (numerator as f64 / scale).round() as u8;
            let d = ((denominator as f64 / scale).round() as u8).max(1);
            (n, d)
        };
        self.ratio(n, d).map_err(|_| ResponseError::Exhausted)
    }
}

/// Newtype wrapper that implements [`RandomProvider`] for any [`rand::Rng`].
///
/// # Example
///
/// ```ignore
/// use apollo_smith::RandProvider;
///
/// let mut rng = RandProvider(rand::rng());
/// let response = ResponseBuilder::new(&mut rng, &doc, &schema).build()?;
/// ```
pub struct RandProvider<R>(pub R);

impl<R: rand::Rng> RandomProvider for RandProvider<R> {
    fn gen_bool(&mut self) -> Result<bool, ResponseError> {
        Ok(self.0.random_bool(0.5))
    }

    fn gen_i32_range(&mut self, min: i32, max: i32) -> Result<i32, ResponseError> {
        Ok(self.0.random_range(min..=max))
    }

    fn gen_usize_range(&mut self, min: usize, max: usize) -> Result<usize, ResponseError> {
        Ok(self.0.random_range(min..=max))
    }

    fn gen_f64_range(&mut self, min: f64, max: f64) -> Result<f64, ResponseError> {
        Ok(self.0.random_range(min..=max))
    }

    fn gen_alphanumeric_char(&mut self) -> Result<char, ResponseError> {
        Ok(self.0.sample(rand::distr::Alphanumeric) as char)
    }

    fn choose_index(&mut self, len: usize) -> Result<usize, ResponseError> {
        if len == 0 {
            return Err(ResponseError::InvalidFormat(
                "cannot choose from empty collection".into(),
            ));
        }
        Ok(self.0.random_range(0..len))
    }

    fn ratio(&mut self, numerator: u32, denominator: u32) -> Result<bool, ResponseError> {
        Ok(self.0.random_ratio(numerator, denominator))
    }
}

/// Configuration for generating scalar values.
///
/// Each variant describes how to generate a value of a particular type using
/// a [`RandomProvider`]. Register custom scalar configs via
/// [`ResponseBuilder::with_scalar_config`][crate::ResponseBuilder::with_scalar_config].
#[derive(Debug, Clone)]
pub enum ScalarConfig {
    /// Generate a random boolean.
    Bool,
    /// Generate a random integer in the given inclusive range.
    Int { min: i32, max: i32 },
    /// Generate a random float in the given inclusive range.
    Float { min: f64, max: f64 },
    /// Generate a random alphanumeric string with length in the given inclusive range.
    String { min_len: usize, max_len: usize },
}

impl ScalarConfig {
    /// The default configuration used for unknown or custom scalars: an
    /// alphanumeric string of length 1–10.
    pub const DEFAULT: Self = Self::String {
        min_len: 1,
        max_len: 10,
    };

    /// Generate a random value according to this configuration.
    pub fn generate<R: RandomProvider>(&self, rng: &mut R) -> Result<Value, ResponseError> {
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
        }
    }
}

/// Returns the default scalar configurations for the built-in GraphQL scalar types.
pub fn default_scalar_configs() -> HashMap<Name, ScalarConfig> {
    [
        (Name::new_unchecked("Boolean"), ScalarConfig::Bool),
        (
            Name::new_unchecked("Int"),
            ScalarConfig::Int { min: 0, max: 100 },
        ),
        (
            Name::new_unchecked("ID"),
            ScalarConfig::Int { min: 0, max: 100 },
        ),
        (
            Name::new_unchecked("Float"),
            ScalarConfig::Float {
                min: -1.0,
                max: 1.0,
            },
        ),
        (
            Name::new_unchecked("String"),
            ScalarConfig::String {
                min_len: 1,
                max_len: 10,
            },
        ),
    ]
    .into_iter()
    .collect()
}
