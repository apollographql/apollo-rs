use arbitrary::Unstructured;
use rand::RngExt;

const ALPHANUM_CHARS: &[char; 62] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l',
    'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', '0', '1', '2', '3', '4',
    '5', '6', '7', '8', '9',
];

/// Error type for response generation.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    /// The randomness source attempted to choose from an empty range.
    #[error("randomness source attempted to choose from an empty range")]
    EmptyChoose,
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

    /// Return `true` with probability `numerator / denominator`. Panics if `numerator == 0` or `numerator > denominator`.
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
        self.choose(ALPHANUM_CHARS)
            .map_err(|_| ResponseError::Exhausted)
            .copied()
    }

    fn choose_index(&mut self, len: usize) -> Result<usize, ResponseError> {
        self.choose_index(len)
            .map_err(|_| ResponseError::EmptyChoose)
    }

    fn ratio(&mut self, numerator: u32, denominator: u32) -> Result<bool, ResponseError> {
        self.ratio(numerator, denominator)
            .map_err(|_| ResponseError::Exhausted)
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
            return Err(ResponseError::EmptyChoose);
        }
        Ok(self.0.random_range(0..len))
    }

    fn ratio(&mut self, numerator: u32, denominator: u32) -> Result<bool, ResponseError> {
        Ok(self.0.random_ratio(numerator, denominator))
    }
}
