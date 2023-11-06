use crate::{
    ast,
    diagnostics::{ApolloDiagnostic, DiagnosticData, Label},
    schema,
    validation::{RecursionGuard, RecursionStack},
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
        seen: &mut RecursionGuard<'_>,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        return match &*def.ty {
            // NonNull type followed by Named type is the one that's not allowed
            // to be cyclical, so this is only case we care about.
            //
            // Everything else may be a cyclical input value.
            ast::Type::NonNullNamed(name) => {
                if !seen.contains(name) {
                    if let Some(def) = self.db.ast_types().input_objects.get(name) {
                        self.input_object_definition(seen.push(name), def)?
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
        mut seen: RecursionGuard<'_>,
        input_object: &ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        for input_value in input_object.fields() {
            self.input_value_definition(&mut seen, input_value)?;
        }

        Ok(())
    }

    fn check(
        db: &dyn ValidationDatabase,
        input_object: &ast::TypeWithExtensions<ast::InputObjectTypeDefinition>,
    ) -> Result<(), Node<ast::InputValueDefinition>> {
        let mut recursion_stack = RecursionStack::with_root(input_object.definition.name.clone());
        FindRecursiveInputValue { db }
            .input_object_definition(recursion_stack.guard(), input_object)
    }
}

pub(crate) fn validate_input_object_definitions(
    db: &dyn ValidationDatabase,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for input_object in db.ast_types().input_objects.values() {
        diagnostics.extend(db.validate_input_object_definition(input_object.clone()));
    }

    diagnostics
}

pub(crate) fn validate_input_object_definition(
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
            input_object.definition.location(),
            "cyclical input object definition",
        )];
        let loc = input_val.location();
        labels.push(Label::new(loc, "refers to itself here"));
        diagnostics.push(
            ApolloDiagnostic::new(
                db,
                input_object.definition.location(),
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

pub(crate) fn validate_argument_definitions(
    db: &dyn ValidationDatabase,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = validate_input_value_definitions(db, input_values, directive_location);

    let mut seen: HashMap<ast::Name, &Node<ast::InputValueDefinition>> = HashMap::new();
    for input_value in input_values {
        let name = &input_value.name;
        if let Some(prev_value) = seen.get(name) {
            let (original_value, redefined_value) = (prev_value.location(), input_value.location());

            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    original_value,
                    DiagnosticData::UniqueInputValue {
                        name: name.to_string(),
                        original_value,
                        redefined_value,
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
        } else {
            seen.insert(name.clone(), input_value);
        }
    }

    diagnostics
}

pub(crate) fn validate_input_value_definitions(
    db: &dyn ValidationDatabase,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) -> Vec<ApolloDiagnostic> {
    let schema = db.schema();

    let mut diagnostics = Vec::new();

    for input_value in input_values {
        diagnostics.extend(super::directive::validate_directives(
            db,
            input_value.directives.iter(),
            directive_location,
            Default::default(), // No variables in an input value definition
        ));
        // Input values must only contain input types.
        let loc = input_value.location();
        if let Some(field_ty) = schema.types.get(input_value.ty.inner_named_type()) {
            if !field_ty.is_input_type() {
                let (particle, kind) = match field_ty {
                    schema::ExtendedType::Scalar(_) => unreachable!(),
                    schema::ExtendedType::Object(_) => ("an", "object"),
                    schema::ExtendedType::Interface(_) => ("an", "interface"),
                    schema::ExtendedType::Union(_) => ("a", "union"),
                    schema::ExtendedType::Enum(_) => unreachable!(),
                    schema::ExtendedType::InputObject(_) => unreachable!(),
                };
                diagnostics.push(
                    ApolloDiagnostic::new(db, loc, DiagnosticData::InputType {
                        name: input_value.name.to_string(),
                        ty: kind,
                    })
                        .label(Label::new(loc, format!("this is {particle} {kind}")))
                        .help(format!("Scalars, Enums, and Input Objects are input types. Change `{}` field to take one of these input types.", input_value.name)),
                );
            }
        } else {
            let named_type = input_value.ty.inner_named_type();
            let loc = named_type.location();
            diagnostics.push(
                ApolloDiagnostic::new(
                    db,
                    loc,
                    DiagnosticData::UndefinedDefinition {
                        name: named_type.to_string(),
                    },
                )
                .label(Label::new(loc, "not found in this scope")),
            );
        }
    }

    diagnostics
}
