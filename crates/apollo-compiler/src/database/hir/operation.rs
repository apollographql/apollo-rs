use std::{fmt, sync::Arc};

use crate::{
    hir::{
        Directive, DirectiveLocation, Field, FragmentDefinition, HirNodeLocation, Name,
        ObjectTypeDefinition, SelectionSet, VariableDefinition,
    },
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct OperationDefinition {
    pub(crate) operation_ty: OperationType,
    pub(crate) name: Option<Name>,
    pub(crate) variables: Arc<Vec<VariableDefinition>>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

impl OperationDefinition {
    /// Get the kind of the operation: `query`, `mutation`, or `subscription`
    pub fn operation_ty(&self) -> OperationType {
        self.operation_ty
    }

    /// Get a mutable reference to the operation definition's name.
    pub fn name(&self) -> Option<&str> {
        self.name.as_ref().map(|n| n.src())
    }

    /// Get a reference to Name's source.
    pub fn name_src(&self) -> Option<&Name> {
        self.name.as_ref()
    }

    /// Get operation's definition object type.
    pub fn object_type(&self, db: &dyn HirDatabase) -> Option<Arc<ObjectTypeDefinition>> {
        let schema = db.schema();
        let name = match self.operation_ty {
            OperationType::Query => schema.query()?,
            OperationType::Mutation => schema.mutation()?,
            OperationType::Subscription => schema.subscription()?,
        };
        db.object_types_with_built_ins().get(name).cloned()
    }

    /// Get a reference to the operation definition's variables.
    pub fn variables(&self) -> &[VariableDefinition] {
        self.variables.as_ref()
    }

    /// Get a mutable reference to the operation definition's directives.
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

    /// Get a reference to the operation definition's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Get fields in the operation definition (excluding inline fragments and
    /// fragment spreads).
    pub fn fields(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_fields(self.selection_set.clone())
    }

    // NOTE @lrlna: this is quite messy. it should live under the
    // inline_fragment/fragment_spread impls, i.e. op.fragment_spread().fields(),
    // op.inline_fragments().fields()
    //
    // We will need to figure out how to store operation definition id on its
    // fragment spreads and inline fragments to do this

    /// Get all fields in an inline fragment.
    pub fn fields_in_inline_fragments(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_inline_fragment_fields(self.selection_set.clone())
    }

    /// Get all fields in a fragment spread
    pub fn fields_in_fragment_spread(&self, db: &dyn HirDatabase) -> Arc<Vec<Field>> {
        db.operation_fragment_spread_fields(self.selection_set.clone())
    }

    /// Get all fragment definitions referenced by the operation.
    pub fn fragment_references(&self, db: &dyn HirDatabase) -> Arc<Vec<Arc<FragmentDefinition>>> {
        db.operation_fragment_references(self.selection_set.clone())
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if this is a query operation and its [`SelectionSet`] is an introspection.
    pub fn is_introspection(&self, db: &dyn HirDatabase) -> bool {
        self.operation_ty().is_query() && self.selection_set().is_introspection(db)
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

impl OperationType {
    /// Returns `true` if the operation type is [`Query`].
    ///
    /// [`Query`]: OperationType::Query
    #[must_use]
    pub fn is_query(&self) -> bool {
        matches!(self, Self::Query)
    }

    /// Returns `true` if the operation type is [`Mutation`].
    ///
    /// [`Mutation`]: OperationType::Mutation
    #[must_use]
    pub fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation)
    }

    /// Returns `true` if the operation type is [`Subscription`].
    ///
    /// [`Subscription`]: OperationType::Subscription
    #[must_use]
    pub fn is_subscription(&self) -> bool {
        matches!(self, Self::Subscription)
    }
}

impl From<OperationType> for &'static str {
    fn from(ty: OperationType) -> &'static str {
        match ty {
            OperationType::Query => "Query",
            OperationType::Mutation => "Mutation",
            OperationType::Subscription => "Subscription",
        }
    }
}

impl fmt::Display for OperationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str((*self).into())
    }
}

impl<'a> From<&'a str> for OperationType {
    fn from(op_type: &str) -> Self {
        if op_type == "Query" {
            OperationType::Query
        } else if op_type == "Mutation" {
            OperationType::Mutation
        } else {
            OperationType::Subscription
        }
    }
}

