use crate::ast;
use crate::schema::{ExtendedType, InputObjectType};
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::{CycleError, DiagnosticList, RecursionGuard, RecursionStack};
use crate::Node;
use std::collections::HashMap;

// Implements [Circular References](https://spec.graphql.org/October2021/#sec-Input-Objects.Circular-References)
// part of the input object validation spec.
struct FindRecursiveInputValue<'a> {
    schema: &'a crate::Schema,
}

impl FindRecursiveInputValue<'_> {
    fn input_value_definition(
        &self,
        seen: &mut RecursionGuard<'_>,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        match &*def.ty {
            // NonNull type followed by Named type is the one that's not allowed
            // to be cyclical, so this is only case we care about.
            //
            // Everything else may be a cyclical input value.
            ast::Type::NonNullNamed(name) => {
                if !seen.contains(name) {
                    if let Some(object_def) = self.schema.get_input_object(name) {
                        self.input_object_definition(seen.push(name)?, object_def)
                            .map_err(|err| err.trace(def))?
                    }
                } else if seen.first() == Some(name) {
                    return Err(CycleError::Recursed(vec![def.clone()]));
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionGuard<'_>,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        for input_value in input_object.fields.values() {
            self.input_value_definition(&mut seen, input_value)?;
        }

        Ok(())
    }

    fn check(
        schema: &crate::Schema,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let mut recursion_stack = RecursionStack::with_root(input_object.name.clone());
        FindRecursiveInputValue { schema }
            .input_object_definition(recursion_stack.guard(), input_object)
    }
}

pub(crate) fn validate_input_object_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
) {
    for ty in schema.types.values() {
        if let ExtendedType::InputObject(input_object) = ty {
            validate_input_object_definition(diagnostics, schema, input_object);
        }
    }
}

pub(crate) fn validate_input_object_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    input_object: &Node<InputObjectType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        input_object.directives.iter_ast(),
        ast::DirectiveLocation::InputObject,
        // input objects don't use variables
        Default::default(),
    );

    match FindRecursiveInputValue::check(schema, input_object) {
        Ok(_) => {}
        Err(CycleError::Recursed(trace)) => diagnostics.push(
            input_object.location(),
            DiagnosticData::RecursiveInputObjectDefinition {
                name: input_object.name.clone(),
                trace,
            },
        ),
        Err(CycleError::Limit(_)) => {
            diagnostics.push(
                input_object.location(),
                DiagnosticData::DeeplyNestedType {
                    name: input_object.name.clone(),
                    describe_type: "input object",
                },
            );
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    let fields: Vec<_> = input_object
        .fields
        .values()
        .map(|c| c.node.clone())
        .collect();
    validate_input_value_definitions(
        diagnostics,
        schema,
        &fields,
        ast::DirectiveLocation::InputFieldDefinition,
    );
}

pub(crate) fn validate_argument_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) {
    validate_input_value_definitions(diagnostics, schema, input_values, directive_location);

    let mut seen: HashMap<ast::Name, &Node<ast::InputValueDefinition>> = HashMap::new();
    for input_value in input_values {
        let name = &input_value.name;
        if let Some(prev_value) = seen.get(name) {
            let (original_definition, redefined_definition) =
                (prev_value.location(), input_value.location());

            diagnostics.push(
                original_definition,
                DiagnosticData::UniqueInputValue {
                    name: name.clone(),
                    original_definition,
                    redefined_definition,
                },
            );
        } else {
            seen.insert(name.clone(), input_value);
        }
    }
}

pub(crate) fn validate_input_value_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) {
    for input_value in input_values {
        super::directive::validate_directives(
            diagnostics,
            Some(schema),
            input_value.directives.iter(),
            directive_location,
            Default::default(), // No variables in an input value definition
        );
        // Input values must only contain input types.
        let loc = input_value.location();
        if let Some(field_ty) = schema.types.get(input_value.ty.inner_named_type()) {
            if !field_ty.is_input_type() {
                diagnostics.push(
                    loc,
                    DiagnosticData::InputType {
                        name: input_value.name.clone(),
                        describe_type: field_ty.describe(),
                        type_location: input_value.ty.location(),
                    },
                );
            }
        } else {
            let named_type = input_value.ty.inner_named_type();
            let loc = named_type.location();
            diagnostics.push(
                loc,
                DiagnosticData::UndefinedDefinition {
                    name: named_type.clone(),
                },
            );
        }
    }
}
