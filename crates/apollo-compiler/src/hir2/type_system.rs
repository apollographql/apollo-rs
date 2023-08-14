use apollo_parser::mir;
use apollo_parser::mir::Harc;
use apollo_parser::mir::Name;
use apollo_parser::mir::Ranged;
use apollo_parser::BowString;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::sync::OnceLock;

/// Results of analysis of type system definitions from any number of input files.
///
/// Information about a given type can come either from its “main” definition
/// or from an extension.
#[derive(Clone, Debug)]
pub struct TypeSystem {
    pub schema: Schema,
    pub directives: IndexMap<Name, Harc<Ranged<mir::DirectiveDefinition>>>,
    pub types: IndexMap<mir::NamedType, Type>,
}

#[derive(Clone, Debug)]
pub struct Schema {
    pub description: Option<BowString>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    /// Name of the object type for the `query` root operation
    pub query: Option<mir::NamedType>,
    /// Name of the object type for the `mutation` root operation
    pub mutation: Option<mir::NamedType>,
    /// Name of the object type for the `subscription` root operation
    pub subscription: Option<mir::NamedType>,
}

/// The definition of a named type, with all information from type extensions folded in
#[derive(Clone, Debug)]
pub enum Type {
    Scalar(ScalarType),
    Object(ObjectType),
    Interface(InterfaceType),
    Union(UnionType),
    Enum(EnumType),
    InputObject(InputObjectType),
}

#[derive(Clone, Debug)]
pub struct ScalarType {
    pub description: Option<BowString>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
}

#[derive(Clone, Debug)]
pub struct ObjectType {
    pub description: Option<BowString>,
    pub implements_interfaces: IndexSet<Name>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub fields: IndexMap<Name, Harc<Ranged<mir::FieldDefinition>>>,
}

#[derive(Clone, Debug)]
pub struct InterfaceType {
    pub description: Option<BowString>,
    pub implements_interfaces: IndexSet<Name>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub fields: IndexMap<Name, Harc<Ranged<mir::FieldDefinition>>>,
}

#[derive(Clone, Debug)]
pub struct UnionType {
    pub description: Option<BowString>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub members: IndexSet<mir::NamedType>,
}

#[derive(Clone, Debug)]
pub struct EnumType {
    pub description: Option<BowString>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub values: IndexMap<Name, Harc<Ranged<mir::EnumValueDefinition>>>,
}

#[derive(Clone, Debug)]
pub struct InputObjectType {
    pub description: Option<BowString>,
    pub directives: Vec<Harc<Ranged<mir::Directive>>>,
    pub fields: IndexMap<Name, Harc<Ranged<mir::InputValueDefinition>>>,
}

