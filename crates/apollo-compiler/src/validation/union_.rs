use std::collections::HashMap;

use crate::{
    diagnostics::{Diagnostic2, DiagnosticData, Label, ObjectType, UndefinedDefinition},
    hir::{TypeDefinition, UnionMember},
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for union_def in db.unions().values() {
        let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
        for union_member in union_def.union_members().iter() {
            let name = union_member.name();
            let redefined_definition = union_member.loc();
            let offset = redefined_definition.offset();
            let len = redefined_definition.node_len();

            // A Union type must include one or more unique member types.
            //
            // Return a Unique Value error in case of a duplicate member.
            if let Some(prev_def) = seen.get(&name) {
                let original_definition = prev_def.loc();

                diagnostics.push(ApolloDiagnostic::Diagnostic2(
                    Diagnostic2::new(
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
                ));
            } else {
                seen.insert(name, union_member);
            }

            match db.upcast().find_type_definition_by_name(name.to_string()) {
                None => {
                    // Union member must be defined.
                    diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                        ty: name.into(),
                        src: db.source_code(union_member.loc().file_id()),
                        definition: (offset, len).into(),
                    }))
                }
                Some(TypeDefinition::ObjectTypeDefinition { .. }) => {} // good
                Some(ty) => {
                    // Union member must be of object type.
                    diagnostics.push(ApolloDiagnostic::ObjectType(ObjectType {
                        name: name.into(),
                        ty: ty.kind(),
                        src: db.source_code(union_member.loc().file_id()),
                        definition: (offset, len).into(),
                    }))
                }
            }
        }
    }

    diagnostics
}
