use crate::definition::TypeDefinition;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::Argument;
use apollo_compiler::ast::Directive;
use apollo_compiler::ast::DirectiveDefinition;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::Value;
use apollo_compiler::name;
use apollo_compiler::ty;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashSet;

impl DocumentBuilder<'_> {
    /// Create an arbitrary vector of `Directive`
    pub fn directives(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<ast::DirectiveList> {
        if self.directive_defs.is_empty() {
            return Ok(ast::DirectiveList::new());
        }

        let num_directives = self.u.int_in_range(0..=self.directive_defs.len() * 2)?;
        let mut directives = ast::DirectiveList::new();
        let mut applied = HashSet::new();
        for _ in 0..num_directives {
            if let Some(directive) = self.directive(directive_location)? {
                if self.is_repeatable(&directive.name) || applied.insert(directive.name.clone()) {
                    directives.push(Node::new(directive));
                }
            }
        }
        Ok(directives)
    }

    /// Directives for a type definition or extension. A type and its
    /// extensions count as one location, so non-repeatable directives in
    /// `applied` are left out.
    pub(crate) fn type_directives(
        &mut self,
        directive_location: DirectiveLocation,
        applied: &IndexSet<Name>,
    ) -> ArbitraryResult<ast::DirectiveList> {
        let mut directives = self.directives(directive_location)?;
        directives.retain(|directive| {
            !applied.contains(&directive.name) || self.is_repeatable(&directive.name)
        });
        Ok(directives)
    }

    /// Drop directives that would make their own definition reference
    /// itself if applied to `ty` or within it. apollo-compiler follows a
    /// directive's argument types, and the directives on those types, their
    /// enum values, and their input fields.
    pub(crate) fn drop_directives_reaching(&self, directives: &mut ast::DirectiveList, ty: &Name) {
        directives.retain(|directive| !self.directive_reaches_type(&directive.name, ty));
    }

    fn directive_reaches_type(&self, directive: &Name, ty: &Name) -> bool {
        self.reaches(vec![directive.clone()], Vec::new(), Reference::Type(ty))
    }

    /// Whether giving the input object `owner` a field of type `field_type`
    /// would make some directive's definition reference itself, because the
    /// directive already reaches `owner` and `field_type` reaches it.
    pub(crate) fn input_field_type_would_cycle(&self, owner: &Name, field_type: &Name) -> bool {
        self.directive_defs.iter().any(|def| {
            self.directive_reaches_type(&def.name, owner)
                && self.reaches(
                    Vec::new(),
                    vec![field_type.clone()],
                    Reference::Directive(&def.name),
                )
        })
    }

    /// Whether `target` can be reached from `directives` and `types` by
    /// following what apollo-compiler follows when checking a directive
    /// definition for references to itself.
    fn reaches(
        &self,
        mut directives: Vec<Name>,
        mut types: Vec<Name>,
        target: Reference<'_>,
    ) -> bool {
        let mut seen_directives = HashSet::new();
        let mut seen_types = HashSet::new();
        loop {
            if let Some(directive) = directives.pop() {
                if target == Reference::Directive(&directive) {
                    return true;
                }
                if seen_directives.insert(directive.clone()) {
                    self.directive_references(&directive, &mut directives, &mut types);
                }
            } else if let Some(ty) = types.pop() {
                if target == Reference::Type(&ty) {
                    return true;
                }
                if seen_types.insert(ty.clone()) {
                    self.input_type_references(&ty, &mut directives, &mut types);
                }
            } else {
                return false;
            }
        }
    }

    /// Directives and types that `directive`'s arguments reference.
    fn directive_references(
        &self,
        directive: &Name,
        directives: &mut Vec<Name>,
        types: &mut Vec<Name>,
    ) {
        let arguments = self
            .directive_defs
            .iter()
            .filter(|def| &def.name == directive)
            .flat_map(|def| &def.arguments);
        for argument in arguments {
            directives.extend(
                argument
                    .directives
                    .iter()
                    .map(|directive| directive.name.clone()),
            );
            types.push(argument.ty.inner_named_type().clone());
        }
    }

    /// Directives and types that the scalar, enum, or input object `ty`
    /// references, across its definition and extensions.
    fn input_type_references(&self, ty: &Name, directives: &mut Vec<Name>, types: &mut Vec<Name>) {
        for scalar in self.scalar_type_defs.iter().filter(|s| s.type_name() == ty) {
            directives.extend(
                scalar
                    .directives()
                    .iter()
                    .map(|directive| directive.name.clone()),
            );
        }
        for enum_ in self.enum_type_defs.iter().filter(|e| e.type_name() == ty) {
            directives.extend(
                enum_
                    .directives()
                    .iter()
                    .map(|directive| directive.name.clone()),
            );
            for value in enum_.enum_values() {
                directives.extend(
                    value
                        .directives
                        .iter()
                        .map(|directive| directive.name.clone()),
                );
            }
        }
        for input in self
            .input_object_type_defs
            .iter()
            .filter(|i| i.type_name() == ty)
        {
            directives.extend(
                input
                    .directives()
                    .iter()
                    .map(|directive| directive.name.clone()),
            );
            for field in input.input_fields() {
                directives.extend(
                    field
                        .directives
                        .iter()
                        .map(|directive| directive.name.clone()),
                );
                types.push(field.ty.inner_named_type().clone());
            }
        }
    }

    fn is_repeatable(&self, name: &Name) -> bool {
        self.directive_defs
            .iter()
            .any(|def| &def.name == name && def.repeatable)
    }

    /// Directives for a field, fragment spread, or inline fragment in an
    /// executable document, adding `@skip` and `@include` when `conditional`
    /// is set. The root field of a subscription can't be conditional.
    pub(crate) fn selection_directives(
        &mut self,
        location: DirectiveLocation,
        conditional: bool,
    ) -> ArbitraryResult<ast::DirectiveList> {
        let mut directives = self.directives(location)?;
        if conditional {
            directives.extend(self.conditional_directives()?);
        }
        Ok(directives)
    }

    /// Sometimes `@skip`, sometimes `@include`, and sometimes both.
    fn conditional_directives(&mut self) -> ArbitraryResult<ast::DirectiveList> {
        let mut directives = IndexMap::new();
        for name in [name!(skip), name!(include)] {
            // Asking whether to leave the directive out keeps exhausted
            // input, where every ratio is true, from making every selection
            // conditional.
            if self.u.ratio(3, 4)? {
                continue;
            }
            let condition = Argument {
                name: name!(if),
                value: Node::new(self.condition()?),
            };
            directives.insert(
                name.clone(),
                Directive {
                    name,
                    arguments: vec![Node::new(condition)],
                },
            );
        }
        Ok(directives.into_values().collect())
    }

    /// Query planners have to handle conditions they can't resolve up front,
    /// so within an operation the condition usually comes from a variable.
    /// Fragment definitions are shared between operations, so they get a
    /// literal.
    fn condition(&mut self) -> ArbitraryResult<Value> {
        if self.operation_variables.is_some() && self.u.ratio(2, 3)? {
            let ty = ty!(Boolean!);
            return Ok(Value::Variable(self.variable_for(&ty)?));
        }
        Ok(Value::Boolean(self.u.arbitrary()?))
    }

    /// Create an arbitrary `Directive` given a directive location
    pub fn directive(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<Option<Directive>> {
        let available_directive_defs: Vec<&Node<DirectiveDefinition>> = self
            .directive_defs
            .iter()
            .filter(|dd| dd.locations.is_empty() || dd.locations.contains(&directive_location))
            .collect();
        if available_directive_defs.is_empty() {
            return Ok(None);
        }
        let directive_def = self.u.choose(&available_directive_defs)?;

        let name = directive_def.name.clone();
        let definitions = directive_def.arguments.clone();
        let arguments = self.arguments_with_def(&definitions)?;

        Ok(Some(Directive {
            name,
            arguments: arguments.into_iter().map(Node::new).collect(),
        }))
    }

    /// Create an arbitrary `DirectiveDefinition`
    pub fn directive_def(&mut self) -> ArbitraryResult<DirectiveDefinition> {
        let description = self.optional_description()?;
        let name = self.type_name()?;
        let arguments_definition = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.arguments_definition())
            .transpose()?;
        let repeatable = self.u.arbitrary().unwrap_or(false);
        let directive_locations = self.directive_locations()?;

        Ok(DirectiveDefinition {
            description,
            name,
            arguments: arguments_definition.unwrap_or_default(),
            repeatable,
            locations: directive_locations.into_iter().collect(),
            directives: ast::DirectiveList::new(),
        })
    }

    /// Create an arbitrary `IndexSet` of `DirectiveLocation`
    pub fn directive_locations(&mut self) -> ArbitraryResult<IndexSet<DirectiveLocation>> {
        (1..self.u.int_in_range(2..=5usize)?)
            .map(|_| self.u.choose(DIRECTIVE_LOCATIONS).copied())
            .collect::<ArbitraryResult<IndexSet<_>>>()
    }
}

