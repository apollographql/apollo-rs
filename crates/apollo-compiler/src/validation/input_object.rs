use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    validation::RecursionStack,
    Node, ValidationDatabase,
};
use std::collections::HashMap;

// Implements [Circular References](https://spec.graphql.org/October2021/#sec-Input-Objects.Circular-References)
// part of the input object validation spec.
struct FindRecursiveInputValue<'a> {
    db: &'a dyn ValidationDatabase,
}

impl FindRecursiveInputValue<'_> {
    fn input_value_definition(
        &self,
        seen: &mut RecursionStack<'_>,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        return match &def.ty {
            // NonNull type followed by Named type is the one that's not allowed
            // to be cyclical, so this is only case we care about.
            //
            // Everything else may be a cyclical input value.
            ast::Type::NonNullNamed(name) => {
                if !seen.contains(name) {
                    if let Some(def) = self.db.ast_types().input_objects.get(name) {
                        self.input_object_definition(seen.push(name.to_string()), def)?
                    }
                } else if seen.first() == Some(name) {
                    return Err(def.clone());
                }

                Ok(())
            }
            _ => Ok(()),
        };
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionStack<'_>,
        input_object: &ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        let mut guard = seen.push(input_object.definition.name.to_string());
        for input_value in input_object.fields() {
            self.input_value_definition(&mut guard, input_value)?;
        }

        Ok(())
    }

    fn check(
        db: &dyn ValidationDatabase,
        input_object: &ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        FindRecursiveInputValue { db }
            .input_object_definition(RecursionStack(&mut vec![]), input_object)
    }
}

pub fn validate_input_object_definitions(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for input_object in db.ast_types().input_objects.values() {
        diagnostics.extend(db.validate_input_object_definition(input_object.clone()));
    }

    diagnostics
}

pub fn validate_input_object_definition(
    db: &dyn ValidationDatabase,
    input_object: ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = super::directive::validate_directives(
        db,
        input_object.directives(),
        ast::DirectiveLocation::InputObject,
        // input objects don't use variables
        Default::default(),
    );

    if let Err(input_val) = FindRecursiveInputValue::check(db, &input_object) {
        let mut labels = vec![Label::new(
            input_object.definition.location().unwrap(),
            "cyclical input object definition",
        )];
        if let Some(loc) = input_val.location() {
            labels.push(Label::new(loc, "refers to itself here"));
        };
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                (input_object.definition.location().unwrap()).into(),
                DiagnosticData::RecursiveInputObjectDefinition {
                    name: input_object.definition.name.to_string(),
                },
            )
            .labels(labels),
        )
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    let fields: Vec<_> = input_object.fields().cloned().collect();
    diagnostics.extend(validate_input_value_definitions(
        db,
        &fields,
        ast::DirectiveLocation::InputFieldDefinition,
    ));

    diagnostics
}

pub fn validate_input_value_definitions(
    db: &dyn ValidationDatabase,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen: HashMap<ast::Name, &Node<ast::InputValueDefinition>> = HashMap::new();
    for input_value in input_values {
        let name = &input_value.name;
        diagnostics.extend(super::directive::validate_directives(
            db,
            input_value.directives.iter(),
            directive_location,
            Default::default(), // No variables in an input value definition
        ));

        if let Some(prev_value) = seen.get(name) {
            if let (Some(original_value), Some(redefined_value)) =
                (prev_value.location(), input_value.location())
            {
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        original_value.into(),
                        DiagnosticData::UniqueInputValue {
                            name: name.to_string(),
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
            seen.insert(name.clone(), input_value);
        }

        /* TODO(@goto-bus-stop) Port to new AST
        // Input values must only contain input types.
        let loc = input_value
            .loc()
            .expect("undefined input value definition location");
        if let Some(field_ty) = input_value.ty().type_def(db.upcast()) {
            if !input_value.ty().is_input_type(db.upcast()) {
                diagnostics.push(
                    ApolloDiagnostic::new(db, loc.into(), DiagnosticData::InputType {
                        name: input_value.name().into(),
                        ty: field_ty.kind(),
                    })
                        .label(Label::new(loc, format!("this is of `{}` type", field_ty.kind())))
                        .help(format!("Scalars, Enums, and Input Objects are input types. Change `{}` field to take one of these input types.", input_value.name())),
                );
            }
        } else if let Some(field_ty_loc) = input_value.ty().loc() {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    field_ty_loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: input_value.name().into(),
                    },
                )
                .label(Label::new(field_ty_loc, "not found in this scope")),
            );
        } else {
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    loc.into(),
                    DiagnosticData::UndefinedDefinition {
                        name: input_value.ty().name(),
                    },
                )
                .label(Label::new(loc, "not found in this scope")),
            );
        }
        */
    }
    diagnostics
}
