use std::collections::HashMap;

use crate::{
    diagnostics::{ObjectType, UndefinedDefinition, UniqueDefinition},
    hir::{TypeDefinition, UnionMember},
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for union_def in db.unions().values() {
        let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
        for union_member in union_def.union_members().iter() {
            let name = union_member.name();
            let offset = union_member.loc().offset();
            let len = union_member.loc().node_len();
            // A Union type must include one or more unique member types.
            //
            // Return a Unique Value error in case of a duplicate member.
            if let Some(prev_def) = seen.get(&name) {
                let prev_offset = prev_def.loc().offset();
                let prev_node_len = prev_def.loc().node_len();

                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    name: name.into(),
                    ty: "union member".into(),
                    src: db.source_code(prev_def.loc().file_id()),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (offset, len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
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
