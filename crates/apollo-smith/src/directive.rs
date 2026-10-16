use crate::argument::Argument;
use crate::argument::ArgumentsDef;
use crate::description::Description;
use crate::input_value::InputValue;
use crate::name::Name;
use crate::ty::Ty;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Arbitrary;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashSet;

/// The `__DirectiveDef` type represents a Directive definition.
///
/// *DirectiveDefinition*:
///     Description? **directive @** Name Arguments Definition? **repeatable**? **on** DirectiveLocations
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Type-System.Directives).
#[derive(Debug, Clone, PartialEq)]
pub struct DirectiveDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) arguments_definition: Option<ArgumentsDef>,
    pub(crate) repeatable: bool,
    pub(crate) directive_locations: IndexSet<DirectiveLocation>,
}

impl From<DirectiveDef> for ast::Definition {
    fn from(dir_def: DirectiveDef) -> Self {
        ast::DirectiveDefinition {
            description: dir_def.description.map(Into::into),
            name: dir_def.name.into(),
            arguments: dir_def
                .arguments_definition
                .map(Into::into)
                .unwrap_or_default(),
            repeatable: dir_def.repeatable,
            locations: dir_def
                .directive_locations
                .into_iter()
                .map(Into::into)
                .collect(),
            directives: Default::default(),
        }
        .into()
    }
}

impl TryFrom<apollo_parser::cst::DirectiveDefinition> for DirectiveDef {
    type Error = crate::FromError;

