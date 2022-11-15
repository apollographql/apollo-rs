use std::fmt;

/// A LimitTracker enforces a particular limit within the parser. It keeps
/// track of utilization so that we can report how close to a limit we
/// approached over the lifetime of the tracker.
/// ```rust
/// use apollo_parser::Parser;
///
/// let query = "
/// {
///     animal
///     ...snackSelection
///     ... on Pet {
///       playmates {
///         count
///       }
///     }
/// }
/// ";
/// // Create a new instance of a parser given a query and a
/// // recursion limit
/// let parser = Parser::new(query).recursion_limit(4);
/// // Parse the query, and return a SyntaxTree.
/// let ast = parser.parse();
/// // Retrieve the limits
/// let usage = ast.recursion_limit();
/// // Print out some of the usage details to see what happened during
/// // our parse. `limit` just reports the limit we set, `high` is the
/// // high-water mark of recursion usage.
/// println!("{:?}", usage);
/// println!("{:?}", usage.limit);
/// println!("{:?}", usage.high);
/// // Check that are no errors. These are not part of the AST.
/// assert_eq!(0, ast.errors().len());
///
/// // Get the document root node
/// let doc = ast.document();
/// // ... continue
/// ```
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct LimitTracker {
    current: usize,
    /// High Water mark for this limit
    pub high: usize,
    /// Limit.
    pub limit: usize,
}

impl Default for LimitTracker {
    fn default() -> Self {
        Self {
            current: 0,
            high: 0,
            limit: 4_096, // Recursion limit derived from router experimentation
        }
    }
}

impl LimitTracker {
    pub fn new(limit: usize) -> Self {
        Self {
            current: 0,
            high: 0,
            limit,
        }
    }

    pub fn limited(&self) -> bool {
        self.current > self.limit
    }

    pub fn consume(&mut self) {
        self.current += 1;
        if self.current > self.high {
            self.high = self.current;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }
}

impl fmt::Debug for LimitTracker {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "recursion limit: {}, high: {}", self.limit, self.high)
    }
}
