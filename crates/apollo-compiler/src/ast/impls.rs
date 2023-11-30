use super::*;
use crate::name;
use crate::node::NodeLocation;
use crate::schema::ComponentName;
use crate::schema::ComponentOrigin;
use crate::schema::SchemaBuilder;
use crate::validation::DiagnosticList;
use crate::validation::Valid;
use crate::validation::WithErrors;
use crate::ExecutableDocument;
use crate::Parser;
use crate::Schema;
use std::fmt;
use std::hash;
use std::path::Path;

impl Document {
    /// Create an empty document
    pub fn new() -> Self {
        Self {
            sources: Default::default(),
            definitions: Vec::new(),
        }
    }

    /// Return a new configurable parser
    pub fn parser() -> Parser {
        Parser::default()
    }

    /// Parse `input` with the default configuration
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    pub fn parse(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, WithErrors<Self>> {
        Self::parser().parse_ast(source_text, path)
    }

    /// Validate as an executable document, as much as possible without a schema
    pub fn validate_standalone_executable(&self) -> Result<(), DiagnosticList> {
        let mut errors = DiagnosticList::new(self.sources.clone());
        let type_system_definitions_are_errors = true;
        let executable = crate::executable::from_ast::document_from_ast(
            None,
            self,
            &mut errors,
            type_system_definitions_are_errors,
        );
        crate::executable::validation::validate_standalone_executable(&mut errors, &executable);
        errors.into_result()
    }

    /// Build a schema with this AST document as its sole input.
    pub fn to_schema(&self) -> Result<Schema, WithErrors<Schema>> {
        let mut builder = Schema::builder();
        let executable_definitions_are_errors = true;
        builder.add_ast_document(self, executable_definitions_are_errors);
        builder.build()
    }

    /// Build and validate a schema with this AST document as its sole input.
    pub fn to_schema_validate(&self) -> Result<Valid<Schema>, WithErrors<Schema>> {
        let mut builder = Schema::builder();
        let executable_definitions_are_errors = true;
        builder.add_ast_document(self, executable_definitions_are_errors);
        let (schema, mut errors) = builder.build_inner();
        crate::schema::validation::validate_schema(&mut errors, &schema);
        errors.into_valid_result(schema)
    }

    /// Build an executable document from this AST, with the given schema
    pub fn to_executable(
        &self,
        schema: &Valid<Schema>,
    ) -> Result<ExecutableDocument, WithErrors<ExecutableDocument>> {
        let mut errors = DiagnosticList::new(self.sources.clone());
        let document = self.to_executable_inner(schema, &mut errors);
        errors.into_result_with(document)
    }

    /// Build and validate an executable document from this AST, with the given schema
    pub fn to_executable_validate(
        &self,
        schema: &Valid<Schema>,
    ) -> Result<Valid<ExecutableDocument>, WithErrors<ExecutableDocument>> {
        let mut errors = DiagnosticList::new(self.sources.clone());
        let document = self.to_executable_inner(schema, &mut errors);
        crate::executable::validation::validate_executable_document(&mut errors, schema, &document);
        errors.into_valid_result(document)
    }

    pub(crate) fn to_executable_inner(
        &self,
        schema: &Valid<Schema>,
        errors: &mut DiagnosticList,
    ) -> ExecutableDocument {
        let type_system_definitions_are_errors = true;
        crate::executable::from_ast::document_from_ast(
            Some(schema),
            self,
            errors,
            type_system_definitions_are_errors,
        )
    }

    /// Build a schema and executable document from this AST containing a mixture
    /// of type system definitions and executable definitions, and validate them.
    /// This is mostly useful for unit tests.
    pub fn to_mixed_validate(
        &self,
    ) -> Result<(Valid<Schema>, Valid<ExecutableDocument>), DiagnosticList> {
        let mut builder = SchemaBuilder::new();
        let executable_definitions_are_errors = false;
        let type_system_definitions_are_errors = false;
        builder.add_ast_document(self, executable_definitions_are_errors);
        let (schema, mut errors) = builder.build_inner();
        let executable = crate::executable::from_ast::document_from_ast(
            Some(&schema),
            self,
            &mut errors,
            type_system_definitions_are_errors,
        );
        crate::schema::validation::validate_schema(&mut errors, &schema);
        crate::executable::validation::validate_executable_document(
            &mut errors,
            &schema,
            &executable,
        );
        errors
            .into_result()
            .map(|()| (Valid(schema), Valid(executable)))
    }

    serialize_method!();
}

/// `source` is ignored for comparison
impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.definitions == other.definitions
    }
}

impl Eq for Document {}

impl hash::Hash for Document {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.definitions.hash(state);
    }
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Document {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Skip two not-useful indentation levels
        for def in &self.definitions {
            def.fmt(f)?;
            f.write_str("\n")?;
        }
        Ok(())
    }
}

impl Definition {
    /// Returns true if this is an executable definition (operation or fragment).
    pub fn is_executable_definition(&self) -> bool {
        matches!(
            self,
            Self::OperationDefinition(_) | Self::FragmentDefinition(_)
        )
    }

    /// Returns true if this is an extension of another definition.
    pub fn is_extension_definition(&self) -> bool {
        matches!(
            self,
            Self::SchemaExtension(_)
                | Self::ScalarTypeExtension(_)
                | Self::ObjectTypeExtension(_)
                | Self::InterfaceTypeExtension(_)
                | Self::UnionTypeExtension(_)
                | Self::EnumTypeExtension(_)
                | Self::InputObjectTypeExtension(_)
        )
    }

