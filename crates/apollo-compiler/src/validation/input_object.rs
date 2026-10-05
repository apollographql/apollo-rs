use crate::ast;
use crate::collections::HashMap;
use crate::coordinate::TypeAttributeCoordinate;
use crate::schema::validation::BuiltInScalars;
use crate::schema::InputObjectType;
use crate::validation::diagnostics::DiagnosticData;
use crate::validation::value::value_of_correct_type;
use crate::validation::CycleError;
use crate::validation::DiagnosticList;
use crate::validation::RecursionGuard;
use crate::validation::RecursionStack;
use crate::Name;
use crate::Node;

// Implements `InputObjectHasUnbreakableCycle()` from the Input Objects type
// validation rules. An `Err(CycleError::Recursed)` means no finite value of the
// input object can be constructed, and carries the field path that proves it.
struct FindUnbreakableCycle<'a> {
    schema: &'a crate::Schema,
}

impl FindUnbreakableCycle<'_> {
    fn input_field_type(
        &self,
        seen: &mut RecursionGuard<'_>,
        def: &Node<ast::InputValueDefinition>,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        // Lists can always be empty, so they break any cycle.
        let (ast::Type::Named(name) | ast::Type::NonNullNamed(name)) = &*def.ty else {
            return Ok(());
        };

        // Any revisit is unbreakable, even when it does not return to the root
        // type. The root still cannot be constructed if every way out of it
        // ends in a cycle elsewhere.
        if seen.contains(name) {
            return Err(CycleError::Recursed(vec![def.clone()]));
        }
        match self.schema.get_input_object(name) {
            Some(object_def) => self
                .input_object_definition(seen.push(name)?, object_def)
                .map_err(|err| err.trace(def)),
            None => Ok(()),
        }
    }

    fn input_object_definition(
        &self,
        mut seen: RecursionGuard<'_>,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        if input_object.is_one_of() {
            // Only one field is provided, so a single breakable field is enough.
            // An empty @oneOf is reported separately as an empty input object.
            let mut last_err = None;
            for field in input_object.fields.values() {
                match self.input_field_type(&mut seen, field) {
                    Ok(()) => return Ok(()),
                    Err(err) => last_err = Some(err),
                }
            }
            last_err.map_or(Ok(()), Err)
        } else {
            for field in input_object.fields.values() {
                if field.ty.is_non_null() {
                    self.input_field_type(&mut seen, field)?;
                }
            }
            Ok(())
        }
    }

    fn check(
        schema: &crate::Schema,
        input_object: &InputObjectType,
    ) -> Result<(), CycleError<ast::InputValueDefinition>> {
        let mut recursion_stack = RecursionStack::with_root(input_object.name.clone());
        FindUnbreakableCycle { schema }
            .input_object_definition(recursion_stack.guard(), input_object)
    }
}

pub(crate) fn validate_input_object_definition(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    input_object: &Node<InputObjectType>,
) {
    super::directive::validate_directives(
        diagnostics,
        Some(schema),
        input_object.directives.iter(),
        ast::DirectiveLocation::InputObject,
        // input objects don't use variables
        Default::default(),
    );

    match FindUnbreakableCycle::check(schema, input_object) {
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

    // @oneOf must not be provided by an input object type extension.
    // https://spec.graphql.org/September2025/#sec-Input-Object-Extensions
    for directive in &input_object.directives.0 {
        if directive.name == "oneOf" {
            if let Some(ext_id) = directive.extension_id() {
                diagnostics.push(
                    directive.location(),
                    DiagnosticData::OneOfDirectiveOnExtension {
                        type_name: input_object.name.clone(),
                        extension_location: ext_id.location(),
                    },
                );
            }
        }
    }

    // @oneOf input objects: all fields must be nullable and must not have default values.
    // https://spec.graphql.org/September2025/#sec-OneOf-Input-Objects
    if input_object.is_one_of() {
        for (field_name, field) in &input_object.fields {
            if field.ty.is_non_null() {
                diagnostics.push(
                    field.location(),
                    DiagnosticData::OneOfInputObjectFieldNonNull {
                        coordinate: TypeAttributeCoordinate {
                            ty: input_object.name.clone(),
                            attribute: field_name.clone(),
                        },
                        definition_location: field.location(),
                    },
                );
            }
            if field.default_value.is_some() {
                let default_location = field.default_value.as_ref().and_then(|v| v.location());
                diagnostics.push(
                    field.location(),
                    DiagnosticData::UnsupportedDefault {
                        coordinate: TypeAttributeCoordinate {
                            ty: input_object.name.clone(),
                            attribute: field_name.clone(),
                        },
                        default_location,
                    },
                );
            }
        }
    }

    // Fields in an Input Object Definition must be unique
    //
    // Returns Unique Definition error.
    let fields: Vec<_> = input_object.fields.values().cloned().collect();
    validate_input_value_definitions(
        diagnostics,
        schema,
        built_in_scalars,
        &fields,
        ast::DirectiveLocation::InputFieldDefinition,
        "an input object field",
    );

    // validate there is at least one input value on the input object type
    // https://spec.graphql.org/September2025/#sec-Input-Objects.Type-Validation
    if input_object.fields.is_empty() {
        diagnostics.push(
            input_object.location(),
            DiagnosticData::EmptyInputValueSet {
                type_name: input_object.name.clone(),
                type_location: input_object.location(),
                extensions_locations: input_object
                    .extensions()
                    .iter()
                    .map(|ext| ext.location())
                    .collect(),
            },
        );
    }
}

pub(crate) fn validate_argument_definitions(
    diagnostics: &mut DiagnosticList,
    schema: &crate::Schema,
    built_in_scalars: &mut BuiltInScalars,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
) {
    validate_input_value_definitions(
        diagnostics,
        schema,
        built_in_scalars,
        input_values,
        directive_location,
        "an argument",
    );

    let mut seen: HashMap<Name, &Node<ast::InputValueDefinition>> = HashMap::default();
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
    built_in_scalars: &mut BuiltInScalars,
    input_values: &[Node<ast::InputValueDefinition>],
    directive_location: ast::DirectiveLocation,
    describe: &'static str,
) {
    for input_value in input_values {
        crate::schema::validation::validate_type_system_name(
            diagnostics,
            &input_value.name,
            describe,
        );
        super::directive::validate_directives(
            diagnostics,
            Some(schema),
            input_value.directives.iter(),
            directive_location,
            Default::default(), // No variables in an input value definition
        );
        // https://spec.graphql.org/September2025/#sec--deprecated
        // > The @deprecated directive must not appear on required (non-null
        // > without a default) arguments or input object field definitions.
        if input_value.ty.is_non_null() && input_value.default_value.is_none() {
            if let Some(deprecated) = input_value.directives.get("deprecated") {
                diagnostics.push(
                    deprecated.location(),
                    DiagnosticData::DeprecatedRequiredInputValue {
                        name: input_value.name.clone(),
                        describe,
                        definition_location: input_value.location(),
                    },
                );
            }
        }
        // Input values must only contain input types.
        let loc = input_value.location();
        let named_type = input_value.ty.inner_named_type();
        let is_built_in = built_in_scalars.record_type_ref(schema, named_type);
        if let Some(field_ty) = schema.types.get(named_type) {
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
            if schema.validate_default_values {
                if let Some(default) = &input_value.default_value {
                    let var_defs = &[];
                    value_of_correct_type(
                        diagnostics,
                        schema,
                        &input_value.ty,
                        default,
                        var_defs,
                        None,
                    );
                }
            }
        } else if is_built_in {
            // `validate_schema()` will insert the missing definition
        } else {
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
