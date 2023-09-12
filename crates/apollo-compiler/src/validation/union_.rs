use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema, ValidationDatabase,
};
use std::collections::HashSet;

pub fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for def in db.ast_types().unions.values() {
        diagnostics.extend(db.validate_union_definition(def.clone()));
    }

    diagnostics
}

pub fn validate_union_definition(
    db: &dyn ValidationDatabase,
    union_def: ast::TypeWithExtensions<ast::UnionTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives2(
        db,
        union_def.directives(),
        ast::DirectiveLocation::Union,
        // unions don't use variables
        Default::default(),
    );

    let schema = db.schema();

    let mut seen: HashSet<ast::Name> = HashSet::new();
    for union_member in union_def.members() {
        let member_location = *union_member.location().unwrap();
        // A Union type must include one or more unique member types.
        //
        // Return a Unique Definition error in case of a duplicate member.
        if let Some(prev_def) = seen.get(union_member) {
            let original_definition = *prev_def.location().unwrap();

            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    member_location.into(),
                    DiagnosticData::UniqueDefinition {
                        ty: "union member",
                        name: union_member.to_string(),
                        original_definition: original_definition.into(),
                        redefined_definition: member_location.into(),
                    },
                )
                .labels([
                    Label::new(
                        original_definition,
                        format!("previous definition of `{union_member}` here"),
                    ),
                    Label::new(member_location, format!("`{union_member}` redefined here")),
                ])
                .help(format!(
                    "`{union_member}` must only be defined once in this document."
                )),
            );
        } else {
            seen.insert(union_member.clone());
        }

        match schema.types.get(union_member) {
            None => {
                // Union member must be defined.
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        member_location.into(),
                        DiagnosticData::UndefinedDefinition {
                            name: union_member.to_string(),
                        },
                    )
                    .label(Label::new(member_location, "not found in this scope")),
                );
            }
            Some(schema::ExtendedType::Object(_)) => {} // good
            Some(ty) => {
                // Union member must be of object type.
                let (particle, kind) = match ty {
                    schema::ExtendedType::Object(_) => unreachable!(),
                    schema::ExtendedType::Scalar(_) => ("a", "scalar"),
                    schema::ExtendedType::Interface(_) => ("an", "interface"),
                    schema::ExtendedType::Union(_) => ("an", "union"),
                    schema::ExtendedType::Enum(_) => ("an", "enum"),
                    schema::ExtendedType::InputObject(_) => ("an", "input object"),
                };
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        member_location.into(),
                        DiagnosticData::ObjectType {
                            name: union_member.to_string(),
                            ty: kind,
                        },
                    )
                    .label(Label::new(
                        member_location,
                        format!("This is a {particle} {kind}"),
                    ))
                    .help("Union members must be of base Object Type."),
                );
            }
        }
    }

    diagnostics
}
