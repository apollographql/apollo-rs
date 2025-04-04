//! Supporting APIs for [GraphQL validation](https://spec.graphql.org/October2021/#sec-Validation)
//! and other kinds of errors.

use crate::coordinate::SchemaCoordinate;
#[cfg(doc)]
use crate::ExecutableDocument;
use crate::Schema;

pub(crate) mod argument;
pub(crate) mod diagnostics;
pub(crate) mod directive;
pub(crate) mod enum_;
pub(crate) mod field;
pub(crate) mod fragment;
pub(crate) mod input_object;
pub(crate) mod interface;
pub(crate) mod object;
pub(crate) mod operation;
pub(crate) mod scalar;
pub(crate) mod schema;
pub(crate) mod selection;
pub(crate) mod union_;
pub(crate) mod value;
pub(crate) mod variable;

use crate::collections::HashMap;
use crate::collections::HashSet;
use crate::collections::IndexSet;
use crate::diagnostic::CliReport;
use crate::diagnostic::Diagnostic;
use crate::diagnostic::ToCliReport;
use crate::executable::BuildError as ExecutableBuildError;
use crate::executable::ConflictingFieldArgument;
use crate::executable::ConflictingFieldName;
use crate::executable::ConflictingFieldType;
use crate::executable::VariableDefinition;
use crate::parser::SourceMap;
use crate::parser::SourceSpan;
use crate::response::GraphQLError;
use crate::schema::BuildError as SchemaBuildError;
use crate::schema::Implementers;
use crate::Name;
use crate::Node;
use std::fmt;
use std::sync::Arc;
use std::sync::OnceLock;

/// Wraps a [`Schema`] or [`ExecutableDocument`] to mark it
/// as [valid](https://spec.graphql.org/October2021/#sec-Validation).
///
/// This is obtained either by running validation with one of:
///
/// * [`Schema::parse_and_validate`]
/// * [`Schema::validate`]
/// * [`ExecutableDocument::parse_and_validate`]
/// * [`ExecutableDocument::validate`]
/// * [`coerce_variable_values`][crate::request::coerce_variable_values]
///
/// … or by explicitly skipping it with [`Valid::assume_valid`].
///
/// The schema or document inside `Valid<T>` is immutable (`&mut T` is not given out).
/// It can be extracted with [`into_inner`][Self::into_inner],
/// such as to mutate it then possibly re-validate it.
#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Valid<T>(pub(crate) T);

impl<T> Valid<T> {
    /// Construct a `Valid` document without actually running validation.
    ///
    /// This takes ownership of the document.
    /// See also [`assume_valid_ref`][Self::assume_valid_ref] which only requires a reference.
    ///
    /// The caller takes responsibility to ascertain that
    /// the document is known through some other means to be valid.
    /// For example, if it was loaded from some external storage
    /// where it was only stored after validation.
    pub fn assume_valid(document: T) -> Self {
        Self(document)
    }

    /// Mark a reference as `Valid` without actually running validation.
    ///
    /// See also [`assume_valid`][Self::assume_valid] returns an owned `Valid<T>`
    /// instead of only a reference.
    ///
    /// The caller takes responsibility to ascertain that
    /// the document is known through some other means to be valid.
    /// For example, if it was loaded from some external storage
    /// where it was only stored after validation.
    pub fn assume_valid_ref(document: &T) -> &Self {
        let ptr: *const T = document;
        let ptr: *const Valid<T> = ptr.cast();
        // SAFETY: `repr(transparent)` makes it valid to transmute `&T` to `&Valid<T>`:
        // <https://doc.rust-lang.org/nomicon/other-reprs.html#reprtransparent>
        unsafe { &*ptr }
    }

    /// Extract the schema or document, such as to mutate it then possibly re-validate it.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> std::ops::Deref for Valid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> AsRef<T> for Valid<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: fmt::Display> fmt::Display for Valid<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Shared context with things that may be used throughout executable validation.
#[derive(Debug)]
pub(crate) struct ExecutableValidationContext<'a> {
    /// When None, rules that require a schema to validate are disabled.
    schema: Option<&'a Schema>,
    /// `schema.implementers_map()` is expensive to compute. This caches it for reuse.
    implementers_map: OnceLock<HashMap<Name, Implementers>>,
}

impl<'a> ExecutableValidationContext<'a> {
    pub(crate) fn new(schema: Option<&'a Schema>) -> Self {
        Self {
            schema,
            implementers_map: Default::default(),
        }
    }

    /// Returns the schema to validate against, if any.
    pub(crate) fn schema(&self) -> Option<&'a Schema> {
        self.schema
    }

    /// Returns a cached reference to the implementers map.
    pub(crate) fn implementers_map(&self) -> &HashMap<Name, Implementers> {
        self.implementers_map.get_or_init(|| {
            self.schema
                .map(|schema| schema.implementers_map())
                .unwrap_or_default()
        })
    }

    /// Returns a context for operation validation.
    pub(crate) fn operation_context<'o>(
        &'o self,
        variables: &'o [Node<VariableDefinition>],
    ) -> OperationValidationContext<'o> {
        OperationValidationContext {
            executable: self,
            variables,
            validated_fragments: HashSet::default(),
        }
    }
}