impl TypeSystem {
    pub fn new(input_files: &[mir::Document]) -> Self {
        static BUILT_IN_TYPES: std::sync::OnceLock<mir::Document> = std::sync::OnceLock::new();
        let built_in = BUILT_IN_TYPES.get_or_init(|| {
            let ast = apollo_parser::Parser::new(include_str!("../built_in_types.graphql")).parse();
            debug_assert_eq!(ast.errors().as_slice(), []);
            ast.into_mir()
        });
        let documents = std::iter::once(built_in).chain(input_files);
        let mut opt_schema = None;
        let mut directives = IndexMap::new();
        let mut types = IndexMap::new();
        // Clone the iterator so we can later iterate again from the start
        for document in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaDefinition(def) => {
                        opt_schema.get_or_insert_with(|| Schema::new(def));
                    }
                    mir::Definition::DirectiveDefinition(def) => {
                        directives.entry(def.name.clone()).or_insert(def.clone());
                    }
                    mir::Definition::ScalarTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Scalar(ScalarType::new(def)));
                    }
                    mir::Definition::ObjectTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Object(ObjectType::new(def)));
                    }
                    mir::Definition::InterfaceTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Interface(InterfaceType::new(def)));
                    }
                    mir::Definition::UnionTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Union(UnionType::new(def)));
                    }
                    mir::Definition::EnumTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::Enum(EnumType::new(def)));
                    }
                    mir::Definition::InputObjectTypeDefinition(def) => {
                        types
                            .entry(def.name.clone())
                            .or_insert(Type::InputObject(InputObjectType::new(def)));
                    }
                    mir::Definition::SchemaExtension(_)
                    | mir::Definition::ScalarTypeExtension(_)
                    | mir::Definition::ObjectTypeExtension(_)
                    | mir::Definition::InterfaceTypeExtension(_)
                    | mir::Definition::UnionTypeExtension(_)
                    | mir::Definition::EnumTypeExtension(_)
                    | mir::Definition::InputObjectTypeExtension(_)
                    | mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_) => todo!(),
                }
            }
        }
        for document in documents.clone() {
            for definition in &document.definitions {
                match definition {
                    mir::Definition::SchemaExtension(ext) => {
                        if let Some(schema) = &mut opt_schema {
                            schema.extend(ext);
                        }
                    }
                    mir::Definition::ScalarTypeExtension(ext) => {
                        if let Some(Type::Scalar(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::ObjectTypeExtension(ext) => {
                        if let Some(Type::Object(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::InterfaceTypeExtension(ext) => {
                        if let Some(Type::Interface(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::UnionTypeExtension(ext) => {
                        if let Some(Type::Union(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::EnumTypeExtension(ext) => {
                        if let Some(Type::Enum(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::InputObjectTypeExtension(ext) => {
                        if let Some(Type::InputObject(ty)) = types.get_mut(&ext.name) {
                            ty.extend(ext)
                        }
                    }
                    mir::Definition::OperationDefinition(_)
                    | mir::Definition::FragmentDefinition(_)
                    | mir::Definition::DirectiveDefinition(_)
                    | mir::Definition::SchemaDefinition(_)
                    | mir::Definition::ScalarTypeDefinition(_)
                    | mir::Definition::ObjectTypeDefinition(_)
                    | mir::Definition::InterfaceTypeDefinition(_)
                    | mir::Definition::UnionTypeDefinition(_)
                    | mir::Definition::EnumTypeDefinition(_)
                    | mir::Definition::InputObjectTypeDefinition(_) => todo!(),
                }
            }
        }
        Self {
            schema: opt_schema.unwrap_or_else(|| Schema::implicit(&types)),
            directives,
            types,
        }
    }

    /// Returns the type with the given name, if it is a scalar type
    pub fn get_scalar(&self, name: &str) -> Option<&ScalarType> {
        if let Some(Type::Scalar(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a object type
    pub fn get_object(&self, name: &str) -> Option<&ObjectType> {
        if let Some(Type::Object(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a interface type
    pub fn get_interface(&self, name: &str) -> Option<&InterfaceType> {
        if let Some(Type::Interface(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a union type
    pub fn get_union(&self, name: &str) -> Option<&UnionType> {
        if let Some(Type::Union(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a enum type
    pub fn get_enum(&self, name: &str) -> Option<&EnumType> {
        if let Some(Type::Enum(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// Returns the type with the given name, if it is a input object type
    pub fn get_input_object(&self, name: &str) -> Option<&InputObjectType> {
        if let Some(Type::InputObject(ty)) = self.types.get(name) {
            Some(ty)
        } else {
            None
        }
    }

    /// If the given name is an object type on interface type, return its field definitions
    pub(crate) fn field_definitions(
        &self,
        name: &str,
    ) -> Option<&IndexMap<Name, Harc<Ranged<mir::FieldDefinition>>>> {
        match self.types.get(name)? {
            Type::Object(ty) => Some(&ty.fields),
            Type::Interface(ty) => Some(&ty.fields),
            _ => None,
        }
    }

    /// Return the meta-fields for a selection set.
    ///
    /// `is_root_operation` must be `Some` if and only if the selection set is the root of an operation.
    pub(crate) fn meta_field_definitions(
        is_root_operation_type: Option<mir::OperationType>,
    ) -> &'static [Harc<Ranged<mir::FieldDefinition>>] {
        static TYPENAME_FIELD: OnceLock<Harc<Ranged<mir::FieldDefinition>>> = OnceLock::new();
        static ROOT_QUERY_FIELDS: OnceLock<[Harc<Ranged<mir::FieldDefinition>>; 3]> =
            OnceLock::new();
        let typename_field = || {
            TYPENAME_FIELD.get_or_init(|| {
                // __typename: String!
                Harc::new(Ranged::no_location(mir::FieldDefinition {
                    description: None,
                    name: "__typename".into(),
                    arguments: Vec::new(),
                    ty: mir::Type::new_named("String").non_null(),
                    directives: Vec::new(),
                }))
            })
        };
        match is_root_operation_type {
            Some(mir::OperationType::Query) => ROOT_QUERY_FIELDS.get_or_init(|| {
                [
                    typename_field().clone(),
                    // __schema: __Schema!
                    Harc::new(Ranged::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__schema".into(),
                        arguments: Vec::new(),
                        ty: mir::Type::new_named("__Schema").non_null(),
                        directives: Vec::new(),
                    })),
                    // __type(name: String!): __Type
                    Harc::new(Ranged::no_location(mir::FieldDefinition {
                        description: None,
                        name: "__type".into(),
                        arguments: vec![Harc::new(Ranged::no_location(
                            mir::InputValueDefinition {
                                description: None,
                                name: "name".into(),
                                ty: mir::Type::new_named("String").non_null(),
                                default_value: None,
                                directives: Vec::new(),
                            },
                        ))],
                        ty: mir::Type::new_named("__Type"),
                        directives: Vec::new(),
                    })),
                ]
            }),
            Some(mir::OperationType::Subscription) => &[],
            _ => std::slice::from_ref(typename_field()),
        }
    }
}

impl Schema {
    fn new(definition: &mir::SchemaDefinition) -> Schema {
        let mut schema = Schema {
            description: definition.description.clone(),
            directives: definition.directives.clone(),
            query: None,
            mutation: None,
            subscription: None,
        };
        schema.add_root_operations(&definition.root_operations);
        schema
    }

    fn extend(&mut self, extension: &mir::SchemaExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        self.add_root_operations(&extension.root_operations)
    }

    fn add_root_operations(&mut self, root_operations: &[(mir::OperationType, BowString)]) {
        for (operation_type, object_type_name) in root_operations {
            match operation_type {
                mir::OperationType::Query => &mut self.query,
                mir::OperationType::Mutation => &mut self.mutation,
                mir::OperationType::Subscription => &mut self.subscription,
            }
            .get_or_insert_with(|| object_type_name.clone());
        }
    }

    /// Returns the name of the object type for the root operation with the given operation kind
    pub fn root_operation(&self, operation_type: mir::OperationType) -> Option<&BowString> {
        match operation_type {
            mir::OperationType::Query => &self.query,
            mir::OperationType::Mutation => &self.mutation,
            mir::OperationType::Subscription => &self.subscription,
        }
        .as_ref()
    }

    fn implicit(types: &IndexMap<BowString, Type>) -> Self {
        let if_has_object_type = |name: &str| {
            if let Some(Type::Object(_)) = types.get(name) {
                Some(BowString::from(name))
            } else {
                None
            }
        };
        Self {
            description: None,
            directives: Vec::new(),
            query: if_has_object_type("Query"),
            mutation: if_has_object_type("Mutation"),
            subscription: if_has_object_type("Subscription"),
        }
    }
}

impl ScalarType {
    fn new(definition: &mir::ScalarTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            directives: definition.directives.clone(),
        }
    }

    fn extend(&mut self, extension: &mir::ScalarTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned())
    }
}

impl ObjectType {
    fn new(definition: &mir::ObjectTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            implements_interfaces: definition.implements_interfaces.iter().cloned().collect(),
            directives: definition.directives.clone(),
            fields: definition
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect(),
        }
    }

    fn extend(&mut self, extension: &mir::ObjectTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        self.implements_interfaces
            .extend(extension.implements_interfaces.iter().cloned());
        for field in &extension.fields {
            self.fields
                .entry(field.name.clone())
                .or_insert_with(|| field.clone());
        }
    }
}

impl InterfaceType {
    fn new(definition: &mir::InterfaceTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            implements_interfaces: definition.implements_interfaces.iter().cloned().collect(),
            directives: definition.directives.clone(),
            fields: definition
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect(),
        }
    }

    fn extend(&mut self, extension: &mir::InterfaceTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        self.implements_interfaces
            .extend(extension.implements_interfaces.iter().cloned());
        for field in &extension.fields {
            self.fields
                .entry(field.name.clone())
                .or_insert_with(|| field.clone());
        }
    }
}

impl UnionType {
    fn new(definition: &mir::UnionTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            directives: definition.directives.clone(),
            members: definition.members.iter().cloned().collect(),
        }
    }

    fn extend(&mut self, extension: &mir::UnionTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        self.members.extend(extension.members.iter().cloned());
    }
}

impl EnumType {
    fn new(definition: &mir::EnumTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            directives: definition.directives.clone(),
            values: definition
                .values
                .iter()
                .map(|v| (v.value.clone(), v.clone()))
                .collect(),
        }
    }

    fn extend(&mut self, extension: &mir::EnumTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        for value in &extension.values {
            self.values
                .entry(value.value.clone())
                .or_insert_with(|| value.clone());
        }
    }
}

impl InputObjectType {
    fn new(definition: &mir::InputObjectTypeDefinition) -> Self {
        Self {
            description: definition.description.clone(),
            directives: definition.directives.clone(),
            fields: definition
                .fields
                .iter()
                .map(|f| (f.name.clone(), f.clone()))
                .collect(),
        }
    }

    fn extend(&mut self, extension: &mir::InputObjectTypeExtension) {
        self.directives.extend(extension.directives.iter().cloned());
        for field in &extension.fields {
            self.fields
                .entry(field.name.clone())
                .or_insert_with(|| field.clone());
        }
    }
}