    pub(crate) fn describe(&self) -> &'static str {
        match self {
            Self::OperationDefinition(_) => "an operation definition",
            Self::FragmentDefinition(_) => "a fragment definition",
            Self::DirectiveDefinition(_) => "a directive definition",
            Self::ScalarTypeDefinition(_) => "a scalar type definition",
            Self::ObjectTypeDefinition(_) => "an object type definition",
            Self::InterfaceTypeDefinition(_) => "an interface type definition",
            Self::UnionTypeDefinition(_) => "a union type definition",
            Self::EnumTypeDefinition(_) => "an enum type definition",
            Self::InputObjectTypeDefinition(_) => "an input object type definition",
            Self::SchemaDefinition(_) => "a schema definition",
            Self::SchemaExtension(_) => "a schema extension",
            Self::ScalarTypeExtension(_) => "a scalar type extension",
            Self::ObjectTypeExtension(_) => "an object type extension",
            Self::InterfaceTypeExtension(_) => "an interface type extension",
            Self::UnionTypeExtension(_) => "a union type extension",
            Self::EnumTypeExtension(_) => "an enum type extension",
            Self::InputObjectTypeExtension(_) => "an input object type extension",
        }
    }

    pub fn location(&self) -> Option<NodeLocation> {
        match self {
            Self::OperationDefinition(def) => def.location(),
            Self::FragmentDefinition(def) => def.location(),
            Self::DirectiveDefinition(def) => def.location(),
            Self::SchemaDefinition(def) => def.location(),
            Self::ScalarTypeDefinition(def) => def.location(),
            Self::ObjectTypeDefinition(def) => def.location(),
            Self::InterfaceTypeDefinition(def) => def.location(),
            Self::UnionTypeDefinition(def) => def.location(),
            Self::EnumTypeDefinition(def) => def.location(),
            Self::InputObjectTypeDefinition(def) => def.location(),
            Self::SchemaExtension(def) => def.location(),
            Self::ScalarTypeExtension(def) => def.location(),
            Self::ObjectTypeExtension(def) => def.location(),
            Self::InterfaceTypeExtension(def) => def.location(),
            Self::UnionTypeExtension(def) => def.location(),
            Self::EnumTypeExtension(def) => def.location(),
            Self::InputObjectTypeExtension(def) => def.location(),
        }
    }

    /// Return the name of this type definition or extension.
    ///
    /// Operations may be anonymous, and schema definitions never have a name, in that case this function returns `None`.
    pub fn name(&self) -> Option<&Name> {
        match self {
            Self::OperationDefinition(def) => def.name.as_ref(),
            Self::FragmentDefinition(def) => Some(&def.name),
            Self::DirectiveDefinition(def) => Some(&def.name),
            Self::SchemaDefinition(_) => None,
            Self::ScalarTypeDefinition(def) => Some(&def.name),
            Self::ObjectTypeDefinition(def) => Some(&def.name),
            Self::InterfaceTypeDefinition(def) => Some(&def.name),
            Self::UnionTypeDefinition(def) => Some(&def.name),
            Self::EnumTypeDefinition(def) => Some(&def.name),
            Self::InputObjectTypeDefinition(def) => Some(&def.name),
            Self::SchemaExtension(_) => None,
            Self::ScalarTypeExtension(def) => Some(&def.name),
            Self::ObjectTypeExtension(def) => Some(&def.name),
            Self::InterfaceTypeExtension(def) => Some(&def.name),
            Self::UnionTypeExtension(def) => Some(&def.name),
            Self::EnumTypeExtension(def) => Some(&def.name),
            Self::InputObjectTypeExtension(def) => Some(&def.name),
        }
    }

    pub fn directives(&self) -> &DirectiveList {
        static EMPTY: DirectiveList = DirectiveList(Vec::new());
        match self {
            Self::DirectiveDefinition(_) => &EMPTY,
            Self::OperationDefinition(def) => &def.directives,
            Self::FragmentDefinition(def) => &def.directives,
            Self::SchemaDefinition(def) => &def.directives,
            Self::ScalarTypeDefinition(def) => &def.directives,
            Self::ObjectTypeDefinition(def) => &def.directives,
            Self::InterfaceTypeDefinition(def) => &def.directives,
            Self::UnionTypeDefinition(def) => &def.directives,
            Self::EnumTypeDefinition(def) => &def.directives,
            Self::InputObjectTypeDefinition(def) => &def.directives,
            Self::SchemaExtension(def) => &def.directives,
            Self::ScalarTypeExtension(def) => &def.directives,
            Self::ObjectTypeExtension(def) => &def.directives,
            Self::InterfaceTypeExtension(def) => &def.directives,
            Self::UnionTypeExtension(def) => &def.directives,
            Self::EnumTypeExtension(def) => &def.directives,
            Self::InputObjectTypeExtension(def) => &def.directives,
        }
    }

    pub fn as_operation_definition(&self) -> Option<&Node<OperationDefinition>> {
        if let Self::OperationDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_fragment_definition(&self) -> Option<&Node<FragmentDefinition>> {
        if let Self::FragmentDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_directive_definition(&self) -> Option<&Node<DirectiveDefinition>> {
        if let Self::DirectiveDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_schema_definition(&self) -> Option<&Node<SchemaDefinition>> {
        if let Self::SchemaDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_scalar_type_definition(&self) -> Option<&Node<ScalarTypeDefinition>> {
        if let Self::ScalarTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_object_type_definition(&self) -> Option<&Node<ObjectTypeDefinition>> {
        if let Self::ObjectTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_interface_type_definition(&self) -> Option<&Node<InterfaceTypeDefinition>> {
        if let Self::InterfaceTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_union_type_definition(&self) -> Option<&Node<UnionTypeDefinition>> {
        if let Self::UnionTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_enum_type_definition(&self) -> Option<&Node<EnumTypeDefinition>> {
        if let Self::EnumTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_input_object_type_definition(&self) -> Option<&Node<InputObjectTypeDefinition>> {
        if let Self::InputObjectTypeDefinition(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_schema_extension(&self) -> Option<&Node<SchemaExtension>> {
        if let Self::SchemaExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_scalar_type_extension(&self) -> Option<&Node<ScalarTypeExtension>> {
        if let Self::ScalarTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_object_type_extension(&self) -> Option<&Node<ObjectTypeExtension>> {
        if let Self::ObjectTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_interface_type_extension(&self) -> Option<&Node<InterfaceTypeExtension>> {
        if let Self::InterfaceTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_union_type_extension(&self) -> Option<&Node<UnionTypeExtension>> {
        if let Self::UnionTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_enum_type_extension(&self) -> Option<&Node<EnumTypeExtension>> {
        if let Self::EnumTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    pub fn as_input_object_type_extension(&self) -> Option<&Node<InputObjectTypeExtension>> {
        if let Self::InputObjectTypeExtension(def) = self {
            Some(def)
        } else {
            None
        }
    }

    serialize_method!();
}

impl fmt::Debug for Definition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Skip the enum variant name as it’s redundant with the struct name in it
        match self {
            Self::OperationDefinition(def) => def.fmt(f),
            Self::FragmentDefinition(def) => def.fmt(f),
            Self::DirectiveDefinition(def) => def.fmt(f),
            Self::SchemaDefinition(def) => def.fmt(f),
            Self::ScalarTypeDefinition(def) => def.fmt(f),
            Self::ObjectTypeDefinition(def) => def.fmt(f),
            Self::InterfaceTypeDefinition(def) => def.fmt(f),
            Self::UnionTypeDefinition(def) => def.fmt(f),
            Self::EnumTypeDefinition(def) => def.fmt(f),
            Self::InputObjectTypeDefinition(def) => def.fmt(f),
            Self::SchemaExtension(def) => def.fmt(f),
            Self::ScalarTypeExtension(def) => def.fmt(f),
            Self::ObjectTypeExtension(def) => def.fmt(f),
            Self::InterfaceTypeExtension(def) => def.fmt(f),
            Self::UnionTypeExtension(def) => def.fmt(f),
            Self::EnumTypeExtension(def) => def.fmt(f),
            Self::InputObjectTypeExtension(def) => def.fmt(f),
        }
    }
}

impl OperationDefinition {
    serialize_method!();
}

impl FragmentDefinition {
    serialize_method!();
}

impl DirectiveDefinition {
    serialize_method!();
}

impl SchemaDefinition {
    serialize_method!();
}
impl Extensible for SchemaDefinition {
    type Extension = SchemaExtension;
}

impl ScalarTypeDefinition {
    serialize_method!();
}
impl Extensible for ScalarTypeDefinition {
    type Extension = ScalarTypeExtension;
}

impl ObjectTypeDefinition {
    serialize_method!();
}
impl Extensible for ObjectTypeDefinition {
    type Extension = ObjectTypeExtension;
}

impl InterfaceTypeDefinition {
    serialize_method!();
}
impl Extensible for InterfaceTypeDefinition {
    type Extension = InterfaceTypeExtension;
}

impl UnionTypeDefinition {
    serialize_method!();
}
impl Extensible for UnionTypeDefinition {
    type Extension = UnionTypeExtension;
}

impl EnumTypeDefinition {
    serialize_method!();
}
impl Extensible for EnumTypeDefinition {
    type Extension = EnumTypeExtension;
}

impl InputObjectTypeDefinition {
    serialize_method!();
}
impl Extensible for InputObjectTypeDefinition {
    type Extension = InputObjectTypeExtension;
}

impl SchemaExtension {
    serialize_method!();
}

impl ScalarTypeExtension {
    serialize_method!();
}

impl ObjectTypeExtension {
    serialize_method!();
}

impl InterfaceTypeExtension {
    serialize_method!();
}

impl UnionTypeExtension {
    serialize_method!();
}

impl EnumTypeExtension {
    serialize_method!();
}

impl InputObjectTypeExtension {
    serialize_method!();
}

impl DirectiveList {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns an iterator of directives with the given name.
    ///
    /// This method is best for repeatable directives.
    /// See also [`get`][Self::get] for non-repeatable directives.
    pub fn get_all<'def: 'name, 'name>(
        &'def self,
        name: &'name str,
    ) -> impl Iterator<Item = &'def Node<Directive>> + 'name {
        self.0.iter().filter(move |dir| dir.name == name)
    }

    /// Returns the first directive with the given name, if any.
    ///
    /// This method is best for non-repeatable directives.
    /// See also [`get_all`][Self::get_all] for repeatable directives.
    pub fn get(&self, name: &str) -> Option<&Node<Directive>> {
        self.get_all(name).next()
    }

    /// Returns whether there is a directive with the given name
    pub fn has(&self, name: &str) -> bool {
        self.get(name).is_some()
    }

    serialize_method!();
}

impl std::fmt::Debug for DirectiveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::ops::Deref for DirectiveList {
    type Target = Vec<Node<Directive>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DirectiveList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> IntoIterator for &'a DirectiveList {
    type Item = &'a Node<Directive>;

    type IntoIter = std::slice::Iter<'a, Node<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl<'a> IntoIterator for &'a mut DirectiveList {
    type Item = &'a mut Node<Directive>;

    type IntoIter = std::slice::IterMut<'a, Node<Directive>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}

impl FromIterator<Node<Directive>> for DirectiveList {
    fn from_iter<T: IntoIterator<Item = Node<Directive>>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl FromIterator<Directive> for DirectiveList {
    fn from_iter<T: IntoIterator<Item = Directive>>(iter: T) -> Self {
        Self(iter.into_iter().map(Node::new).collect())
    }
}

impl Directive {
    pub fn argument_by_name(&self, name: &str) -> Option<&Node<Value>> {
        self.arguments
            .iter()
            .find_map(|arg| (arg.name == name).then_some(&arg.value))
    }

    serialize_method!();
}

impl OperationType {
    /// Get the name of this operation type as it would appear in GraphQL source code.
    pub fn name(self) -> &'static str {
        match self {
            OperationType::Query => "query",
            OperationType::Mutation => "mutation",
            OperationType::Subscription => "subscription",
        }
    }

    /// Get the default name of the object type for this operation type
    pub const fn default_type_name(self) -> NamedType {
        match self {
            OperationType::Query => name!("Query"),
            OperationType::Mutation => name!("Mutation"),
            OperationType::Subscription => name!("Subscription"),
        }
    }

    serialize_method!();
}

impl DirectiveLocation {
    /// Get the name of this directive location as it would appear in GraphQL source code.
    pub fn name(self) -> &'static str {
        match self {
            DirectiveLocation::Query => "QUERY",
            DirectiveLocation::Mutation => "MUTATION",
            DirectiveLocation::Subscription => "SUBSCRIPTION",
            DirectiveLocation::Field => "FIELD",
            DirectiveLocation::FragmentDefinition => "FRAGMENT_DEFINITION",
            DirectiveLocation::FragmentSpread => "FRAGMENT_SPREAD",
            DirectiveLocation::InlineFragment => "INLINE_FRAGMENT",
            DirectiveLocation::VariableDefinition => "VARIABLE_DEFINITION",
            DirectiveLocation::Schema => "SCHEMA",
            DirectiveLocation::Scalar => "SCALAR",
            DirectiveLocation::Object => "OBJECT",
            DirectiveLocation::FieldDefinition => "FIELD_DEFINITION",
            DirectiveLocation::ArgumentDefinition => "ARGUMENT_DEFINITION",
            DirectiveLocation::Interface => "INTERFACE",
            DirectiveLocation::Union => "UNION",
            DirectiveLocation::Enum => "ENUM",
            DirectiveLocation::EnumValue => "ENUM_VALUE",
            DirectiveLocation::InputObject => "INPUT_OBJECT",
            DirectiveLocation::InputFieldDefinition => "INPUT_FIELD_DEFINITION",
        }
    }
}

impl fmt::Debug for DirectiveLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name().fmt(f)
    }
}

impl From<OperationType> for DirectiveLocation {
    fn from(ty: OperationType) -> Self {
        match ty {
            OperationType::Query => DirectiveLocation::Query,
            OperationType::Mutation => DirectiveLocation::Mutation,
            OperationType::Subscription => DirectiveLocation::Subscription,
        }
    }
}

impl VariableDefinition {
    serialize_method!();
}

/// Create a static [`Type`] with GraphQL-like syntax
///
/// ```
/// use apollo_compiler::ty;
///
/// assert_eq!(ty!(Obj).to_string(), "Obj");
/// assert_eq!(ty!(Obj!).to_string(), "Obj!");
/// assert_eq!(ty!([Obj]).to_string(), "[Obj]");
/// assert_eq!(ty!([Obj]!).to_string(), "[Obj]!");
/// assert_eq!(ty!([[[Obj ] !]]!).to_string(), "[[[Obj]!]]!");
/// ```
#[macro_export]
macro_rules! ty {
    ($name: ident) => {
        $crate::ast::Type::Named($crate::name!($name))
    };
    ($name: ident !) => {
        $crate::ast::Type::NonNullNamed($crate::name!($name))
    };
    ([ $($tt: tt)+ ]) => {
        $crate::ast::Type::List(::std::boxed::Box::new($crate::ty!( $($tt)+ )))
    };
    ([ $($tt: tt)+ ]!) => {
        $crate::ast::Type::NonNullList(::std::boxed::Box::new($crate::ty!( $($tt)+ )))
    };
}

impl Type {
    /// Returns this type made non-null, if it isn’t already.
    pub fn non_null(self) -> Self {
        match self {
            Type::Named(name) => Type::NonNullNamed(name),
            Type::List(inner) => Type::NonNullList(inner),
            Type::NonNullNamed(_) => self,
            Type::NonNullList(_) => self,
        }
    }

    /// Returns this type made nullable, if it isn’t already.
    pub fn nullable(self) -> Self {
        match self {
            Type::Named(_) => self,
            Type::List(_) => self,
            Type::NonNullNamed(name) => Type::Named(name),
            Type::NonNullList(inner) => Type::List(inner),
        }
    }

    /// Returns a list type whose items are this type.
    pub fn list(self) -> Self {
        Type::List(Box::new(self))
    }

    /// If the type is a list type or a non-null list type, return the item type.
    ///
    /// # Example
    /// ```
    /// use apollo_compiler::ty;
    /// // Returns the inner type of the list.
    /// assert_eq!(ty!([Foo!]).item_type(), &ty!(Foo!));
    /// // Not a list: returns the input.
    /// assert_eq!(ty!(Foo!).item_type(), &ty!(Foo!));
    /// ```
    pub fn item_type(&self) -> &Self {
        match self {
            Type::List(inner) | Type::NonNullList(inner) => inner,
            ty => ty,
        }
    }

    /// Returns the inner named type, after unwrapping any non-null or list markers.
    pub fn inner_named_type(&self) -> &NamedType {
        match self {
            Type::Named(name) | Type::NonNullNamed(name) => name,
            Type::List(inner) | Type::NonNullList(inner) => inner.inner_named_type(),
        }
    }

    /// Returns whether this type is non-null
    pub fn is_non_null(&self) -> bool {
        matches!(self, Type::NonNullNamed(_) | Type::NonNullList(_))
    }

    /// Returns whether this type is a list, on a non-null list
    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_) | Type::NonNullList(_))
    }

    pub fn is_named(&self) -> bool {
        matches!(self, Type::Named(_) | Type::NonNullNamed(_))
    }

    /// Can a value of this type be used when the `target` type is expected?
    ///
    /// Implementation of spec function `AreTypesCompatible()`.
    pub fn is_assignable_to(&self, target: &Self) -> bool {
        match (target, self) {
            // Can't assign a nullable type to a non-nullable type.
            (Type::NonNullNamed(_) | Type::NonNullList(_), Type::Named(_) | Type::List(_)) => false,
            // Can't assign a list type to a non-list type.
            (Type::Named(_) | Type::NonNullNamed(_), Type::List(_) | Type::NonNullList(_)) => false,
            // Can't assign a non-list type to a list type.
            (Type::List(_) | Type::NonNullList(_), Type::Named(_) | Type::NonNullNamed(_)) => false,
            // Non-null named types can be assigned if they are the same.
            (Type::NonNullNamed(left), Type::NonNullNamed(right)) => left == right,
            // Non-null list types can be assigned if their inner types are compatible.
            (Type::NonNullList(left), Type::NonNullList(right)) => right.is_assignable_to(left),
            // Both nullable and non-nullable named types can be assigned to a nullable type of the
            // same name.
            (Type::Named(left), Type::Named(right) | Type::NonNullNamed(right)) => left == right,
            // Nullable and non-nullable lists can be assigned to a matching nullable list type.
            (Type::List(left), Type::List(right) | Type::NonNullList(right)) => {
                right.is_assignable_to(left)
            }
        }
    }

    /// Parse the given source text as a reference to a type.
    ///
    /// `path` is the filesystem path (or arbitrary string) used in diagnostics
    /// to identify this source file to users.
    ///
    /// Create a [`Parser`] to use different parser configuration.
    pub fn parse(
        source_text: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, DiagnosticList> {
        Parser::new().parse_type(source_text, path)
    }

    serialize_method!();
}

impl FieldDefinition {
    serialize_method!();
}

impl InputValueDefinition {
    pub fn is_required(&self) -> bool {
        matches!(*self.ty, Type::NonNullNamed(_) | Type::NonNullList(_))
    }

    serialize_method!();
}

impl EnumValueDefinition {
    serialize_method!();
}

impl Selection {
    pub fn location(&self) -> Option<NodeLocation> {
        match self {
            Self::Field(field) => field.location(),
            Self::FragmentSpread(fragment) => fragment.location(),
            Self::InlineFragment(fragment) => fragment.location(),
        }
    }

    serialize_method!();
}

impl Field {
    /// Get the name that will be used for this field selection in response formatting.
    ///
    /// For example, in this operation, the response name is "sourceField":
    /// ```graphql
    /// query GetField { sourceField }
    /// ```
    ///
    /// But in this operation that uses an alias, the response name is "responseField":
    /// ```graphql
    /// query GetField { responseField: sourceField }
    /// ```
    pub fn response_name(&self) -> &Name {
        self.alias.as_ref().unwrap_or(&self.name)
    }

    serialize_method!();
}

impl FragmentSpread {
    serialize_method!();
}

impl InlineFragment {
    serialize_method!();
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn as_enum(&self) -> Option<&Name> {
        if let Value::Enum(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn as_variable(&self) -> Option<&Name> {
        if let Value::Variable(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_node_str(&self) -> Option<&NodeStr> {
        if let Value::String(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn to_f64(&self) -> Option<f64> {
        match self {
            Value::Float(value) => value.try_to_f64().ok(),
            Value::Int(value) => value.try_to_f64().ok(),
            _ => None,
        }
    }

    pub fn to_i32(&self) -> Option<i32> {
        if let Value::Int(value) = self {
            value.try_to_i32().ok()
        } else {
            None
        }
    }

    pub fn to_bool(&self) -> Option<bool> {
        if let Value::Boolean(value) = *self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_list(&self) -> Option<&[Node<Value>]> {
        if let Value::List(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn as_object(&self) -> Option<&[(Name, Node<Value>)]> {
        if let Value::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Value::Null => "Null",
            Value::Enum(_) => "Enum",
            Value::Variable(_) => "Variable",
            Value::String(_) => "String",
            Value::Float(_) => "Float",
            Value::Int(_) => "Int",
            Value::Boolean(_) => "Boolean",
            Value::List(_) => "List",
            Value::Object(_) => "Object",
        }
    }

    serialize_method!();
}

impl IntValue {
    /// Constructs from a string matching the [`IntValue`
    /// grammar specification](https://spec.graphql.org/October2021/#IntValue)
    ///
    /// To convert an `i32`, use `from` or `into` instead.
    pub fn new_parsed(text: &str) -> Self {
        debug_assert!(IntValue::valid_syntax(text), "{text:?}");
        Self(text.into())
    }

    fn valid_syntax(text: &str) -> bool {
        match text.strip_prefix('-').unwrap_or(text).as_bytes() {
            [b'0'..=b'9'] => true,
            [b'1'..=b'9', rest @ ..] => rest.iter().all(|b| b.is_ascii_digit()),
            _ => false,
        }
    }

    /// Returns the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts to `i32`, returning an error on overflow
    ///
    /// Note: parsing is expected to succeed with a correctly-constructed `IntValue`,
    /// leaving overflow as the only error case.
    pub fn try_to_i32(&self) -> Result<i32, std::num::ParseIntError> {
        self.0.parse()
    }

    /// Converts to a finite `f64`, returning an error on overflow to infinity
    ///
    /// An `IntValue` signals integer syntax was used, but is also valid in contexts
    /// where a `Float` is expected.
    ///
    /// Note: parsing is expected to succeed with a correctly-constructed `IntValue`,
    /// leaving overflow as the only error case.
    pub fn try_to_f64(&self) -> Result<f64, FloatOverflowError> {
        try_to_f64(&self.0)
    }
}

impl FloatValue {
    /// Constructs from a string matching the [`FloatValue`
    /// grammar specification](https://spec.graphql.org/October2021/#IntValue)
    ///
    /// To convert an `f64`, use `from` or `into` instead.
    pub fn new_parsed(text: &str) -> Self {
        debug_assert!(FloatValue::valid_syntax(text), "{text:?}");
        Self(text.into())
    }

    fn valid_syntax(text: &str) -> bool {
        if let Some((mantissa, exponent)) = text.split_once(['e', 'E']) {
            let exponent = exponent.strip_prefix(['+', '-']).unwrap_or(exponent);
            if !exponent.bytes().all(|b| b.is_ascii_digit()) {
                return false;
            }
            if let Some((int, fract)) = mantissa.split_once('.') {
                Self::valid_fractional_syntax(int, fract)
            } else {
                IntValue::valid_syntax(mantissa)
            }
        } else {
            text.split_once('.')
                .is_some_and(|(int, fract)| Self::valid_fractional_syntax(int, fract))
        }
    }

    fn valid_fractional_syntax(integer: &str, fractional: &str) -> bool {
        IntValue::valid_syntax(integer)
            && !fractional.is_empty()
            && fractional.bytes().all(|b| b.is_ascii_digit())
    }

    /// Returns the string representation
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Converts to a finite `f64`, returning an error on overflow to infinity
    ///
    /// Note: parsing is expected to succeed with a correctly-constructed `FloatValue`,
    /// leaving overflow as the only error case.
    pub fn try_to_f64(&self) -> Result<f64, FloatOverflowError> {
        try_to_f64(&self.0)
    }
}

fn try_to_f64(text: &str) -> Result<f64, FloatOverflowError> {
    let parsed = text.parse::<f64>();
    debug_assert!(parsed.is_ok(), "{}", parsed.unwrap_err());
    let Ok(float) = parsed else {
        return Err(FloatOverflowError {});
    };
    debug_assert!(!float.is_nan());
    if float.is_finite() {
        Ok(float)
    } else {
        Err(FloatOverflowError {})
    }
}

impl From<i32> for IntValue {
    fn from(value: i32) -> Self {
        let text = value.to_string();
        debug_assert!(IntValue::valid_syntax(&text), "{text:?}");
        Self(text)
    }
}

impl From<f64> for FloatValue {
    fn from(value: f64) -> Self {
        let mut text = value.to_string();
        if !text.contains('.') {
            text.push_str(".0")
        }
        debug_assert!(FloatValue::valid_syntax(&text), "{text:?}");
        Self(text)
    }
}

impl fmt::Display for IntValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for IntValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl fmt::Debug for FloatValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl<'de> serde::Deserialize<'de> for IntValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const EXPECTING: &str = "a string in GraphQL IntValue syntax";
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = IntValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(EXPECTING)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if IntValue::valid_syntax(v) {
                    Ok(IntValue(v.to_owned()))
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(v), &EXPECTING))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if IntValue::valid_syntax(&v) {
                    Ok(IntValue(v))
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(&v), &EXPECTING))
                }
            }
        }
        deserializer.deserialize_string(Visitor)
    }
}

impl<'de> serde::Deserialize<'de> for FloatValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const EXPECTING: &str = "a string in GraphQL FloatValue syntax";
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FloatValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(EXPECTING)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if FloatValue::valid_syntax(v) {
                    Ok(FloatValue(v.to_owned()))
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(v), &EXPECTING))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if FloatValue::valid_syntax(&v) {
                    Ok(FloatValue(v))
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(&v), &EXPECTING))
                }
            }
        }
        deserializer.deserialize_string(Visitor)
    }
}

impl serde::Serialize for IntValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl serde::Serialize for FloatValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl fmt::Display for FloatOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("value magnitude too large to be converted to `f64`")
    }
}

impl fmt::Debug for FloatOverflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl From<Node<OperationDefinition>> for Definition {
    fn from(def: Node<OperationDefinition>) -> Self {
        Self::OperationDefinition(def)
    }
}

impl From<Node<FragmentDefinition>> for Definition {
    fn from(def: Node<FragmentDefinition>) -> Self {
        Self::FragmentDefinition(def)
    }
}

impl From<Node<DirectiveDefinition>> for Definition {
    fn from(def: Node<DirectiveDefinition>) -> Self {
        Self::DirectiveDefinition(def)
    }
}

impl From<Node<SchemaDefinition>> for Definition {
    fn from(def: Node<SchemaDefinition>) -> Self {
        Self::SchemaDefinition(def)
    }
}

impl From<Node<ScalarTypeDefinition>> for Definition {
    fn from(def: Node<ScalarTypeDefinition>) -> Self {
        Self::ScalarTypeDefinition(def)
    }
}

impl From<Node<ObjectTypeDefinition>> for Definition {
    fn from(def: Node<ObjectTypeDefinition>) -> Self {
        Self::ObjectTypeDefinition(def)
    }
}

impl From<Node<InterfaceTypeDefinition>> for Definition {
    fn from(def: Node<InterfaceTypeDefinition>) -> Self {
        Self::InterfaceTypeDefinition(def)
    }
}

impl From<Node<UnionTypeDefinition>> for Definition {
    fn from(def: Node<UnionTypeDefinition>) -> Self {
        Self::UnionTypeDefinition(def)
    }
}

impl From<Node<EnumTypeDefinition>> for Definition {
    fn from(def: Node<EnumTypeDefinition>) -> Self {
        Self::EnumTypeDefinition(def)
    }
}

impl From<Node<InputObjectTypeDefinition>> for Definition {
    fn from(def: Node<InputObjectTypeDefinition>) -> Self {
        Self::InputObjectTypeDefinition(def)
    }
}

impl From<Node<SchemaExtension>> for Definition {
    fn from(def: Node<SchemaExtension>) -> Self {
        Self::SchemaExtension(def)
    }
}

impl From<Node<ScalarTypeExtension>> for Definition {
    fn from(def: Node<ScalarTypeExtension>) -> Self {
        Self::ScalarTypeExtension(def)
    }
}

impl From<Node<ObjectTypeExtension>> for Definition {
    fn from(def: Node<ObjectTypeExtension>) -> Self {
        Self::ObjectTypeExtension(def)
    }
}

impl From<Node<InterfaceTypeExtension>> for Definition {
    fn from(def: Node<InterfaceTypeExtension>) -> Self {
        Self::InterfaceTypeExtension(def)
    }
}

impl From<Node<UnionTypeExtension>> for Definition {
    fn from(def: Node<UnionTypeExtension>) -> Self {
        Self::UnionTypeExtension(def)
    }
}

impl From<Node<EnumTypeExtension>> for Definition {
    fn from(def: Node<EnumTypeExtension>) -> Self {
        Self::EnumTypeExtension(def)
    }
}

impl From<Node<InputObjectTypeExtension>> for Definition {
    fn from(def: Node<InputObjectTypeExtension>) -> Self {
        Self::InputObjectTypeExtension(def)
    }
}

impl From<OperationDefinition> for Definition {
    fn from(def: OperationDefinition) -> Self {
        Self::OperationDefinition(Node::new(def))
    }
}

impl From<FragmentDefinition> for Definition {
    fn from(def: FragmentDefinition) -> Self {
        Self::FragmentDefinition(Node::new(def))
    }
}

impl From<DirectiveDefinition> for Definition {
    fn from(def: DirectiveDefinition) -> Self {
        Self::DirectiveDefinition(Node::new(def))
    }
}

impl From<SchemaDefinition> for Definition {
    fn from(def: SchemaDefinition) -> Self {
        Self::SchemaDefinition(Node::new(def))
    }
}

impl From<ScalarTypeDefinition> for Definition {
    fn from(def: ScalarTypeDefinition) -> Self {
        Self::ScalarTypeDefinition(Node::new(def))
    }
}

impl From<ObjectTypeDefinition> for Definition {
    fn from(def: ObjectTypeDefinition) -> Self {
        Self::ObjectTypeDefinition(Node::new(def))
    }
}

impl From<InterfaceTypeDefinition> for Definition {
    fn from(def: InterfaceTypeDefinition) -> Self {
        Self::InterfaceTypeDefinition(Node::new(def))
    }
}

impl From<UnionTypeDefinition> for Definition {
    fn from(def: UnionTypeDefinition) -> Self {
        Self::UnionTypeDefinition(Node::new(def))
    }
}

impl From<EnumTypeDefinition> for Definition {
    fn from(def: EnumTypeDefinition) -> Self {
        Self::EnumTypeDefinition(Node::new(def))
    }
}

impl From<InputObjectTypeDefinition> for Definition {
    fn from(def: InputObjectTypeDefinition) -> Self {
        Self::InputObjectTypeDefinition(Node::new(def))
    }
}

impl From<SchemaExtension> for Definition {
    fn from(def: SchemaExtension) -> Self {
        Self::SchemaExtension(Node::new(def))
    }
}

impl From<ScalarTypeExtension> for Definition {
    fn from(def: ScalarTypeExtension) -> Self {
        Self::ScalarTypeExtension(Node::new(def))
    }
}

impl From<ObjectTypeExtension> for Definition {
    fn from(def: ObjectTypeExtension) -> Self {
        Self::ObjectTypeExtension(Node::new(def))
    }
}

impl From<InterfaceTypeExtension> for Definition {
    fn from(def: InterfaceTypeExtension) -> Self {
        Self::InterfaceTypeExtension(Node::new(def))
    }
}

impl From<UnionTypeExtension> for Definition {
    fn from(def: UnionTypeExtension) -> Self {
        Self::UnionTypeExtension(Node::new(def))
    }
}

impl From<EnumTypeExtension> for Definition {
    fn from(def: EnumTypeExtension) -> Self {
        Self::EnumTypeExtension(Node::new(def))
    }
}

impl From<InputObjectTypeExtension> for Definition {
    fn from(def: InputObjectTypeExtension) -> Self {
        Self::InputObjectTypeExtension(Node::new(def))
    }
}

impl From<()> for Value {
    fn from(_value: ()) -> Self {
        Value::Null
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Value::Float(value.into())
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::Int(value.into())
    }
}

impl From<&'_ str> for Value {
    fn from(value: &'_ str) -> Self {
        Value::String(value.into())
    }
}