/// Shared context when validating things inside an operation.
#[derive(Debug)]
pub(crate) struct OperationValidationContext<'a> {
    /// Parent context. Using a reference so the `OnceLock` is shared between all operation
    /// contexts.
    executable: &'a ExecutableValidationContext<'a>,
    /// The variables defined for this operation.
    pub(crate) variables: &'a [Node<VariableDefinition>],
    pub(crate) validated_fragments: HashSet<Name>,
}

impl<'a> OperationValidationContext<'a> {
    pub(crate) fn schema(&self) -> Option<&'a Schema> {
        self.executable.schema
    }

    /// Returns a cached reference to the implementers map.
    pub(crate) fn implementers_map(&self) -> &HashMap<Name, Implementers> {
        self.executable.implementers_map()
    }
}

/// A conversion failed with some errors, but also resulted in a partial document.
///
/// The [`Debug`][fmt::Debug] trait is implemented by forwarding to [`Self::errors`] and
/// ignoring [`Self::partial`].
/// This is so that the panic message prints (only) errors when [`.unwrap()`][Result::unwrap]
/// is called on a `Result<_, WithError<_>>` value as returned by various APIs.
pub struct WithErrors<T> {
    /// The partial result of the conversion.
    /// Some components may be missing,
    /// for example if an error causes them not to be representable in the target data structure.
    pub partial: T,

    /// Errors collected during the conversion.
    /// Should be non-empty when `WithError` is returned.
    pub errors: DiagnosticList,
}

impl<T> fmt::Debug for WithErrors<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.errors.fmt(f)
    }
}

impl<T> fmt::Display for WithErrors<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.errors.fmt(f)
    }
}

/// Returned as an error for situtations that should not happen with a valid schema or document.
///
/// Since the relevant APIs take [`Valid<_>`][crate::validation::Valid] parameters,
/// either apollo-compiler has a validation bug
/// or [`assume_valid`][crate::validation::Valid::assume_valid] was used incorrectly.
///
/// Can be [converted][std::convert] to [`GraphQLError`],
/// which populates [`extensions`][GraphQLError::extensions]
/// with a `"APOLLO_SUSPECTED_VALIDATION_BUG": true` entry.
#[derive(Debug, Clone)]
pub(crate) struct SuspectedValidationBug {
    pub message: String,
    pub location: Option<SourceSpan>,
}

/// A collection of diagnostics returned by some validation method
#[derive(Clone)]
pub struct DiagnosticList {
    pub(crate) sources: SourceMap,
    diagnostics_data: Vec<DiagnosticData>,
}

// TODO(@goto-bus-stop) Can/should this be non-pub?
#[derive(thiserror::Error, Debug, Clone)]
#[error("{details}")]
pub struct DiagnosticData {
    location: Option<SourceSpan>,
    details: Details,
}

#[derive(thiserror::Error, Debug, Clone)]
pub(crate) enum Details {
    #[error("{message}")]
    ParserLimit { message: String },
    #[error("syntax error: {message}")]
    SyntaxError { message: String },
    #[error("{0}")]
    SchemaBuildError(SchemaBuildError),
    #[error("{0}")]
    ExecutableBuildError(ExecutableBuildError),
    // TODO: Merge ValidationError into this enum
    #[error(transparent)]
    CompilerDiagnostic(diagnostics::DiagnosticData),
    #[error("too much recursion")]
    RecursionLimitError,
}

