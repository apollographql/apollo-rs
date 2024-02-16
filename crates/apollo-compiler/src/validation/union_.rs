use crate::{
    ast,
    schema::{self, ExtendedType, UnionType},
    validation::diagnostics::{DiagnosticData, ValidationError},
    Node,
};

pub(crate) fn validate_union_definitions(schema: &crate::Schema) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for ty in schema.types.values() {
        if let ExtendedType::Union(def) = ty {
            diagnostics.extend(validate_union_definition(schema, def));
        }
    }

    diagnostics
}

pub(crate) fn validate_union_definition(
    schema: &crate::Schema,
    union_def: &Node<UnionType>,
) -> Vec<ValidationError> {
    let mut diagnostics = super::directive::validate_directives(
        Some(schema),
        union_def.directives.iter_ast(),
        ast::DirectiveLocation::Union,
        // unions don't use variables
        Default::default(),
    );

    for union_member in &union_def.members {
        let member_location = union_member.location();
        // TODO: (?) A Union type must include one or more unique member types.

        match schema.types.get(&union_member.name) {
            None => {
                // Union member must be defined.
                diagnostics.push(ValidationError::new(
                    member_location,
                    DiagnosticData::UndefinedDefinition {
                        name: union_member.name.clone(),
                    },
                ));
            }
            Some(schema::ExtendedType::Object(_)) => {} // good
            Some(ty) => {
                // Union member must be of object type.
                diagnostics.push(ValidationError::new(
                    member_location,
                    DiagnosticData::UnionMemberObjectType {
                        name: union_member.name.clone(),
                        describe_type: ty.describe(),
                    },
                ));
            }
        }
    }

    diagnostics
}
