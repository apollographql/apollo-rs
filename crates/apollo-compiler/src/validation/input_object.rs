use std::{collections::HashMap, sync::Arc};

use crate::{
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    hir,
    validation::RecursionStack,
    ValidationDatabase,
};

// Implements [Circular References](https://spec.graphql.org/October2021/#sec-Input-Objects.Circular-References)
// part of the input object validation spec.
struct FindRecursiveInputValue<'a> {
    db: &'a dyn ValidationDatabase,
}

impl FindRecursiveInputValue<'_> {
    fn input_value_definition(
        &self,
        seen: &mut RecursionStack<'_>,
        def: &hir::InputValueDefinition,
    ) -> Result<(), hir::InputValueDefinition> {
        let ty = def.ty();
        return match ty {
            hir::Type::NonNull { ty, loc: _ } => match ty.as_ref() {
                // NonNull type followed by Named type is the one that's not allowed
                // to be cyclical, so this is only case we care about.
                //
                // Everything else may be a cyclical input value.
                hir::Type::Named { name, loc: _ } => {
                    if !seen.contains(name) {
                        if let Some(def) = self.db.find_input_object_by_name(name.into()) {
                            self.input_object_definition(seen.push(name.into()), def.as_ref())?
                        }
                    } else if seen.first() == Some(name) {
                        return Err(def.clone());
                    }

                    Ok(())
                }
                hir::Type::NonNull { .. } | hir::Type::List { .. } => Ok(()),
            },
            hir::Type::List { .. } => Ok(()),
            hir::Type::Named { .. } => Ok(()),
        };
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionStack<'_>,
        def: &hir::InputObjectTypeDefinition,
    ) -> Result<(), hir::InputValueDefinition> {
        let mut guard = seen.push(def.name().to_string());
        for input_value in def.fields() {
            self.input_value_definition(&mut guard, input_value)?;
        }

        Ok(())
    }

    fn check(
        db: &dyn ValidationDatabase,
        input_obj: &hir::InputObjectTypeDefinition,
    ) -> Result<(), hir::InputValueDefinition> {
        FindRecursiveInputValue { db }
            .input_object_definition(RecursionStack(&mut vec![]), input_obj)
    }
}

pub fn validate_input_object_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defs = &db.type_system_definitions().input_objects;
    for def in defs.values() {
        diagnostics.extend(db.validate_input_object_definition(def.clone()));
    }

    diagnostics
}

pub fn validate_input_object_definition(
    db: &dyn ValidationDatabase,
    input_obj: Arc<hir::InputObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = db.validate_directives(
        input_obj.directives().cloned().collect(),
        hir::DirectiveLocation::InputObject,
    );

    if let Err(input_val) = FindRecursiveInputValue::check(db, input_obj.as_ref()) {
        let mut labels = vec![Label::new(
            input_obj.loc(),
            "cyclical input object definition",
        )];
        if let Some(loc) = input_val.loc() {
            labels.push(Label::new(loc, "refers to itself here"));
        };
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                input_obj.loc().into(),
                DiagnosticData::RecursiveInputObjectDefinition {
                    name: input_obj.name().into(),
                },
            )
            .labels(labels),
        )
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    diagnostics.extend(db.validate_input_values(
        input_obj.input_fields_definition.clone(),
        hir::DirectiveLocation::InputFieldDefinition,
    ));

    diagnostics
}

pub fn validate_input_values(
    db: &dyn ValidationDatabase,
    input_values: Arc<Vec<hir::InputValueDefinition>>,
    // directive location depends on parent node location, so we pass this down from parent
    dir_loc: hir::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<&str, &hir::InputValueDefinition> = HashMap::new();

    for input_value in input_values.iter() {
        diagnostics.extend(db.validate_directives(input_value.directives().to_vec(), dir_loc));

        let name = input_value.name();
        if let Some(prev_value) = seen.get(name) {
            if let (Some(original_value), Some(redefined_value)) =
                (prev_value.loc(), input_value.loc())
            {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        original_value.into(),
                        DiagnosticData::UniqueInputValue {
                            name: name.into(),
                            original_value: original_value.into(),
                            redefined_value: redefined_value.into(),
                        },
                    )
                    .labels([
                        Label::new(
                            original_value,
                            format!("previous definition of `{name}` here"),
                        ),
                        Label::new(redefined_value, format!("`{name}` redefined here")),
                    ])
                    .help(format!(
                        "`{name}` field must only be defined once in this input object definition."
                    )),
                );
            }
        } else {
            seen.insert(name, input_value);
        }
    }

    diagnostics
}
