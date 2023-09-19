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

macro_rules! directive_by_name_method {
    () => {
        /// Returns the first directive with the given name, if any.
        ///
        /// This method is best for non-repeatable directives. For repeatable directives,
        /// see [`directives_by_name`][Self::directives_by_name] (plural)
        pub fn directive_by_name(&self, name: &str) -> Option<&Node<Directive>> {
            self.directives_by_name(name).next()
        }
    };
}

macro_rules! directive_methods {
    () => {
        /// Returns an iterator of directives with the given name.
        ///
        /// This method is best for repeatable directives. For non-repeatable directives,
        /// see [`directive_by_name`][Self::directive_by_name] (singular)
        pub fn directives_by_name<'def: 'name, 'name>(
            &'def self,
            name: &'name str,
        ) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
            directives_by_name(&self.directives, name)
        }

        directive_by_name_method!();
    };
}
