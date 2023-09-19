macro_rules! serialize_method {
    () => {
        /// Returns a builder that has chaining methods for setting serialization configuration,
        /// and implements the [`Display`][std::fmt::Display] and [`ToString`] traits
        /// by writing GraphQL syntax.
        pub fn serialize(&self) -> $crate::ast::serialize::Serialize<Self> {
            $crate::ast::serialize::Serialize {
                node: self,
                config: Default::default(),
            }
        }
    };
}
