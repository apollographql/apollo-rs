use crate::{
    ast, schema,
    validation::diagnostics::{DiagnosticData, ValidationError},
    ValidationDatabase,
};

pub(crate) fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ValidationError> {
    let mut diagnostics = Vec::new();

    for def in db.ast_types().unions.values() {
        diagnostics.extend(db.validate_union_definition(def.clone()));
    }

    diagnostics
}

pub(crate) fn validate_union_definition(
    db: &dyn ValidationDatabase,
    union_def: ast::TypeWithExtensions<ast::UnionTypeDefinition>,
) -> Vec<ValidationError> {
    let mut diagnostics = super::directive::validate_directives(
        db,
        union_def.directives(),
        ast::DirectiveLocation::Union,
        // unions don't use variables
        Default::default(),
    );

    let schema = db.schema();

    for union_member in union_def.members() {
        let member_location = union_member.location();
        // TODO: (?) A Union type must include one or more unique member types.

        match schema.types.get(union_member) {
            None => {
                // Union member must be defined.
                diagnostics.push(ValidationError::new(
                    member_location,
                    DiagnosticData::UndefinedDefinition {
                        name: union_member.clone(),
                    },
                ));
            }
            Some(schema::ExtendedType::Object(_)) => {} // good
            Some(ty) => {
                // Union member must be of object type.
                diagnostics.push(ValidationError::new(
                    member_location,
                    DiagnosticData::UnionMemberObjectType {
                        name: union_member.clone(),
                        describe_type: ty.describe(),
                    },
                ));
            }
        }
    }

    diagnostics
}