const DIRECTIVE_LOCATIONS: &[DirectiveLocation] = &[
    DirectiveLocation::Query,
    DirectiveLocation::Mutation,
    DirectiveLocation::Subscription,
    DirectiveLocation::Field,
    DirectiveLocation::FragmentDefinition,
    DirectiveLocation::FragmentSpread,
    DirectiveLocation::InlineFragment,
    DirectiveLocation::VariableDefinition,
    DirectiveLocation::Schema,
    DirectiveLocation::Scalar,
    DirectiveLocation::Object,
    DirectiveLocation::FieldDefinition,
    DirectiveLocation::ArgumentDefinition,
    DirectiveLocation::Interface,
    DirectiveLocation::Union,
    DirectiveLocation::Enum,
    DirectiveLocation::EnumValue,
    DirectiveLocation::InputObject,
    DirectiveLocation::InputFieldDefinition,
    DirectiveLocation::DirectiveDefinition,
];

#[derive(PartialEq)]
enum Reference<'a> {
    Directive(&'a Name),
    Type(&'a Name),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::generated;
    use crate::test_support::input;
    use crate::test_support::prop_assert_valid;
    use crate::variable::OperationVariables;
    use crate::DocumentBuilder;
    use arbitrary::Unstructured;
    use proptest::prelude::*;

    #[test]
    fn directive_cycles_ignore_application_arguments() {
        let document = ast::Document::parse(
            "type Query { f: T } scalar T
             directive @d(x: T) repeatable on SCALAR
             scalar S @d(x: 1) @d(x: 2)",
            "schema.graphql",
        )
        .expect("parse schema");
        document.to_schema_validate().expect("valid seed");
        let mut directives = document.definitions.last().unwrap().directives().clone();
        let mut u = Unstructured::new(&[]);
        let builder = DocumentBuilder::with_document(&mut u, document).expect("builder");
        builder.drop_directives_reaching(&mut directives, &name!(T));
        assert!(directives.is_empty());
    }

    #[test]
    fn repeatable_directives_can_be_applied_more_than_once() {
        let mut document = ast::Document::parse(
            "type Query { f: Int } directive @d repeatable on SCALAR",
            "schema.graphql",
        )
        .expect("parse schema");
        let mut u = Unstructured::new(&[2]);
        let mut builder =
            DocumentBuilder::with_document(&mut u, document.clone()).expect("builder");
        let directives = builder
            .directives(DirectiveLocation::Scalar)
            .expect("directives");
        assert_eq!(directives.len(), 2);
        document.definitions.push(
            ast::ScalarTypeDefinition {
                description: None,
                name: name!(S),
                directives,
            }
            .into(),
        );
        document
            .to_schema_validate()
            .expect("valid repeated applications");
    }

    #[test]
    fn non_repeatable_directives_are_applied_once() {
        let document = ast::Document::parse(
            "type Query { f: Int } directive @d on SCALAR",
            "schema.graphql",
        )
        .expect("parse schema");
        let mut u = Unstructured::new(&[2]);
        let mut builder = DocumentBuilder::with_document(&mut u, document).expect("builder");
        let directives = builder
            .directives(DirectiveLocation::Scalar)
            .expect("directives");
        assert_eq!(directives.len(), 1);
    }

    const SCHEMA: &str = "schema { query: Query }
        type Query { a: Int }
        directive @d(x: Int, y: [String!]!) on FIELD_DEFINITION | OBJECT | ARGUMENT_DEFINITION";

    // A subscription's root field can't be conditional.
    const CONDITIONS: &str = "schema { query: Query, subscription: Subscription }
        type Query { a(x: Int): Int, b: B, c: Int }
        type Subscription { s: B, t: Int }
        type B { d: Int, e(y: String): String }";

    proptest! {
        #[test]
        fn generates_valid_definitions(case in generated(SCHEMA, |b| b.directive_def().map(Some))) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }

        #[test]
        fn conditional_selections_stay_valid(case in generated(CONDITIONS, |b| b.operation_definition())) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }

        #[test]
        fn conditions_are_literals_or_defined_boolean_variables(data in input()) {
            let mut u = Unstructured::new(&data);
            let mut builder = DocumentBuilder::new(&mut u);
            builder.operation_variables = Some(OperationVariables::default());
            let directives: ast::DirectiveList = builder.conditional_directives().expect("conditions");
            let defined = builder.operation_variables.take().expect("variables").definitions;
            let boolean = ty!(Boolean!);
            for directive in &directives {
                let name = &directive.name;
                prop_assert!(name == "skip" || name == "include", "{name:?}");
                prop_assert_eq!(directive.arguments.len(), 1);
                let condition = &directive.arguments[0];
                prop_assert_eq!(condition.name.as_str(), "if");
                match &*condition.value {
                    Value::Boolean(_) => {}
                    Value::Variable(variable) => prop_assert!(
                        defined.iter().any(|d| &d.name == variable && *d.ty == boolean),
                        "{variable:?} isn't a defined Boolean! variable"
                    ),
                    other => prop_assert!(false, "unexpected condition {other:?}"),
                }
            }
        }
    }
}
