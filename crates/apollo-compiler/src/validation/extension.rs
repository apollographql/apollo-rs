use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir::{TypeDefinition, TypeExtension},
    validation::ValidationDatabase,
};

/// Choose "a" or "an" depending on the first letter of `ty`. Only supports capital letters.
fn particle(ty: &str) -> &str {
    match ty.chars().next() {
        Some('A' | 'E' | 'I' | 'O' | 'U') => "an",
        _ => "a",
    }
}

pub fn validate_extensions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for extension in db.extensions().iter() {
        let Some(definition) = db.find_type_definition_by_name(extension.name().into()) else {
            diagnostics.push(
                ApolloDiagnostic::new(db, extension.loc().into(), DiagnosticData::UndefinedDefinition {
                    name: extension.name().into(),
                })
                .label(Label::new(extension.name_src().loc.unwrap(), format!("`{}` type definition does not exist", extension.name()))),
            );

            continue;
        };

        match (definition, extension) {
            (TypeDefinition::ScalarTypeDefinition(_), TypeExtension::ScalarTypeExtension(_))
            | (TypeDefinition::ObjectTypeDefinition(_), TypeExtension::ObjectTypeExtension(_))
            | (
                TypeDefinition::InterfaceTypeDefinition(_),
                TypeExtension::InterfaceTypeExtension(_),
            )
            | (TypeDefinition::UnionTypeDefinition(_), TypeExtension::UnionTypeExtension(_))
            | (TypeDefinition::EnumTypeDefinition(_), TypeExtension::EnumTypeExtension(_))
            | (
                TypeDefinition::InputObjectTypeDefinition(_),
                TypeExtension::InputObjectTypeExtension(_),
            ) => {
                // Definition/extension kinds are the same
            }
            (definition, extension) => {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        extension.loc().into(),
                        DiagnosticData::WrongTypeExtension {
                            name: extension.name().into(),
                            definition: definition.loc().into(),
                            extension: extension.loc().into(),
                        },
                    )
                    .label(Label::new(
                        extension.loc(),
                        format!(
                            "adding {} {}, but `{}` is {} {}",
                            particle(extension.kind()),
                            extension.kind(),
                            extension.name(),
                            particle(definition.kind()),
                            definition.kind()
                        ),
                    ))
                    .label(Label::new(definition.loc(), "original type defined here")),
                );
            }
        }
    }

    diagnostics
}
