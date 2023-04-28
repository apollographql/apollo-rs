use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{self, TypeDefinition, UnionMember},
    ValidationDatabase,
};

pub fn validate_union_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().unions;
    for def in defs.values() {
        diagnostics.extend(db.validate_union_definition(def.clone()));
    }

    diagnostics
}

fn iter_with_extensions<'a, Item, Ext>(
    base: &'a [Item],
    extensions: &'a [Arc<Ext>],
    method: impl Fn(&'a Ext) -> &'a [Item],
) -> impl Iterator<Item = &'a Item> {
    base.iter()
        .chain(extensions.iter().flat_map(move |ext| method(ext).iter()))
}

pub fn validate_union_definition(
    db: &dyn ValidationDatabase,
    union_def: Arc<hir::UnionTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        union_def.directives().cloned().collect(),
        hir::DirectiveLocation::Union,
    );

    let union_members = iter_with_extensions(
        union_def.self_members(),
        union_def.extensions(),
        hir::UnionTypeExtension::members,
    );
    let mut seen: HashMap<&str, &UnionMember> = HashMap::new();
    for union_member in union_members {
        let name = union_member.name();
        let redefined_definition = union_member.loc();
        // A Union type must include one or more unique member types.
        //
        // Return a Unique Definition error in case of a duplicate member.
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

    diagnostics
}
