use std::collections::HashMap;

use crate::{diagnostics::UniqueDefinition, values::UnionMember, ApolloDiagnostic, SourceDatabase};

pub fn check(db: &dyn SourceDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    // A Union type must include one or more unique member types.
    //
    // Return a Unique Value error in case of a duplicate member.
    for union_def in db.unions().iter() {
        let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
        for union_member in union_def.union_members().iter() {
            let name = union_member.name();
            if let Some(prev_def) = seen.get(&name) {
                let prev_offset: usize = prev_def.ast_node(db).text_range().start().into();
                let prev_node_len: usize = prev_def.ast_node(db).text_range().len().into();

                let current_offset: usize = union_member.ast_node(db).text_range().start().into();
                let current_node_len: usize = union_member.ast_node(db).text_range().len().into();
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                    name: name.into(),
                    ty: "union member".into(),
                    src: db.input_string(()).to_string(),
                    original_definition: (prev_offset, prev_node_len).into(),
                    redefined_definition: (current_offset, current_node_len).into(),
                    help: Some(format!(
                        "`{name}` must only be defined once in this document."
                    )),
                }));
            } else {
                seen.insert(name, union_member);
            }
        }
    }

    diagnostics
}
