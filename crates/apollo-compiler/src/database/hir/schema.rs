use std::sync::Arc;

use crate::{
    hir::{Directive, HirNodeLocation, ObjectTypeDefinition, OperationType, Type},
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Default, Eq)]
pub struct SchemaDefinition {
    pub(crate) description: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
    pub(crate) loc: Option<HirNodeLocation>,
    pub(crate) extensions: Vec<Arc<SchemaExtension>>,
    pub(crate) root_operation_names: RootOperationNames,
}

#[derive(Default, Clone, Debug, Hash, PartialEq, Eq)]
pub(crate) struct RootOperationNames {
    pub(crate) query: Option<String>,
    pub(crate) mutation: Option<String>,
    pub(crate) subscription: Option<String>,
}

impl SchemaDefinition {
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to the schema definition's directives (excluding those on extensions).
    pub fn self_directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns an iterator of directives on either the `schema` definition or its extensions
    pub fn directives(&self) -> impl Iterator<Item = &Directive> + '_ {
        self.self_directives()
            .iter()
            .chain(self.extensions.iter().flat_map(|ext| ext.directives()))
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    ///
    /// Includes directives on either the `schema` definition or its extensions,
    /// like [`directives`][Self::directives].
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .filter(move |directive| directive.name() == name)
    }

    /// Returns the root operations from this schema definition,
    /// excluding those from schema extensions.
    pub fn self_root_operations(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Returns an iterator of root operations, from either on this schema defintion or its extensions.
    pub fn root_operations(&self) -> impl Iterator<Item = &RootOperationTypeDefinition> {
        self.self_root_operations().iter().chain(
            self.extensions()
                .iter()
                .flat_map(|ext| ext.root_operations()),
        )
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Extensions that apply to this definition
    pub fn extensions(&self) -> &[Arc<SchemaExtension>] {
        &self.extensions
    }

    /// Returns the name of the object type for the `query` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn query(&self) -> Option<&str> {
        self.root_operation_names.query.as_deref()
    }

    /// Returns the name of the object type for the `mutation` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn mutation(&self) -> Option<&str> {
        self.root_operation_names.mutation.as_deref()
    }

    /// Returns the name of the object type for the `subscription` root operation,
    /// defined either on this schema defintion or its extensions.
    ///
    /// The corresponding object type definition can be found
    /// at [`compiler.db.object_types().get(name)`][HirDatabase::object_types].
    pub fn subscription(&self) -> Option<&str> {
        self.root_operation_names.subscription.as_deref()
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct RootOperationTypeDefinition {
    pub(crate) operation_ty: OperationType,
    pub(crate) named_type: Type,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl RootOperationTypeDefinition {
    /// Get a reference to the root operation type definition's named type.
    pub fn named_type(&self) -> &Type {
        &self.named_type
    }

    /// Get the kind of the root operation type definition: `query`, `mutation`, or `subscription`
    pub fn operation_ty(&self) -> OperationType {
        self.operation_ty
    }

    /// Get the object type this root operation is referencing.
    pub fn object_type(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        db.find_object_type_by_name(self.named_type().name())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }
}

impl Default for RootOperationTypeDefinition {
    fn default() -> Self {
        Self {
            operation_ty: OperationType::Query,
            named_type: Type::Named {
                name: "Query".to_string(),
                loc: None,
            },
            loc: None,
        }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct SchemaExtension {
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) root_operation_type_definition: Arc<Vec<RootOperationTypeDefinition>>,
    pub(crate) loc: HirNodeLocation,
}

impl SchemaExtension {
    /// Get a reference to the schema definition's directives.
    pub fn directives(&self) -> &[Directive] {
        self.directives.as_ref()
    }

    /// Returns the first directive with the given name.
    ///
    /// For repeatable directives, see [`directives_by_name`][Self::directives_by_name] (plural).
    pub fn directive_by_name(&self, name: &str) -> Option<&Directive> {
        self.directives_by_name(name).next()
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// For non-repeatable directives, [`directive_by_name`][Self::directive_by_name] (singular).
    pub fn directives_by_name<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Directive> + 'name {
        self.directives()
            .iter()
            .filter(move |directive| directive.name() == name)
    }

    /// Get a reference to the schema definition's root operation type definition.
    pub fn root_operations(&self) -> &[RootOperationTypeDefinition] {
        self.root_operation_type_definition.as_ref()
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn root_operations() {
        let mut compiler = ApolloCompiler::new();
        let first = r#"
            schema @core(feature: "https://specs.apollo.dev/core/v0.1")
            type Query {
                field: Int
            }
            type Subscription {
                newsletter: [String]
            }
        "#;
        let second = r#"
            extend schema @core(feature: "https://specs.apollo.dev/join/v0.1") {
                query: MyQuery
            }
            type MyQuery {
                different_field: String
            }
        "#;
        compiler.add_type_system(first, "first.graphql");
        compiler.add_type_system(second, "second.graphql");

        let schema = compiler.db.schema();
        assert_eq!(
            schema
                .self_root_operations()
                .iter()
                .map(|op| op.named_type().name())
                .collect::<Vec<_>>(),
            ["Subscription"]
        );
        assert_eq!(
            schema
                .root_operations()
                .map(|op| op.named_type().name())
                .collect::<Vec<_>>(),
            ["Subscription", "MyQuery"]
        );
        assert!(schema.mutation().is_none());
        assert_eq!(schema.query().unwrap(), "MyQuery");
        assert_eq!(schema.subscription().unwrap(), "Subscription");
    }
}