impl From<OperationType> for DirectiveLocation {
    fn from(op_type: OperationType) -> Self {
        if op_type.is_subscription() {
            DirectiveLocation::Subscription
        } else if op_type.is_mutation() {
            DirectiveLocation::Mutation
        } else {
            DirectiveLocation::Query
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::hir::OperationDefinition;
    use crate::ApolloCompiler;
    use crate::HirDatabase;
    use std::sync::Arc;

    #[test]
    fn is_introspection_operation() {
        let query_input = r#"
            query TypeIntrospect {
              __type(name: "User") {
                name
                fields {
                  name
                  type {
                    name
                  }
                }
              }
              __schema {
                types {
                  fields {
                    name
                  }
                }
              }
            }
        "#;

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");

        let db = compiler.db;
        let type_introspect: Arc<OperationDefinition> = db
            .find_operation(query_id, Some(String::from("TypeIntrospect")))
            .expect("TypeIntrospect operation does not exist");

        assert!(type_introspect.is_introspection(&db));
    }

    #[test]
    fn is_not_introspection_operation() {
        let mutation_input = r#"
            mutation PurchaseBasket {
              buyA5Wagyu(pounds: 15) {
                submitted
              }
            }
        "#;

        let query_input = r#"
            query CheckStock {
              isKagoshimaWagyuInstock

              __schema {
                types {
                  fields {
                    name
                  }
                }
              }
            }
        "#;

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");
        let mutation_id = compiler.add_executable(mutation_input, "mutation.graphql");

        let db = compiler.db;
        let check_stock: Arc<OperationDefinition> = db
            .find_operation(query_id, Some("CheckStock".into()))
            .expect("CheckStock operation does not exist");

        let purchase_operation: Arc<OperationDefinition> = db
            .find_operation(mutation_id, Some("PurchaseBasket".into()))
            .expect("CheckStock operation does not exist");

        assert!(!check_stock.is_introspection(&db));
        assert!(!purchase_operation.is_introspection(&db));
    }

    #[test]
    fn is_introspection_deep() {
        let query_input = r#"
          query IntrospectDeepFragments {
            ...onRootTrippy
          }

          fragment onRootTrippy on Root {
             ...onRooten
          }

          fragment onRooten on Root {
            ...onRooten2

            ... on Root {
              __schema {
                types {
                  name
                }
              }
            }
          }

          fragment onRooten2 on Root {
             __type(name: "Root") {
              ...onType
            }
            ... on Root {
              __schema {
                directives {
                  name
                }
              }
            }
          }
          fragment onType on __Type {
            fields {
              name
            }
          }

          fragment onRooten2_not_intro on Root {
            species(id: "Ewok") {
              name
            }

            ... on Root {
              __schema {
                directives {
                  name
                }
              }
            }
         }
        "#;

        let query_input_not_introspect =
            query_input.replace("...onRooten2", "...onRooten2_not_intro");

        let mut compiler = ApolloCompiler::new();
        let query_id = compiler.add_executable(query_input, "query.graphql");
        let query_id_not_introspect =
            compiler.add_executable(query_input_not_introspect.as_str(), "query2.graphql");

        let db = compiler.db;
        let deep_introspect: Arc<OperationDefinition> = db
            .find_operation(query_id, Some("IntrospectDeepFragments".into()))
            .expect("IntrospectDeepFragments operation does not exist");

        assert!(deep_introspect.is_introspection(&db));

        let deep_introspect: Arc<OperationDefinition> = db
            .find_operation(
                query_id_not_introspect,
                Some("IntrospectDeepFragments".into()),
            )
            .expect("IntrospectDeepFragments operation does not exist");
        assert!(!deep_introspect.is_introspection(&db));
    }

    #[test]
    fn introspection_field_types() {
        let input = r#"
type Query {
  id: String
  name: String
  birthday: Date
}

scalar Date @specifiedBy(url: "datespec.com")

{
  __type(name: "User") {
    name
    fields {
      name
      type {
        name
      }
    }
  }
}
        "#;
        let mut compiler = ApolloCompiler::new();
        let file_id = compiler.add_type_system(input, "ts.graphql");

        let diagnostics = compiler.validate();
        assert!(diagnostics.is_empty());

        let db = compiler.db;
        let op = db.find_operation(file_id, None).unwrap();
        let ty_field = op
            .selection_set()
            .field("__type")
            .unwrap()
            .ty(&db)
            .unwrap()
            .name();

        assert_eq!(ty_field, "__Type");
    }
}