impl DiagnosticData {
    /// Returns the internal error name for an (operation) validation error.
    /// This is meant for debugging apollo-rs, not for public consumption.
    #[doc(hidden)]
    pub fn unstable_error_name(&self) -> Option<&'static str> {
        match &self.details {
            Details::CompilerDiagnostic(diagnostic) => {
                use diagnostics::DiagnosticData::*;
                Some(match diagnostic {
                    RecursionError { .. } => "RecursionError",
                    UniqueVariable { .. } => "UniqueVariable",
                    UniqueArgument { .. } => "UniqueArgument",
                    UniqueInputValue { .. } => "UniqueInputValue",
                    UndefinedArgument { .. } => "UndefinedArgument",
                    UndefinedDefinition { .. } => "UndefinedDefinition",
                    UndefinedDirective { .. } => "UndefinedDirective",
                    UndefinedVariable { .. } => "UndefinedVariable",
                    UndefinedFragment { .. } => "UndefinedFragment",
                    UndefinedEnumValue { .. } => "UndefinedEnumValue",
                    UndefinedInputValue { .. } => "UndefinedInputValue",
                    MissingInterfaceField { .. } => "MissingInterfaceField",
                    RequiredArgument { .. } => "RequiredArgument",
                    RequiredField { .. } => "RequiredField",
                    TransitiveImplementedInterfaces { .. } => "TransitiveImplementedInterfaces",
                    OutputType { .. } => "OutputType",
                    InputType { .. } => "InputType",
                    VariableInputType { .. } => "VariableInputType",
                    QueryRootOperationType => "QueryRootOperationType",
                    UnusedVariable { .. } => "UnusedVariable",
                    RootOperationObjectType { .. } => "RootOperationObjectType",
                    UnionMemberObjectType { .. } => "UnionMemberObjectType",
                    UnsupportedLocation { .. } => "UnsupportedLocation",
                    UnsupportedValueType { .. } => "UnsupportedValueType",
                    IntCoercionError { .. } => "IntCoercionError",
                    FloatCoercionError { .. } => "FloatCoercionError",
                    UniqueDirective { .. } => "UniqueDirective",
                    MissingSubselection { .. } => "MissingSubselection",
                    InvalidFragmentTarget { .. } => "InvalidFragmentTarget",
                    InvalidFragmentSpread { .. } => "InvalidFragmentSpread",
                    UnusedFragment { .. } => "UnusedFragment",
                    DisallowedVariableUsage { .. } => "DisallowedVariableUsage",
                    RecursiveDirectiveDefinition { .. } => "RecursiveDirectiveDefinition",
                    RecursiveInterfaceDefinition { .. } => "RecursiveInterfaceDefinition",
                    RecursiveInputObjectDefinition { .. } => "RecursiveInputObjectDefinition",
                    RecursiveFragmentDefinition { .. } => "RecursiveFragmentDefinition",
                    DeeplyNestedType { .. } => "DeeplyNestedType",
                    EmptyFieldSet { .. } => "EmptyFieldSet",
                    EmptyValueSet { .. } => "EmptyValueSet",
                    EmptyMemberSet { .. } => "EmptyMemberSet",
                    EmptyInputValueSet { .. } => "EmptyInputValueSet",
                    ReservedName { .. } => "ReservedName",
                })
            }
            Details::ExecutableBuildError(error) => Some(match error {
                ExecutableBuildError::UndefinedField { .. } => "UndefinedField",
                ExecutableBuildError::TypeSystemDefinition { .. } => "TypeSystemDefinition",
                ExecutableBuildError::AmbiguousAnonymousOperation => {
                    "AmbiguousAnonymousOperation"
                }
                ExecutableBuildError::OperationNameCollision { .. } => "OperationNameCollision",
                ExecutableBuildError::FragmentNameCollision { .. } => "FragmentNameCollision",
                ExecutableBuildError::UndefinedRootOperation { .. } => "UndefinedRootOperation",
                ExecutableBuildError::UndefinedTypeInNamedFragmentTypeCondition { .. } => {
                    "UndefinedTypeInNamedFragmentTypeCondition"
                }
                ExecutableBuildError::UndefinedTypeInInlineFragmentTypeCondition { .. } => {
                    "UndefinedTypeInInlineFragmentTypeCondition"
                }
                ExecutableBuildError::SubselectionOnScalarType { .. } => "SubselectionOnScalarType",
                ExecutableBuildError::SubselectionOnEnumType { .. } => "SubselectionOnEnumType",
                ExecutableBuildError::SubscriptionUsesMultipleFields { .. } => {
                    "SubscriptionUsesMultipleFields"
                }
                ExecutableBuildError::SubscriptionUsesIntrospection { .. } => {
                    "SubscriptionUsesIntrospection"
                }
                ExecutableBuildError::ConflictingFieldType(_) => "ConflictingFieldType",
                ExecutableBuildError::ConflictingFieldName(_) => "ConflictingFieldName",
                ExecutableBuildError::ConflictingFieldArgument(_) => "ConflictingFieldArgument",
            }),
            Details::RecursionLimitError => Some("RecursionLimitError"),
            _ => None,
        }
    }

    /// Returns an error message for this diagnostic, mimicking the graphql-js format.
    ///
    /// This is meant as a migration path for the Apollo Router, and use by other consumers
    /// is not supported.
    #[doc(hidden)]
    pub fn unstable_compat_message(&self) -> Option<String> {
        match &self.details {
            Details::CompilerDiagnostic(diagnostic) => {
                use diagnostics::DiagnosticData::*;
                match diagnostic {
                    RecursionError { .. } => None,
                    UniqueVariable { name, .. } => Some(format!(
                        r#"There can be only one variable named "${name}"."#
                    )),
                    UniqueArgument { name, .. } => {
                        Some(format!(r#"There can be only one argument named "{name}"."#))
                    }
                    UniqueInputValue { .. } => None,
                    UndefinedArgument {
                        name, coordinate, ..
                    } => Some(format!(
                        r#"Unknown argument "{name}" on field "{coordinate}"."#
                    )),
                    UndefinedDefinition { name } => Some(format!(r#"Unknown type "{name}"."#)),
                    UndefinedDirective { name } => Some(format!(r#"Unknown directive "@{name}"."#)),
                    UndefinedVariable { name } => {
                        Some(format!(r#"Variable "${name}" is not defined."#))
                    }
                    UndefinedFragment { name } => Some(format!(r#"Unknown fragment "{name}"."#)),
                    UndefinedEnumValue {
                        value, definition, ..
                    } => Some(format!(
                        r#"Value "{value}" does not exist in "{definition}" enum."#
                    )),
                    UndefinedInputValue {
                        value, definition, ..
                    } => Some(format!(
                        r#"Field "{value}" is not defined by type "{definition}"."#
                    )),
                    MissingInterfaceField { .. } => None,
                    RequiredArgument {
                        name,
                        coordinate,
                        expected_type,
                        ..
                    } => match coordinate {
                        SchemaCoordinate::FieldArgument(coordinate) => Some(format!(
                            r#"Field "{}" argument "{name}" of type "{expected_type}" is required, but it was not provided."#,
                            coordinate.field,
                        )),
                        SchemaCoordinate::DirectiveArgument(coordinate) => Some(format!(
                            r#"Directive "@{}" argument "{name}" of type "{expected_type}" is required, but it was not provided."#,
                            coordinate.directive,
                        )),
                        // It's always an argument coordinate so we don't need to handle other cases.
                        _ => None,
                    },
                    RequiredField {
                        coordinate,
                        expected_type,
                        ..
                    } => Some(format!(
                        r#"Field "{coordinate}" of required type "{expected_type}" was not provided."#
                    )),
                    TransitiveImplementedInterfaces { .. } => None,
                    OutputType { .. } => None,
                    InputType { .. } => None,
                    VariableInputType { name, ty, .. } => Some(format!(
                        r#"Variable "${name}" cannot be non-input type "{ty}"."#
                    )),
                    QueryRootOperationType => None,
                    UnusedVariable { name } => {
                        Some(format!(r#"Variable "${name}" is never used."#))
                    }
                    RootOperationObjectType { .. } => None,
                    UnionMemberObjectType { .. } => None,
                    UnsupportedLocation { name, location, .. } => Some(format!(
                        r#"Directive "@{name}" may not be used on {location}."#
                    )),
                    UnsupportedValueType { ty, value, .. } => Some(format!(
                        r#"{} cannot represent value: {value}"#,
                        ty.inner_named_type()
                    )),
                    IntCoercionError { value } => {
                        let is_integer = value
                            .chars()
                            // The possible characters in "-1e+100"
                            .all(|c| matches!(c, '-' | '+' | 'e' | '0'..='9'));
                        if is_integer {
                            Some(format!(
                                r#"Int cannot represent non 32-bit signed integer value: {value}"#
                            ))
                        } else {
                            Some(format!(
                                r#"Int cannot represent non-integer value: {value}"#
                            ))
                        }
                    }
                    FloatCoercionError { value } => Some(format!(
                        r#"Float cannot represent non numeric value: {value}"#
                    )),
                    UniqueDirective { name, .. } => Some(format!(
                        r#"The directive "@{name}" can only be used once at this location."#
                    )),
                    MissingSubselection { coordinate, .. } => Some(format!(
                        r#"Field "{field}" of type "{ty}" must have a selection of subfields. Did you mean "{field} {{ ... }}"?"#,
                        ty = coordinate.ty,
                        field = coordinate.attribute,
                    )),
                    InvalidFragmentTarget { name, ty } => {
                        if let Some(name) = name {
                            Some(format!(
                                r#"Fragment "{name}" cannot condition on non composite type "{ty}"."#
                            ))
                        } else {
                            Some(format!(
                                r#"Fragment cannot condition on non composite type "{ty}"."#
                            ))
                        }
                    }
                    InvalidFragmentSpread {
                        name,
                        type_name,
                        type_condition,
                        ..
                    } => {
                        if let Some(name) = name {
                            Some(format!(
                                r#"Fragment "{name}" cannot be spread here as objects of type "{type_name}" can never be of type "{type_condition}"."#
                            ))
                        } else {
                            Some(format!(
                                r#"Fragment cannot be spread here as objects of type "{type_name}" can never be of type "{type_condition}"."#
                            ))
                        }
                    }
                    UnusedFragment { name } => Some(format!(r#"Fragment "{name}" is never used."#)),
                    DisallowedVariableUsage {
                        variable,
                        variable_type,
                        argument_type,
                        ..
                    } => Some(format!(
                        r#"Variable "${variable}" of type "{variable_type}" used in position expecting type "{argument_type}"."#
                    )),
                    RecursiveDirectiveDefinition { .. } => None,
                    RecursiveInterfaceDefinition { .. } => None,
                    RecursiveInputObjectDefinition { .. } => None,
                    RecursiveFragmentDefinition { name, trace, .. } => Some(format!(
                        r#"Cannot spread fragment "{name}" within itself via {}"#,
                        // Some inefficient allocation but :shrug:, not a big deal here
                        trace
                            .iter()
                            .map(|spread| format!(r#""{}""#, spread.fragment_name))
                            .collect::<Vec<_>>()
                            .join(", "),
                    )),
                    DeeplyNestedType { .. } => None,
                    EmptyFieldSet { .. } => None,
                    EmptyValueSet { .. } => None,
                    EmptyMemberSet { .. } => None,
                    EmptyInputValueSet { .. } => None,
                    ReservedName { .. } => None,
                }
            }
            Details::ExecutableBuildError(error) => match error {
                ExecutableBuildError::UndefinedField {
                    type_name,
                    field_name,
                    ..
                } => Some(format!(
                    r#"Cannot query field "{field_name}" on type "{type_name}"."#
                )),
                ExecutableBuildError::TypeSystemDefinition { name, .. } => {
                    if let Some(name) = name {
                        Some(format!(r#"The "{name}" definition is not executable."#))
                    } else {
                        // Among type system definitions, only schema definitions do have a name
                        Some("The schema definition is not executable.".to_string())
                    }
                }
                ExecutableBuildError::AmbiguousAnonymousOperation => {
                    Some("This anonymous operation must be the only defined operation.".to_string())
                }
                ExecutableBuildError::OperationNameCollision {
                    name_at_previous_location,
                } => Some(format!(
                    r#"There can be only one operation named "{name_at_previous_location}"."#
                )),
                ExecutableBuildError::FragmentNameCollision {
                    name_at_previous_location,
                } => Some(format!(
                    r#"There can be only one fragment named "{name_at_previous_location}"."#
                )),
                ExecutableBuildError::UndefinedRootOperation { operation_type } => Some(format!(
                    // no period unlike other messages :zany_face:
                    r#"The schema has no "{operation_type}" root type defined"#
                )),
                ExecutableBuildError::UndefinedTypeInNamedFragmentTypeCondition {
                    type_name,
                    ..
                }
                | ExecutableBuildError::UndefinedTypeInInlineFragmentTypeCondition {
                    type_name,
                    ..
                } => Some(format!(r#"Unknown type "{type_name}"."#)),
                ExecutableBuildError::SubselectionOnScalarType { type_name, path }
                | ExecutableBuildError::SubselectionOnEnumType { type_name, path } => {
                    #[allow(clippy::manual_map)]
                    if let Some(field) = path.nested_fields.last() {
                        Some(format!(
                            r#"Field "{field}" must not have a selection since type "{type_name}" has no subfields"#
                        ))
                    } else {
                        None // Can this happen?
                    }
                }
                ExecutableBuildError::SubscriptionUsesMultipleFields { name, .. } => {
                    if let Some(name) = name {
                        Some(format!(
                            r#"Subscription "{name}" must select only one top level field."#
                        ))
                    } else {
                        Some(
                            "Anonymous Subscription must select only one top level field."
                                .to_string(),
                        )
                    }
                }
                ExecutableBuildError::SubscriptionUsesIntrospection { name, .. } => {
                    if let Some(name) = name {
                        Some(format!(
                            r#"Subscription "{name}" must not select an introspection top level field."#
                        ))
                    } else {
                        Some("Anonymous Subscription must not select an introspection top level field."
                            .to_string())
                    }
                }
                ExecutableBuildError::ConflictingFieldType(inner) => {
                    let ConflictingFieldType {
                        alias,
                        original_type,
                        conflicting_type,
                        ..
                    } = &**inner;
                    Some(format!(
                        r#"Fields "{alias}" conflict because they return conflicting types "{original_type} and "{conflicting_type}". Use different aliases on the fields to fetch both if this was intentional."#
                    ))
                }
                ExecutableBuildError::ConflictingFieldName(inner) => {
                    let ConflictingFieldName {
                        alias,
                        original_selection,
                        conflicting_selection,
                        ..
                    } = &**inner;
                    Some(format!(
                        r#"Fields "{alias}" conflict because "{}" and "{}" are different fields. Use different aliases on the fields to fetch both if this was intentional."#,
                        original_selection.attribute, conflicting_selection.attribute
                    ))
                }
                ExecutableBuildError::ConflictingFieldArgument(inner) => {
                    let ConflictingFieldArgument { alias, .. } = &**inner;
                    Some(format!(
                        r#"Fields "{alias}" conflict because they have differing arguments. Use different aliases on the fields to fetch both if this was intentional."#
                    ))
                }
            },
            _ => None,
        }
    }
}

impl ToCliReport for DiagnosticData {
    fn location(&self) -> Option<SourceSpan> {
        self.location
    }

    fn report(&self, report: &mut CliReport) {
        if let Details::CompilerDiagnostic(diagnostic) = &self.details {
            diagnostic.report(self.location, report);
            return;
        }

        // Every case should also have a label at the main location
        // (preferably saying something not completely redundant with the main message)
        // and may have additional labels.
        // Labels are always optional because locations are always optional,
        // so essential information should be in the main message.
        match &self.details {
            Details::CompilerDiagnostic(_) => unreachable!(),
            Details::ParserLimit { message, .. } => report.with_label_opt(self.location, message),
            Details::SyntaxError { message, .. } => report.with_label_opt(self.location, message),
            Details::SchemaBuildError(err) => match err {
                SchemaBuildError::ExecutableDefinition { .. } => report.with_label_opt(
                    self.location,
                    "remove this definition, or use `parse_mixed()`",
                ),
                SchemaBuildError::SchemaDefinitionCollision {
                    previous_location, ..
                } => {
                    report.with_label_opt(*previous_location, "previous `schema` definition here");
                    report.with_label_opt(self.location, "`schema` redefined here");
                    report.with_help(
                        "merge this definition with the previous one, or use `extend schema`",
                    );
                }
                SchemaBuildError::DirectiveDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `@{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`@{name}` redefined here"));
                    report.with_help("remove or rename one of the definitions");
                }
                SchemaBuildError::TypeDefinitionCollision {
                    previous_location,
                    name,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                    report.with_help("remove or rename one of the definitions, or use `extend`");
                }
                SchemaBuildError::BuiltInScalarTypeRedefinition => {
                    report.with_label_opt(self.location, "remove this scalar definition");
                }
                SchemaBuildError::OrphanSchemaExtension => {
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::OrphanTypeExtension { .. } => {
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::TypeExtensionKindMismatch { def_location, .. } => {
                    report.with_label_opt(*def_location, "type definition");
                    report.with_label_opt(self.location, "extension here")
                }
                SchemaBuildError::DuplicateRootOperation {
                    previous_location,
                    operation_type,
                    ..
                } => {
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{operation_type}` here"),
                    );
                    report.with_label_opt(
                        self.location,
                        format_args!("`{operation_type}` redefined here"),
                    );
                }
                SchemaBuildError::DuplicateImplementsInterfaceInObject {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::DuplicateImplementsInterfaceInInterface {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous implementation of `{name}` here"),
                    );
                    report.with_label_opt(
                        self.location,
                        format_args!("`{name}` implemented again here"),
                    );
                }
                SchemaBuildError::ObjectFieldNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::InterfaceFieldNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::EnumValueNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::UnionMemberNameCollision {
                    name_at_previous_location,
                    ..
                }
                | SchemaBuildError::InputFieldNameCollision {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                }
            },
            Details::ExecutableBuildError(err) => match err {
                ExecutableBuildError::TypeSystemDefinition { .. } => report.with_label_opt(
                    self.location,
                    "remove this definition, or use `parse_mixed()`",
                ),
                ExecutableBuildError::AmbiguousAnonymousOperation => {
                    report.with_label_opt(self.location, "provide a name for this definition");
                    report.with_help(
                        "GraphQL requires operations to be named if the document has more than one",
                    );
                }
                ExecutableBuildError::OperationNameCollision {
                    name_at_previous_location,
                    ..
                }
                | ExecutableBuildError::FragmentNameCollision {
                    name_at_previous_location,
                    ..
                } => {
                    let previous_location = &name_at_previous_location.location();
                    let name = name_at_previous_location;
                    report.with_label_opt(
                        *previous_location,
                        format_args!("previous definition of `{name}` here"),
                    );
                    report.with_label_opt(self.location, format_args!("`{name}` redefined here"));
                }
                ExecutableBuildError::UndefinedRootOperation { operation_type, .. } => {
                    report.with_label_opt(
                        self.location,
                        format_args!(
                            "`{operation_type}` is not defined in the schema and is therefore not supported"
                        ),
                    );
                    report.with_help(format_args!(
                        "consider defining a `{operation_type}` root operation type in your schema"
                    ))
                }
                ExecutableBuildError::UndefinedTypeInNamedFragmentTypeCondition { .. } => {
                    report.with_label_opt(self.location, "type condition here")
                }
                ExecutableBuildError::UndefinedTypeInInlineFragmentTypeCondition {
                    path, ..
                } => {
                    report.with_label_opt(self.location, "type condition here");
                    report.with_note(format_args!("path to the inline fragment: `{path} → ...`"))
                }
                ExecutableBuildError::SubselectionOnScalarType { path, .. }
                | ExecutableBuildError::SubselectionOnEnumType { path, .. } => {
                    report.with_label_opt(self.location, "remove subselections here");
                    report.with_note(format_args!("path to the field: `{path}`"))
                }
                ExecutableBuildError::UndefinedField {
                    field_name,
                    type_name,
                    path,
                    ..
                } => {
                    report.with_label_opt(
                        self.location,
                        format_args!("field `{field_name}` selected here"),
                    );
                    report.with_label_opt(
                        type_name.location(),
                        format_args!("type `{type_name}` defined here"),
                    );
                    report.with_note(format_args!("path to the field: `{path}`"))
                }
                ExecutableBuildError::SubscriptionUsesMultipleFields { fields, .. } => {
                    report.with_label_opt(
                        self.location,
                        format_args!("subscription with {} root fields", fields.len()),
                    );
                    report.with_help(format_args!(
                        "There are {} root fields: {}. This is not allowed.",
                        fields.len(),
                        CommaSeparated(fields)
                    ));
                }
                ExecutableBuildError::SubscriptionUsesIntrospection { field, .. } => {
                    report.with_label_opt(
                        self.location,
                        format_args!("{field} is an introspection field"),
                    );
                }
                ExecutableBuildError::ConflictingFieldType(inner) => {
                    let ConflictingFieldType {
                        alias,
                        original_location,
                        original_coordinate,
                        original_type,
                        conflicting_location,
                        conflicting_coordinate,
                        conflicting_type,
                    } = &**inner;
                    report.with_label_opt(
                        *original_location,
                        format_args!(
                        "`{alias}` is selected from `{original_coordinate}: {original_type}` here"
                    ),
                    );
                    report.with_label_opt(
                    *conflicting_location,
                    format_args!("`{alias}` is selected from `{conflicting_coordinate}: {conflicting_type}` here"),
                );
                }
                ExecutableBuildError::ConflictingFieldArgument(inner) => {
                    let ConflictingFieldArgument {
                        alias,
                        original_location,
                        original_coordinate,
                        original_value,
                        conflicting_location,
                        conflicting_coordinate: _,
                        conflicting_value,
                    } = &**inner;
                    let argument = &original_coordinate.argument;
                    match (original_value, conflicting_value) {
                        (Some(_), Some(_)) => {
                            report.with_label_opt(
                                *original_location,
                                format_args!(
                                    "`{original_coordinate}` is used with one argument value here"
                                ),
                            );
                            report.with_label_opt(
                                *conflicting_location,
                                "but a different value here",
                            );
                        }
                        (Some(_), None) => {
                            report.with_label_opt(
                                *original_location,
                                format!("`{alias}` is selected with argument `{argument}` here",),
                            );
                            report.with_label_opt(
                                *conflicting_location,
                                format!("but argument `{argument}` is not provided here"),
                            );
                        }
                        (None, Some(_)) => {
                            report.with_label_opt(
                                *conflicting_location,
                                format!("`{alias}` is selected with argument `{argument}` here",),
                            );
                            report.with_label_opt(
                                *original_location,
                                format!("but argument `{argument}` is not provided here"),
                            );
                        }
                        (None, None) => unreachable!(),
                    }
                    report.with_help("The same name cannot be selected multiple times with different arguments, because it's not clear which set of arguments should be used to fill the response. If you intend to use diverging arguments, consider adding an alias to differentiate");
                }
                ExecutableBuildError::ConflictingFieldName(inner) => {
                    let ConflictingFieldName {
                        alias: field,
                        original_selection,
                        original_location,
                        conflicting_selection,
                        conflicting_location,
                    } = &**inner;
                    report.with_label_opt(
                        *original_location,
                        format_args!("`{field}` is selected from `{original_selection}` here"),
                    );
                    report.with_label_opt(
                        *conflicting_location,
                        format_args!("`{field}` is selected from `{conflicting_selection}` here"),
                    );

                    report.with_help("Both fields may be present on the schema type, so it's not clear which one should be used to fill the response");
                }
            },
            Details::RecursionLimitError => {}
        }
    }
}

impl Diagnostic<'_, DiagnosticData> {
    /// Get a [`serde`]-serializable version of the current diagnostic. This method mimicks the
    /// shape and message of errors produced by graphql-js.
    ///
    /// This is only for use by the Apollo Router, any other consumer is not supported.
    #[doc(hidden)]
    pub fn unstable_to_json_compat(&self) -> GraphQLError {
        GraphQLError::new(
            self.error
                .unstable_compat_message()
                .unwrap_or_else(|| self.error.to_string()),
            self.error.location(),
            self.sources,
        )
    }
}

impl DiagnosticList {
    /// Creates an empty diagnostic list with the given source map.
    pub fn new(sources: SourceMap) -> Self {
        Self {
            sources,
            diagnostics_data: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.diagnostics_data.is_empty()
    }

    pub fn len(&self) -> usize {
        self.diagnostics_data.len()
    }

    pub fn iter(
        &self,
    ) -> impl DoubleEndedIterator<Item = Diagnostic<'_, DiagnosticData>> + ExactSizeIterator {
        self.diagnostics_data
            .iter()
            .map(|data| data.to_diagnostic(&self.sources))
    }

    pub(crate) fn push(&mut self, location: Option<SourceSpan>, details: impl Into<Details>) {
        self.diagnostics_data.push(DiagnosticData {
            location,
            details: details.into(),
        })
    }

    /// Concatenate an `other` list of diagnostics into `self`, and sort them together.
    pub fn merge(&mut self, other: Self) {
        if !Arc::ptr_eq(&self.sources, &other.sources) {
            let sources = Arc::make_mut(&mut self.sources);
            for (&k, v) in &*other.sources {
                sources.entry(k).or_insert_with(|| v.clone());
            }
        }
        self.diagnostics_data.extend(other.diagnostics_data);
        self.sort()
    }

    fn sort(&mut self) {
        self.diagnostics_data
            .sort_by_key(|err| err.location.map(|loc| (loc.file_id(), loc.offset())));
    }

    pub(crate) fn into_result(mut self) -> Result<(), Self> {
        if self.diagnostics_data.is_empty() {
            Ok(())
        } else {
            self.sort();
            Err(self)
        }
    }

    pub(crate) fn into_result_with<T>(self, value: T) -> Result<T, WithErrors<T>> {
        match self.into_result() {
            Ok(()) => Ok(value),
            Err(errors) => Err(WithErrors {
                partial: value,
                errors,
            }),
        }
    }

    pub(crate) fn into_valid_result<T>(self, value: T) -> Result<Valid<T>, WithErrors<T>> {
        match self.into_result() {
            Ok(()) => Ok(Valid(value)),
            Err(errors) => Err(WithErrors {
                partial: value,
                errors,
            }),
        }
    }
}

/// Use Display formatting to output without colors: `format!("{diagnostics}")`
impl fmt::Display for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.iter() {
            fmt::Display::fmt(&diagnostic, f)?
        }
        Ok(())
    }
}

/// Use Debug formatting to output with colors: `format!("{diagnostics:?}")`
impl fmt::Debug for DiagnosticList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in self.iter() {
            fmt::Debug::fmt(&diagnostic, f)?
        }
        Ok(())
    }
}

impl From<SchemaBuildError> for Details {
    fn from(value: SchemaBuildError) -> Self {
        Details::SchemaBuildError(value)
    }
}

impl From<ExecutableBuildError> for Details {
    fn from(value: ExecutableBuildError) -> Self {
        Details::ExecutableBuildError(value)
    }
}

impl From<diagnostics::DiagnosticData> for Details {
    fn from(value: diagnostics::DiagnosticData) -> Self {
        Details::CompilerDiagnostic(value)
    }
}

const DEFAULT_RECURSION_LIMIT: usize = 32;

#[derive(Debug, Clone, thiserror::Error)]
#[error("Recursion limit reached")]
#[non_exhaustive]
struct RecursionLimitError {}

/// Track used names in a recursive function.
#[derive(Debug)]
struct RecursionStack {
    seen: IndexSet<Name>,
    high: usize,
    limit: usize,
}

impl RecursionStack {
    fn new() -> Self {
        Self {
            seen: IndexSet::with_hasher(Default::default()),
            high: 0,
            limit: DEFAULT_RECURSION_LIMIT,
        }
    }

    fn with_root(root: Name) -> Self {
        let mut stack = Self::new();
        stack.seen.insert(root);
        stack
    }

    fn with_limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Return the actual API for tracking recursive uses.
    pub(crate) fn guard(&mut self) -> RecursionGuard<'_> {
        RecursionGuard(self)
    }
}

/// Track used names in a recursive function.
///
/// Pass the result of `guard.push(name)` to recursive calls. Use `guard.contains(name)` to check
/// if the name was used somewhere up the call stack. When a guard is dropped, its name is removed
/// from the list.
struct RecursionGuard<'a>(&'a mut RecursionStack);

impl RecursionGuard<'_> {
    /// Mark that we saw a name. If there are too many names, return an error.
    fn push(&mut self, name: &Name) -> Result<RecursionGuard<'_>, RecursionLimitError> {
        let new = self.0.seen.insert(name.clone());
        debug_assert!(
            new,
            "cannot push the same name twice to RecursionGuard, check contains() first"
        );
        self.0.high = self.0.high.max(self.0.seen.len());
        if self.0.seen.len() > self.0.limit {
            Err(RecursionLimitError {})
        } else {
            Ok(RecursionGuard(self.0))
        }
    }

    /// Check if we saw a name somewhere up the call stack.
    fn contains(&self, name: &Name) -> bool {
        self.0.seen.contains(name)
    }

    /// Return the name where we started.
    fn first(&self) -> Option<&Name> {
        self.0.seen.first()
    }
}

impl Drop for RecursionGuard<'_> {
    fn drop(&mut self) {
        // This may already be empty if it's the original `stack.guard()` result, but that's fine
        let _ = self.0.seen.pop();
    }
}

/// Errors that can happen when chasing potentially cyclical references.
#[derive(Debug, Clone, thiserror::Error)]
enum CycleError<T> {
    /// Detected a cycle, value contains the path from the offending node back to the node where we
    /// started.
    #[error("Cycle detected")]
    Recursed(Vec<Node<T>>),
    /// Ran into recursion limit before a cycle could be detected.
    #[error(transparent)]
    Limit(#[from] RecursionLimitError),
}

impl<T> CycleError<T> {
    fn trace(mut self, node: &Node<T>) -> Self {
        if let Self::Recursed(trace) = &mut self {
            trace.push(node.clone());
        }
        self
    }
}

struct CommaSeparated<'a, It>(&'a It);
impl<'a, T, It> fmt::Display for CommaSeparated<'a, It>
where
    T: fmt::Display,
    &'a It: IntoIterator<Item = T>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut it = self.0.into_iter();
        if let Some(element) = it.next() {
            element.fmt(f)?;
        }
        for element in it {
            f.write_str(", ")?;
            element.fmt(f)?;
        }
        Ok(())
    }
}
