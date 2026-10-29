use apollo_compiler::ast::Definition;
use apollo_compiler::ast::EnumValueDefinition;
use apollo_compiler::ast::FieldDefinition;
use apollo_compiler::ast::InputValueDefinition;
use apollo_compiler::Name;
use apollo_compiler::Node;

pub(crate) trait TypeDefinition {
    fn type_name(&self) -> &Name;
    fn interfaces(&self) -> &[Name];
    fn fields(&self) -> &[Node<FieldDefinition>];
    fn input_fields(&self) -> &[Node<InputValueDefinition>];
    fn enum_values(&self) -> &[Node<EnumValueDefinition>];
    fn members(&self) -> &[Name];
}

impl TypeDefinition for Definition {
    fn type_name(&self) -> &Name {
        self.name().expect("named type definition")
    }

    fn interfaces(&self) -> &[Name] {
        match self {
            Self::ObjectTypeDefinition(d) => &d.implements_interfaces,
            Self::ObjectTypeExtension(d) => &d.implements_interfaces,
            Self::InterfaceTypeDefinition(d) => &d.implements_interfaces,
            Self::InterfaceTypeExtension(d) => &d.implements_interfaces,
            _ => &[],
        }
    }

    fn fields(&self) -> &[Node<FieldDefinition>] {
        match self {
            Self::ObjectTypeDefinition(d) => &d.fields,
            Self::ObjectTypeExtension(d) => &d.fields,
            Self::InterfaceTypeDefinition(d) => &d.fields,
            Self::InterfaceTypeExtension(d) => &d.fields,
            _ => &[],
        }
    }

    fn input_fields(&self) -> &[Node<InputValueDefinition>] {
        match self {
            Self::InputObjectTypeDefinition(d) => &d.fields,
            Self::InputObjectTypeExtension(d) => &d.fields,
            _ => &[],
        }
    }

    fn enum_values(&self) -> &[Node<EnumValueDefinition>] {
        match self {
            Self::EnumTypeDefinition(d) => &d.values,
            Self::EnumTypeExtension(d) => &d.values,
            _ => &[],
        }
    }

    fn members(&self) -> &[Name] {
        match self {
            Self::UnionTypeDefinition(d) => &d.members,
            Self::UnionTypeExtension(d) => &d.members,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TypeDefinition;
    use apollo_compiler::ast::Document;
    use apollo_compiler::name;

    #[test]
    fn accessors_include_extensions() {
        let document = Document::parse(
            "interface I { id: ID } type O implements I { id: ID }
             extend type O { extra: Int } input In { x: Int }
             enum E { A } union U = O scalar S",
            "schema.graphql",
        )
        .unwrap();
        let object = &document.definitions[1];
        assert_eq!(object.interfaces(), &[name!(I)]);
        assert_eq!(object.fields()[0].name, name!(id));
        assert_eq!(document.definitions[2].fields()[0].name, name!(extra));
        assert_eq!(document.definitions[3].input_fields()[0].name, name!(x));
        assert_eq!(document.definitions[4].enum_values()[0].value, name!(A));
        assert_eq!(document.definitions[5].members(), &[name!(O)]);
        assert!(document.definitions[6].fields().is_empty());
    }
}