impl From<&'_ String> for Value {
    fn from(value: &'_ String) -> Self {
        Value::String(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<()> for Node<Value> {
    fn from(value: ()) -> Self {
        Node::new(value.into())
    }
}

impl From<f64> for Node<Value> {
    fn from(value: f64) -> Self {
        Node::new(value.into())
    }
}

impl From<i32> for Node<Value> {
    fn from(value: i32) -> Self {
        Node::new(value.into())
    }
}

impl From<&'_ str> for Node<Value> {
    fn from(value: &'_ str) -> Self {
        Node::new(value.into())
    }
}

impl From<&'_ String> for Node<Value> {
    fn from(value: &'_ String) -> Self {
        Node::new(value.into())
    }
}

impl From<String> for Node<Value> {
    fn from(value: String) -> Self {
        Node::new(value.into())
    }
}

impl From<bool> for Node<Value> {
    fn from(value: bool) -> Self {
        Node::new(value.into())
    }
}

impl<N: Into<Name>, V: Into<Node<Value>>> From<(N, V)> for Node<Argument> {
    fn from((name, value): (N, V)) -> Self {
        Node::new(Argument {
            name: name.into(),
            value: value.into(),
        })
    }
}

/// Create a [`Name`] from a string literal or identifier, checked for validity at compile time.
///
/// A `Name` created this way does not own allocated heap memory or a reference counter,
/// so cloning it is extremely cheap.
///
/// # Examples
///
/// ```
/// use apollo_compiler::name;
///
/// assert_eq!(name!("Query").as_str(), "Query");
/// assert_eq!(name!(Query).as_str(), "Query");
/// ```
///
/// ```compile_fail
/// # use apollo_compiler::name;
/// // error[E0080]: evaluation of constant value failed
/// // assertion failed: ::apollo_compiler::ast::Name::valid_syntax(\"è_é\")
/// let invalid = name!("è_é");
/// ```
#[macro_export]
macro_rules! name {
    ($value: ident) => {
        $crate::name!(stringify!($value))
    };
    ($value: expr) => {{
        const _: () = { assert!($crate::ast::Name::valid_syntax($value)) };
        $crate::ast::Name::new_unchecked($crate::NodeStr::from_static(&$value))
    }};
}

