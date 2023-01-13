use std::collections::HashMap;

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{TypeDefinition, UnionMember},
    ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for union_def in db.unions().values() {
        let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
        for union_member in union_def.union_members().iter() {
            let name = union_member.name();
            let redefined_definition = union_member.loc();

            // A Union type must include one or more unique member types.
            //
            // Return a Unique Value error in case of a duplicate member.
            if let Some(prev_def) = seen.get(&name) {
                let original_definition = prev_def.loc();

                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        union_member.loc().into(),
                        DiagnosticData::UniqueDefinition {
                            ty: "union member",
                            name: name.into(),
                            original_definition: original_definition.into(),
                            redefined_definition: redefined_definition.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_definition, format!("`{name}` redefined here")),
                    ])
                    .help(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                );
            } else {
                seen.insert(name, union_member);
            }

            match db.upcast().find_type_definition_by_name(name.to_string()) {
                None => {
                    // Union member must be defined.
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            union_member.loc().into(),
                            DiagnosticData::UndefinedDefinition { name: name.into() },
                        )
                        .label(Label::new(union_member.loc(), "not found in this scope")),
                    );
                }
                Some(TypeDefinition::ObjectTypeDefinition { .. }) => {} // good
                Some(ty) => {
                    // Union member must be of object type.
                    let kind = ty.kind();
                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            union_member.loc().into(),
                            DiagnosticData::ObjectType {
                                name: name.into(),
                                ty: kind,
                            },
                        )
                        .label(Label::new(
                            union_member.loc(),
                            format!("This is of `{kind}` type"),
                        ))
                        .help("Union members must be of base Object Type."),
                    );
                }
            }
        }
    }

    diagnostics
}
