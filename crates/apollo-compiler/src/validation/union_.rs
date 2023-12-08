use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema, ValidationDatabase,
};

pub(crate) fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for def in db.ast_types().unions.values() {
        diagnostics.extend(db.validate_union_definition(def.clone()));
    }

    diagnostics
}

pub(crate) fn validate_union_definition(
    db: &dyn ValidationDatabase,
    union_def: ast::TypeWithExtensions<ast::UnionTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
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
                diagnostics.push(ApolloDiagnostic::new(
                    db,
                    member_location,
                    DiagnosticData::UndefinedDefinition {
                        name: union_member.to_string(),
                    },
                ));
            }
            Some(schema::ExtendedType::Object(_)) => {} // good
            Some(ty) => {
                // Union member must be of object type.
                let kind = match ty {
                    schema::ExtendedType::Object(_) => unreachable!(),
                    schema::ExtendedType::Scalar(_) => "scalar",
                    schema::ExtendedType::Interface(_) => "interface",
                    schema::ExtendedType::Union(_) => "union",
                    schema::ExtendedType::Enum(_) => "enum",
                    schema::ExtendedType::InputObject(_) => "input object",
                };
                diagnostics.push(ApolloDiagnostic::new(
                    db,
                    member_location,
                    DiagnosticData::UnionMemberObjectType {
                        name: union_member.to_string(),
                        ty: kind,
                    },
                ));
            }
        }
    }

    diagnostics
}
