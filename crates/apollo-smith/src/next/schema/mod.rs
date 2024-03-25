use std::ops::Deref;
use std::sync::OnceLock;
use apollo_compiler::ast::{FieldDefinition, Name};
use apollo_compiler::schema::{Component, ExtendedType, InterfaceType, ObjectType, UnionType};
use indexmap::IndexMap;
use apollo_compiler::Schema;
use crate::next::Unstructured;

pub(crate) mod extended_type;

pub(crate) mod object_type;
pub(crate) mod schema;

/// macro for accessing fields on schema elements
macro_rules! field_access {
    ($ty:ty) => {
        paste::paste! {
            pub(crate) trait [<$ty Ext>] {
                fn random_field(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Option<&apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let mut fields = self.target().fields.values().collect::<Vec<_>>();
                    match u.choose_index(fields.len()) {
                        Ok(idx)=> Ok(Some(fields.remove(idx))),
                        Err(arbitrary::Error::EmptyChoose)=> Ok(None),
                        Err(e)=> Err(e)
                    }
                }

                fn random_field_mut(
                    &mut self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Option<&mut apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let mut fields = self.target_mut().fields.values_mut().collect::<Vec<_>>();
                    match u.choose_index(fields.len()) {
                        Ok(idx)=> Ok(Some(fields.remove(idx))),
                        Err(arbitrary::Error::EmptyChoose)=> Ok(None),
                        Err(e)=> Err(e)
                    }

                }

                fn sample_fields(
                    &self,
                    u: &mut crate::next::Unstructured,
                ) -> arbitrary::Result<Vec<&apollo_compiler::schema::Component<apollo_compiler::ast::FieldDefinition>>> {
                    let existing = self
                        .target()
                        .fields
                        .values()
                        .filter(|_| u.arbitrary().unwrap_or(false))
                        .collect::<Vec<_>>();

                    Ok(existing)
                }
                fn target(&self) -> &$ty;
                fn target_mut(&mut self) -> &mut $ty;
            }

            impl [<$ty Ext>] for $ty {
                fn target(&self) -> &$ty {
                    self
                }
                fn target_mut(&mut self) -> &mut $ty {
                    self
                }
            }
        }
    };
}

field_access!(ObjectType);
field_access!(InterfaceType);

pub(crate) trait Selectable {

    fn name(&self) -> &Name;
    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>>;

    fn random_specialization<'a>(&self, u: &mut Unstructured, schema: &'a Schema) -> arbitrary::Result<Option<&'a ExtendedType>>;
    fn random_field(&self, u: &mut Unstructured) -> arbitrary::Result<&Component<FieldDefinition>> {
        // Types always have at least one field
        let fields = self.fields().values().collect::<Vec<_>>();
        Ok(fields[u.choose_index(fields.len())?])
    }

}

impl Selectable for ObjectType {
    fn name(&self) -> &Name {
        &self.name
    }



    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        &self.fields
    }

    fn random_specialization<'a>(&self, _u: &mut Unstructured, _schema: &'a Schema) -> arbitrary::Result<Option<&'a ExtendedType>> {
        Ok(None)
    }
}

impl Selectable for &UnionType {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        static EMPTY: OnceLock<IndexMap<Name, Component<FieldDefinition>>> = OnceLock::new();
        &EMPTY.get_or_init(||Default::default())
    }

    fn random_specialization<'a>(&self, u: &mut Unstructured, schema: &'a Schema) -> arbitrary::Result<Option<&'a ExtendedType>> {
        let members = self.members.iter().map(|name| schema.types.get(&name.name)).collect::<Vec<_>>();
        if members.is_empty() {
            Ok(None)
        }
        else {
            Ok(members[u.choose_index(members.len())?])
        }
    }
}

impl Selectable for InterfaceType {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        &self.fields
    }

    fn random_specialization<'a>(&self, u: &mut Unstructured, schema: &'a Schema) -> arbitrary::Result<Option<&'a ExtendedType>> {
        // An interface specialization is either an object or another interface that implements this interface
        let implements = schema
            .types
            .values()
            .filter(|ty| {
                match ty {
                    ExtendedType::Object(o) => o.implements_interfaces.contains(&self.name),
                    ExtendedType::Interface(i) => i.implements_interfaces.contains(&self.name),
                    _=> return false,
                }
            })
            .collect::<Vec<_>>();
        if implements.is_empty() {
            Ok(None)
        }
        else {
            Ok(Some(implements[u.choose_index(implements.len())?]))
        }


    }
}


impl Selectable for ExtendedType {
    fn name(&self) -> &Name {
        match self {
            ExtendedType::Scalar(scalar) => {&scalar.name}
            ExtendedType::Object(object_type) => {&object_type.name}
            ExtendedType::Interface(interface_type) => {&interface_type.name}
            ExtendedType::Union(union_type) => {&union_type.name}
            ExtendedType::Enum(enum_type) => {&enum_type.name}
            ExtendedType::InputObject(input_object) => {&input_object.name}
        }
    }



    fn fields(&self) -> &IndexMap<Name, Component<FieldDefinition>> {
        static EMPTY: OnceLock<IndexMap<Name, Component<FieldDefinition>>> = OnceLock::new();
        match self {
            ExtendedType::Object(t) => &t.fields,
            ExtendedType::Interface(t) => &t.fields,
            _ => &EMPTY.get_or_init(||Default::default()),
        }
    }

    fn random_specialization<'a>(&self, u: &mut Unstructured, schema: &'a Schema) -> arbitrary::Result<Option<&'a ExtendedType>> {
        match self {
            ExtendedType::Object(object_type) => {object_type.deref().random_specialization(u, schema)}
            ExtendedType::Interface(interface_type) => { interface_type.deref().random_specialization(u, schema)}
            ExtendedType::Union(union_type) => { union_type.deref().random_specialization(u, schema)}
            _ => Ok(None)
        }
    }
}
