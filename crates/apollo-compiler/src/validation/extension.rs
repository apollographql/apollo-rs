use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema::ExtendedType,
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

    let schema = db.schema();
    for file_id in db.type_definition_files() {
        for extension in &db.ast(file_id).definitions {
            type A = ast::Definition;
            let name = match extension {
                A::ObjectTypeExtension(obj) => &obj.name,
                A::ScalarTypeExtension(obj) => &obj.name,
                A::EnumTypeExtension(obj) => &obj.name,
                A::UnionTypeExtension(obj) => &obj.name,
                A::InterfaceTypeExtension(obj) => &obj.name,
                A::InputObjectTypeExtension(obj) => &obj.name,
                // `extend schema` can extend an implicit schema.
                A::SchemaExtension(_) => continue,
                // Non-extensions
                _ => continue,
            };
            let name_location = name.location().unwrap();

            let Some(definition) = schema.types.get(name) else {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        extension.location().unwrap().into(),
                        DiagnosticData::UndefinedDefinition {
                            name: name.to_string(),
                        },
                    )
                    .label(Label::new(
                        name_location,
                        format!("`{name}` type definition does not exist"),
                    )),
                );
                continue;
            };

            match (definition, extension) {
                (ExtendedType::Scalar(_), A::ScalarTypeExtension(_))
                | (ExtendedType::Object(_), A::ObjectTypeExtension(_))
                | (ExtendedType::Interface(_), A::InterfaceTypeExtension(_))
                | (ExtendedType::Union(_), A::UnionTypeExtension(_))
                | (ExtendedType::Enum(_), A::EnumTypeExtension(_))
                | (ExtendedType::InputObject(_), A::InputObjectTypeExtension(_)) => {
                    // Definition/extension kinds are the same
                }
                (definition, extension) => {
                    let definition_location = match definition {
                        ExtendedType::Scalar(def) => def.location(),
                        ExtendedType::Object(def) => def.location(),
                        ExtendedType::Interface(def) => def.location(),
                        ExtendedType::Union(def) => def.location(),
                        ExtendedType::Enum(def) => def.location(),
                        ExtendedType::InputObject(def) => def.location(),
                    }
                    .unwrap();
                    let definition_kind = match definition {
                        ExtendedType::Scalar(_) => "Scalar",
                        ExtendedType::Object(_) => "Object",
                        ExtendedType::Interface(_) => "Interface",
                        ExtendedType::Union(_) => "Union",
                        ExtendedType::Enum(_) => "Enum",
                        ExtendedType::InputObject(_) => "InputObject",
                    };

                    diagnostics.push(
                        ApolloDiagnostic::new(
                            db,
                            extension.location().unwrap().into(),
                            DiagnosticData::WrongTypeExtension {
                                name: name.to_string(),
                                definition: definition_location.into(),
                                extension: extension.location().unwrap().into(),
                            },
                        )
                        .label(Label::new(
                            extension.location().unwrap(),
                            format!(
                                "adding {} {}, but `{}` is {} {}",
                                particle(extension.kind()),
                                extension.kind(),
                                name,
                                particle(definition_kind),
                                definition_kind,
                            ),
                        ))
                        .label(Label::new(
                            definition_location,
                            "original type defined here",
                        )),
                    );
                }
            }
        }
    }

    diagnostics
}