impl Name {
    /// Creates a new `Name` if the given value is a valid GraphQL name.
    pub fn new(value: impl Into<NodeStr>) -> Result<Self, InvalidNameError> {
        let value = value.into();
        if Self::valid_syntax(&value) {
            Ok(Self::new_unchecked(value))
        } else {
            Err(InvalidNameError(value))
        }
    }

    /// Creates a new `Name` without validity checking.
    ///
    /// Constructing an invalid name may cause invalid document serialization
    /// but not memory-safety issues.
    pub const fn new_unchecked(value: NodeStr) -> Self {
        Self(value)
    }

    /// Returns whether the given string is a valid GraphQL name.
    ///
    /// <https://spec.graphql.org/October2021/#Name>
    pub const fn valid_syntax(value: &str) -> bool {
        let bytes = value.as_bytes();
        let Some(&first) = bytes.first() else {
            return false;
        };
        // `NameStart` and `NameContinue` are within the ASCII range,
        // so converting from `u8` to `char` here does not affect the result:
        if !Self::char_is_name_start(first as char) {
            return false;
        }
        // TODO: iterator when available in const
        let mut i = 1;
        while i < bytes.len() {
            if !Self::char_is_name_continue(bytes[i] as char) {
                return false;
            }
            i += 1
        }
        true
    }