    fn try_from(
        directive_def: apollo_parser::cst::DirectiveDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            description: directive_def
                .description()
                .and_then(|d| d.string_value())
                .map(|s| Description::from(Into::<String>::into(s))),
            name: directive_def.name().unwrap().into(),
            arguments_definition: directive_def
                .arguments_definition()
                .map(ArgumentsDef::try_from)
                .transpose()?,
            repeatable: directive_def.repeatable_token().is_some(),
            directive_locations: directive_def
                .directive_locations()
                .map(|dls| {
                    dls.directive_locations()
                        .map(|dl| DirectiveLocation::from(dl.text().unwrap().to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

/// The `__Directive` type represents a Directive, it provides a way to describe alternate runtime execution and type validation behavior in a GraphQL document.
///
/// *Directive*:
///     @ Name Arguments?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#sec-Language.Directives).
#[derive(Debug, Clone, PartialEq)]
pub struct Directive {
    pub(crate) name: Name,
    pub(crate) arguments: Vec<Argument>,
}

impl From<Directive> for ast::Directive {
    fn from(directive: Directive) -> Self {
        Self {
            name: directive.name.into(),
            arguments: directive
                .arguments
                .into_iter()
                .map(|a| Node::new(a.into()))
                .collect(),
        }
    }
}

impl TryFrom<apollo_parser::cst::Directive> for Directive {
    type Error = crate::FromError;

    fn try_from(directive: apollo_parser::cst::Directive) -> Result<Self, Self::Error> {
        Ok(Self {
            name: directive.name().unwrap().into(),
            arguments: directive
                .arguments()
                .map(|args| {
                    args.arguments()
                        .map(Argument::try_from)
                        .collect::<Result<_, _>>()
                })
                .transpose()?
                .unwrap_or_default(),
        })
    }
}

impl Directive {
    pub(crate) fn convert_directives(
        directives: apollo_parser::cst::Directives,
    ) -> Result<IndexMap<Name, Directive>, crate::FromError> {
        directives
            .directives()
            .map(|d| Ok((d.name().unwrap().into(), Directive::try_from(d)?)))
            .collect()
    }

    pub(crate) fn to_ast(map: IndexMap<Name, Directive>) -> ast::DirectiveList {
        map.into_values().map(ast::Directive::from).collect()
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary vector of `Directive`
    pub fn directives(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<IndexMap<Name, Directive>> {
        if self.directive_defs.is_empty() {
            return Ok(IndexMap::new());
        }

        let num_directives = self.u.int_in_range(0..=self.directive_defs.len())?;
        let directives = (0..num_directives)
            .map(|_| self.directive(directive_location))
            .collect::<ArbitraryResult<Vec<_>>>()?
            .into_iter()
            .flat_map(|d| d.map(|d| (d.name.clone(), d)))
            .collect();

        Ok(directives)
    }

    /// Directives for a type definition or extension. A type and its
    /// extensions count as one location, so non-repeatable directives in
    /// `applied` are left out.
    pub(crate) fn type_directives(
        &mut self,
        directive_location: DirectiveLocation,
        applied: &IndexSet<Name>,
    ) -> ArbitraryResult<IndexMap<Name, Directive>> {
        let mut directives = self.directives(directive_location)?;
        directives.retain(|name, _| !applied.contains(name) || self.is_repeatable(name));
        Ok(directives)
    }

    /// Drop directives that would make their own definition reference
    /// itself if applied to `ty` or within it. apollo-compiler follows a
    /// directive's argument types, and the directives on those types, their
    /// enum values, and their input fields.
    pub(crate) fn drop_directives_reaching(
        &self,
        directives: &mut IndexMap<Name, Directive>,
        ty: &Name,
    ) {
        directives.retain(|directive, _| !self.directive_reaches_type(directive, ty));
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
            .filter_map(|def| def.arguments_definition.as_ref())
            .flat_map(|args| &args.input_value_definitions);
        for argument in arguments {
            directives.extend(argument.directives.keys().cloned());
            types.push(argument.ty.name().clone());
        }
    }

    /// Directives and types that the scalar, enum, or input object `ty`
    /// references, across its definition and extensions.
    fn input_type_references(&self, ty: &Name, directives: &mut Vec<Name>, types: &mut Vec<Name>) {
        for scalar in self.scalar_type_defs.iter().filter(|s| &s.name == ty) {
            directives.extend(scalar.directives.keys().cloned());
        }
        for enum_ in self.enum_type_defs.iter().filter(|e| &e.name == ty) {
            directives.extend(enum_.directives.keys().cloned());
            for value in &enum_.enum_values_def {
                directives.extend(value.directives.keys().cloned());
            }
        }
        for input in self.input_object_type_defs.iter().filter(|i| &i.name == ty) {
            directives.extend(input.directives.keys().cloned());
            for field in &input.fields {
                directives.extend(field.directives.keys().cloned());
                types.push(field.ty.name().clone());
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
    ) -> ArbitraryResult<IndexMap<Name, Directive>> {
        let mut directives = self.directives(location)?;
        if conditional {
            directives.extend(self.conditional_directives()?);
        }
        Ok(directives)
    }

    /// Sometimes `@skip`, sometimes `@include`, and sometimes both.
    fn conditional_directives(&mut self) -> ArbitraryResult<IndexMap<Name, Directive>> {
        let mut directives = IndexMap::new();
        for name in ["skip", "include"] {
            // Asking whether to leave the directive out keeps exhausted
            // input, where every ratio is true, from making every selection
            // conditional.
            if self.u.ratio(3, 4)? {
                continue;
            }
            let name = Name::new(name.to_string());
            let condition = Argument {
                name: Name::new("if".to_string()),
                value: self.condition()?,
            };
            directives.insert(
                name.clone(),
                Directive {
                    name,
                    arguments: vec![condition],
                },
            );
        }
        Ok(directives)
    }

    /// Query planners have to handle conditions they can't resolve up front,
    /// so within an operation the condition usually comes from a variable.
    /// Fragment definitions are shared between operations, so they get a
    /// literal.
    fn condition(&mut self) -> ArbitraryResult<InputValue> {
        if self.operation_variables.is_some() && self.u.ratio(2, 3)? {
            let ty = Ty::NonNull(Box::new(Ty::Named(Name::new("Boolean".to_string()))));
            return Ok(InputValue::Variable(self.variable_for(&ty)?));
        }
        Ok(InputValue::Boolean(self.u.arbitrary()?))
    }

    /// Create an arbitrary `Directive` given a directive location
    pub fn directive(
        &mut self,
        directive_location: DirectiveLocation,
    ) -> ArbitraryResult<Option<Directive>> {
        let available_directive_defs: Vec<&DirectiveDef> = self
            .directive_defs
            .iter()
            .filter(|dd| {
                dd.directive_locations.is_empty()
                    || dd.directive_locations.contains(&directive_location)
            })
            .collect();
        if available_directive_defs.is_empty() {
            return Ok(None);
        }
        let directive_def = self.u.choose(&available_directive_defs)?;

        let name = directive_def.name.clone();
        let arguments = directive_def
            .arguments_definition
            .clone()
            .map(|args_def| self.arguments_with_def(&args_def))
            .unwrap_or_else(|| Ok(vec![]))?;

        Ok(Some(Directive { name, arguments }))
    }

    /// Create an arbitrary `DirectiveDef`
    pub fn directive_def(&mut self) -> ArbitraryResult<DirectiveDef> {
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

        Ok(DirectiveDef {
            description,
            name,
            arguments_definition,
            repeatable,
            directive_locations,
        })
    }

    /// Create an arbitrary `IndexSet` of `DirectiveLocation`
    pub fn directive_locations(&mut self) -> ArbitraryResult<IndexSet<DirectiveLocation>> {
        (1..self.u.int_in_range(2..=5usize)?)
            .map(|_| self.u.arbitrary())
            .collect::<ArbitraryResult<IndexSet<_>>>()
    }
}

/// The `__DirectiveLocation` type represents a Directive location.
#[derive(Debug, Clone, PartialEq, Hash, Eq, Arbitrary, Copy)]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    VariableDefinition,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

impl From<DirectiveLocation> for ast::DirectiveLocation {
    fn from(dl: DirectiveLocation) -> Self {
        match dl {
            DirectiveLocation::Query => Self::Query,
            DirectiveLocation::Mutation => Self::Mutation,
            DirectiveLocation::Subscription => Self::Subscription,
            DirectiveLocation::Field => Self::Field,
            DirectiveLocation::FragmentDefinition => Self::FragmentDefinition,
            DirectiveLocation::FragmentSpread => Self::FragmentSpread,
            DirectiveLocation::InlineFragment => Self::InlineFragment,
            DirectiveLocation::VariableDefinition => Self::VariableDefinition,
            DirectiveLocation::Schema => Self::Schema,
            DirectiveLocation::Scalar => Self::Scalar,
            DirectiveLocation::Object => Self::Object,
            DirectiveLocation::FieldDefinition => Self::FieldDefinition,
            DirectiveLocation::ArgumentDefinition => Self::ArgumentDefinition,
            DirectiveLocation::Interface => Self::Interface,
            DirectiveLocation::Union => Self::Union,
            DirectiveLocation::Enum => Self::Enum,
            DirectiveLocation::EnumValue => Self::EnumValue,
            DirectiveLocation::InputObject => Self::InputObject,
            DirectiveLocation::InputFieldDefinition => Self::InputFieldDefinition,
        }
    }
}

impl From<DirectiveLocation> for String {
    fn from(dl: DirectiveLocation) -> Self {
        match dl {
            DirectiveLocation::Query => String::from("QUERY"),
            DirectiveLocation::Mutation => String::from("MUTATION"),
            DirectiveLocation::Subscription => String::from("SUBSCRIPTION"),
            DirectiveLocation::Field => String::from("FIELD"),
            DirectiveLocation::FragmentDefinition => String::from("FRAGMENT_DEFINITION"),
            DirectiveLocation::FragmentSpread => String::from("FRAGMENT_SPREAD"),
            DirectiveLocation::InlineFragment => String::from("INLINE_FRAGMENT"),
            DirectiveLocation::VariableDefinition => String::from("VARIABLE_DEFINITION"),
            DirectiveLocation::Schema => String::from("SCHEMA"),
            DirectiveLocation::Scalar => String::from("SCALAR"),
            DirectiveLocation::Object => String::from("OBJECT"),
            DirectiveLocation::FieldDefinition => String::from("FIELD_DEFINITION"),
            DirectiveLocation::ArgumentDefinition => String::from("ARGUMENT_DEFINITION"),
            DirectiveLocation::Interface => String::from("INTERFACE"),
            DirectiveLocation::Union => String::from("UNION"),
            DirectiveLocation::Enum => String::from("ENUM"),
            DirectiveLocation::EnumValue => String::from("ENUM_VALUE"),
            DirectiveLocation::InputObject => String::from("INPUT_OBJECT"),
            DirectiveLocation::InputFieldDefinition => String::from("INPUT_FIELD_DEFINITION"),
        }
    }
}

impl From<String> for DirectiveLocation {
    fn from(dl: String) -> Self {
        match dl.as_str() {
            "QUERY" => DirectiveLocation::Query,
            "MUTATION" => DirectiveLocation::Mutation,
            "SUBSCRIPTION" => DirectiveLocation::Subscription,
            "FIELD" => DirectiveLocation::Field,
            "FRAGMENT_DEFINITION" => DirectiveLocation::FragmentDefinition,
            "FRAGMENT_SPREAD" => DirectiveLocation::FragmentSpread,
            "INLINE_FRAGMENT" => DirectiveLocation::InlineFragment,
            "VARIABLE_DEFINITION" => DirectiveLocation::VariableDefinition,
            "SCHEMA" => DirectiveLocation::Schema,
            "SCALAR" => DirectiveLocation::Scalar,
            "OBJECT" => DirectiveLocation::Object,
            "FIELD_DEFINITION" => DirectiveLocation::FieldDefinition,
            "ARGUMENT_DEFINITION" => DirectiveLocation::ArgumentDefinition,
            "INTERFACE" => DirectiveLocation::Interface,
            "UNION" => DirectiveLocation::Union,
            "ENUM" => DirectiveLocation::Enum,
            "ENUM_VALUE" => DirectiveLocation::EnumValue,
            "INPUT_OBJECT" => DirectiveLocation::InputObject,
            "INPUT_FIELD_DEFINITION" => DirectiveLocation::InputFieldDefinition,
            other => unreachable!(
                "cannot have {} as a directive location. Documentation: https://spec.graphql.org/September2025/#DirectiveLocation",
                other
            ),
        }
    }
}

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
            let directives = builder.conditional_directives().expect("conditions");
            let defined = builder.operation_variables.take().expect("variables").definitions;
            let boolean = Ty::NonNull(Box::new(Ty::Named(Name::new("Boolean".to_string()))));
            for (name, directive) in &directives {
                prop_assert!(name.name == "skip" || name.name == "include", "{name:?}");
                prop_assert_eq!(directive.arguments.len(), 1);
                let condition = &directive.arguments[0];
                prop_assert_eq!(condition.name.name.as_str(), "if");
                match &condition.value {
                    InputValue::Boolean(_) => {}
                    InputValue::Variable(variable) => prop_assert!(
                        defined.iter().any(|d| &d.name == variable && d.ty == boolean),
                        "{variable:?} isn't a defined Boolean! variable"
                    ),
                    other => prop_assert!(false, "unexpected condition {other:?}"),
                }
            }
        }
    }
}
