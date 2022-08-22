use std::collections::HashMap;

use crate::{
    diagnostics::{ObjectType, UndefinedDefinition, UniqueDefinition},
    values::UnionMember,
    ApolloDiagnostic, Document,
};

pub fn check(db: &dyn Document) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for union_def in db.unions().iter() {
        let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
        for union_member in union_def.union_members().iter() {
            let name = union_member.name();
            let offset: usize = union_member.ast_node(db).text_range().start().into();
            let len: usize = union_member.ast_node(db).text_range().len().into();
            // A Union type must include one or more unique member types.
            //
            // Return a Unique Value error in case of a duplicate member.
            if let Some(prev_def) = seen.get(&name) {
                let prev_offset: usize = prev_def.ast_node(db).text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db).text_range().len().into();

                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    name: name.into(),
                    ty: "union member".into(),
                    src: db.input().to_string(),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (offset, len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            } else {
                seen.insert(name, union_member);
            }

            // Union member must be defined.
            let union_member_type = db.find_type_system_definition_by_name(name.to_string());
            if union_member_type.is_none() {
                diagnostics.push(ApolloDiagnostic::UndefinedDefinition(UndefinedDefinition {
                    ty: name.into(),
                    src: db.input().to_string(),
                    definition: (offset, len).into(),
                }))
            } else if let Some(ty) = union_member_type {
                // Union member must be of object type.
                if !ty.is_object_type_definition() {
                    diagnostics.push(ApolloDiagnostic::ObjectType(ObjectType {
                        name: name.into(),
                        ty: ty.ty(),
                        src: db.input().to_string(),
                        definition: (offset, len).into(),
                    }))
                }
            }
        }
    }

    diagnostics
}