    /// <https://spec.graphql.org/October2021/#NameStart>
    const fn char_is_name_start(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_')
    }

    /// <https://spec.graphql.org/October2021/#NameContinue>
    const fn char_is_name_continue(c: char) -> bool {
        matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9')
    }

    pub fn to_component(&self, origin: ComponentOrigin) -> ComponentName {
        ComponentName {
            origin,
            name: self.clone(),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const EXPECTING: &str = "a string in GraphQL Name syntax";
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = Name;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(EXPECTING)
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                if Name::valid_syntax(v) {
                    Ok(Name(v.into()))
                } else {
                    Err(E::invalid_value(serde::de::Unexpected::Str(v), &EXPECTING))
                }
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

impl serde::Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl TryFrom<NodeStr> for Name {
    type Error = InvalidNameError;

    fn try_from(value: NodeStr) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&'_ NodeStr> for Name {
    type Error = InvalidNameError;

    fn try_from(value: &'_ NodeStr) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Name {
    type Error = InvalidNameError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Name {
    type Error = InvalidNameError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&'_ String> for Name {
    type Error = InvalidNameError;

    fn try_from(value: &'_ String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl std::ops::Deref for Name {
    type Target = NodeStr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl AsRef<NodeStr> for Name {
    fn as_ref(&self) -> &NodeStr {
        &self.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for Name {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<str> for Name {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&'_ str> for Name {
    fn eq(&self, other: &&'_ str) -> bool {
        self.as_str() == *other
    }
}

impl PartialOrd<str> for Name {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}

impl PartialOrd<&'_ str> for Name {
    fn partial_cmp(&self, other: &&'_ str) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(*other)
    }
}

impl fmt::Debug for InvalidNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<T: Extensible> TypeWithExtensions<T> {
    /// Iterate over elements of the base definition and its extensions.
    pub fn iter_all<'a, Item, DefIter, ExtIter>(
        &'a self,
        mut map_definition: impl FnMut(&'a Node<T>) -> DefIter + 'a,
        map_extension: impl FnMut(&'a Node<T::Extension>) -> ExtIter + 'a,
    ) -> impl Iterator<Item = Item> + 'a
    where
        Item: 'a,
        DefIter: Iterator<Item = Item> + 'a,
        ExtIter: Iterator<Item = Item> + 'a,
    {
        map_definition(&self.definition).chain(self.extensions.iter().flat_map(map_extension))
    }
}

macro_rules! iter_extensible_method {
    ($property:ident, $ty:ty) => {
        pub fn $property(&self) -> impl Iterator<Item = &'_ $ty> + '_ {
            self.iter_all(
                |definition| definition.$property.iter(),
                |extension| extension.$property.iter(),
            )
        }
    };
}

impl TypeWithExtensions<SchemaDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(root_operations, Node<(OperationType, NamedType)>);
}

impl TypeWithExtensions<ObjectTypeDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(fields, Node<FieldDefinition>);
    iter_extensible_method!(implements_interfaces, NamedType);
}

impl TypeWithExtensions<InterfaceTypeDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(fields, Node<FieldDefinition>);
    iter_extensible_method!(implements_interfaces, NamedType);
}

impl TypeWithExtensions<UnionTypeDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(members, NamedType);
}

impl TypeWithExtensions<EnumTypeDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(values, Node<EnumValueDefinition>);
}

impl TypeWithExtensions<InputObjectTypeDefinition> {
    iter_extensible_method!(directives, Node<Directive>);
    iter_extensible_method!(fields, Node<InputValueDefinition>);
}
