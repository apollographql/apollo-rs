use std::sync::Arc;

use crate::{
    hir::{
        Argument, ArgumentsDefinition, Directive, HirNodeLocation, Name, SelectionSet, Type,
        TypeDefinition, Variable,
    },
    HirDatabase,
};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Alias(pub String);
impl Alias {
    pub fn name(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Field {
    pub(crate) alias: Option<Arc<Alias>>,
    pub(crate) name: Name,
    pub(crate) arguments: Arc<Vec<Argument>>,
    pub(crate) parent_obj: Option<String>,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) selection_set: SelectionSet,
    pub(crate) loc: HirNodeLocation,
}

impl Field {
    /// Get a reference to the field's alias.
    pub fn alias(&self) -> Option<&Alias> {
        match &self.alias {
            Some(alias) => Some(alias.as_ref()),
            None => None,
        }
    }

    /// Get the field's name, corresponding to the definition it looks up.
    ///
    /// For example, in this operation, the `.name()` is "sourceField":
    /// ```graphql
    /// query GetField { alias: sourceField }
    /// ```
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get the name that will be used for this field selection in response formatting.
    ///
    /// For example, in this operation, the `.response_name()` is "sourceField":
    /// ```graphql
    /// query GetField { sourceField }
    /// ```
    ///
    /// But in this operation that uses an alias, the `.response_name()` is "responseField":
    /// ```graphql
    /// query GetField { responseField: sourceField }
    /// ```
    pub fn response_name(&self) -> &str {
        self.alias().map(Alias::name).unwrap_or_else(|| self.name())
    }

    /// Get a reference to field's type.
    pub fn ty(&self, db: &dyn HirDatabase) -> Option<Type> {
        let def = db
            .find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())?
            .field(db, self.name())?
            .ty()
            .to_owned();
        Some(def)
    }

    /// Get the field's parent type definition.
    pub fn parent_type(&self, db: &dyn HirDatabase) -> Option<TypeDefinition> {
        db.find_type_definition_by_name(self.parent_obj.as_ref()?.to_string())
    }

    /// Get field's original field definition.
    pub fn field_definition(&self, db: &dyn HirDatabase) -> Option<FieldDefinition> {
        let type_name = self.parent_obj.as_ref()?.to_string();
        let type_def = db.find_type_definition_by_name(type_name)?;

        match type_def {
            TypeDefinition::ObjectTypeDefinition(obj) => obj.field(db, self.name()).cloned(),
            TypeDefinition::InterfaceTypeDefinition(iface) => iface.field(self.name()).cloned(),
            _ => None,
        }
    }

    /// Get a reference to the field's arguments.
    pub fn arguments(&self) -> &[Argument] {
        self.arguments.as_ref()
    }

    /// Get a reference to the field's directives.
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

    /// Get a reference to the field's selection set.
    pub fn selection_set(&self) -> &SelectionSet {
        &self.selection_set
    }

    /// Return an iterator over the variables used in arguments to this field and its directives.
    pub(crate) fn self_used_variables(&self) -> impl Iterator<Item = Variable> + '_ {
        self.arguments
            .iter()
            .chain(
                self.directives()
                    .iter()
                    .flat_map(|directive| directive.arguments()),
            )
            .flat_map(|arg| arg.value().variables())
    }

    /// Get variables used in the field, including in sub-selections.
    ///
    /// For example, with this field:
    /// ```graphql
    /// {
    ///   field(arg: $arg) {
    ///     number(formatAs: $format)
    ///   }
    /// }
    /// ```
    /// the used variables are `$arg` and `$format`.
    pub fn variables(&self, db: &dyn HirDatabase) -> Vec<Variable> {
        let mut vars = self.self_used_variables().collect::<Vec<_>>();
        vars.extend(self.selection_set.variables(db));
        vars
    }

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> HirNodeLocation {
        self.loc
    }

    /// Returns true if this is an introspection field (i.e. it's
    /// [`Self::name()`] is one of __type, or __schema).
    pub fn is_introspection(&self) -> bool {
        let field_name = self.name();
        field_name == "__type" || field_name == "__schema" || field_name == "__typename"
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FieldDefinition {
    pub(crate) description: Option<String>,
    pub(crate) name: Name,
    pub(crate) arguments: ArgumentsDefinition,
    pub(crate) ty: Type,
    pub(crate) directives: Arc<Vec<Directive>>,
    pub(crate) loc: Option<HirNodeLocation>,
}

impl FieldDefinition {
    /// Get a reference to the field definition's name.
    pub fn name(&self) -> &str {
        self.name.src()
    }

    /// Get a reference to the field definition's description.
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Get a reference to the field's directives.
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

    /// Get the AST location information for this HIR node.
    pub fn loc(&self) -> Option<HirNodeLocation> {
        self.loc
    }

    /// Get a reference to field definition's type.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Get a reference to field definition's arguments
    pub fn arguments(&self) -> &ArgumentsDefinition {
        &self.arguments
    }
}

#[cfg(test)]
mod tests {
    use crate::ApolloCompiler;
    use crate::HirDatabase;

    #[test]
    fn field_definition() {
        let input = r#"
schema {
  query Query
}

type Query {
  foo: String
  creature: Creature
}

interface Creature {
  name: String
}

query {
  foo
  creature {
    name
  }
}
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "test.graphql");
        let db = &compiler.db;
        let all_ops = db.all_operations();
        let default_query_op = all_ops
            .iter()
            .find(|op| op.name().is_none())
            .expect("default query not found");

        let sel_set = default_query_op.selection_set();
        let query_type = default_query_op
            .object_type(&compiler.db)
            .expect("query type not found");

        let sel_foo_field_def = sel_set
            .field("foo")
            .expect("query.foo selection field not found")
            .field_definition(db)
            .expect("field_definition returned none for query.foo");

        let query_foo_field_def = query_type
            .field(db, "foo")
            .expect("foo field not found on query type");

        // assert that field_definition() returns a field def for object types
        assert_eq!(&sel_foo_field_def, query_foo_field_def);

        let creature_type = db
            .find_interface_by_name("Creature".to_owned())
            .expect("creature type not found");

        let sel_creature_name_field_def = sel_set
            .field("creature")
            .expect("creature field not found on query selection")
            .selection_set()
            .field("name")
            .expect("name field not found on creature selection")
            .field_definition(db)
            .expect("field definition not found on creature.name selection");

        let hir_creature_field_def = creature_type
            .field("name")
            .expect("name field not found on creature type");

        // assert that field_definition() also returns a field def for interface types
        assert_eq!(hir_creature_field_def, &sel_creature_name_field_def)
    }
}
