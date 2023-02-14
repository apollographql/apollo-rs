use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
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
                .label(Label::new(extension.name_src().loc.unwrap(), format!("a type definition named `{}` does not exist", extension.name()))),
            );

            continue;
        };

        match (definition, extension) {
            (
                hir::TypeDefinition::ScalarTypeDefinition(_def),
                hir::TypeExtension::ScalarTypeExtension(_ext),
            ) => (),
            (
                hir::TypeDefinition::ObjectTypeDefinition(_def),
                hir::TypeExtension::ObjectTypeExtension(_ext),
            ) => (),
            (
                hir::TypeDefinition::InterfaceTypeDefinition(_def),
                hir::TypeExtension::InterfaceTypeExtension(_ext),
            ) => (),
            (
                hir::TypeDefinition::UnionTypeDefinition(_def),
                hir::TypeExtension::UnionTypeExtension(_ext),
            ) => (),
            (
                hir::TypeDefinition::EnumTypeDefinition(_def),
                hir::TypeExtension::EnumTypeExtension(_ext),
            ) => (),
            (
                hir::TypeDefinition::InputObjectTypeDefinition(_def),
                hir::TypeExtension::InputObjectTypeExtension(_ext),
            ) => (),
            (definition, extension) => {
                let mut diagnostic = ApolloDiagnostic::new(
                    db,
                    extension.loc().into(),
                    DiagnosticData::WrongTypeExtension {
                        name: extension.name().into(),
                        definition: definition.loc().map(|loc| loc.into()),
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
                ));
                if let Some(def_loc) = definition.loc() {
                    diagnostic =
                        diagnostic.label(Label::new(def_loc, format!("original type defined here")))
                }
                diagnostics.push(diagnostic);
            }
        }
    }

    diagnostics
}
