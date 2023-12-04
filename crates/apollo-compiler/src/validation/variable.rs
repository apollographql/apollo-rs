use crate::diagnostics::{ApolloDiagnostic, DiagnosticData, Label};
use crate::validation::{
    FileId, NodeLocation, RecursionGuard, RecursionLimitError, RecursionStack,
};
use crate::{ast, schema, Node, ValidationDatabase};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};

pub(crate) fn validate_variable_definitions(
    db: &dyn ValidationDatabase,
    variables: &[Node<ast::VariableDefinition>],
    has_schema: bool,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let schema = db.schema();

    let mut seen: HashMap<ast::Name, &Node<ast::VariableDefinition>> = HashMap::new();
    for variable in variables.iter() {
        diagnostics.extend(super::directive::validate_directives(
            db,
            variable.directives.iter(),
            ast::DirectiveLocation::VariableDefinition,
            // let's assume that variable definitions cannot reference other
            // variables and provide them as arguments to directives
            Default::default(),
        ));

        if has_schema {
            let ty = &variable.ty;
            let type_definition = schema.types.get(ty.inner_named_type());

            match type_definition {
                Some(type_definition) if type_definition.is_input_type() => {
                    // OK!
                }
                Some(type_definition) => {
                    let kind = match type_definition {
                        schema::ExtendedType::Scalar(_) => "scalar",
                        schema::ExtendedType::Object(_) => "object",
                        schema::ExtendedType::Interface(_) => "interface",
                        schema::ExtendedType::Union(_) => "union",
                        schema::ExtendedType::Enum(_) => "enum",
                        schema::ExtendedType::InputObject(_) => "input object",
                    };
                    diagnostics.push(
                        ApolloDiagnostic::new(db, variable.location(), DiagnosticData::InputType {
                            name: variable.name.to_string(),
                            ty: kind,
                        })
                        .label(Label::new(ty.inner_named_type().location(), format!("this is of `{kind}` type")))
                        .help("objects, unions, and interfaces cannot be used because variables can only be of input type"),
                        );
                }
                None => diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        variable.location(),
                        DiagnosticData::UndefinedDefinition {
                            name: ty.inner_named_type().to_string(),
                        },
                    )
                    .label(Label::new(
                        ty.inner_named_type().location(),
                        "not found in the type system",
                    )),
                ),
            }
        }

        match seen.entry(variable.name.clone()) {
            Entry::Occupied(original) => {
                let original_definition = original.get().location();
                let redefined_definition = variable.location();
                diagnostics.push(
                    ApolloDiagnostic::new(
                        db,
                        redefined_definition,
                        DiagnosticData::UniqueDefinition {
                            ty: "variable",
                            name: variable.name.to_string(),
                            original_definition,
                            redefined_definition,
                        },
                    )
                    .labels([
                        Label::new(
                            original_definition,
                            format!("previous definition of `{}` here", variable.name),
                        ),
                        Label::new(
                            redefined_definition,
                            format!("`{}` redefined here", variable.name),
                        ),
                    ])
                    .help(format!(
                        "{} must only be defined once in this enum.",
                        variable.name
                    )),
                );
            }
            Entry::Vacant(entry) => {
                entry.insert(variable);
            }
        }
    }

    diagnostics
}

fn walk_selections(
    document: &ast::Document,
    selections: &[ast::Selection],
    mut f: impl FnMut(&ast::Selection),
) -> Result<(), RecursionLimitError> {
    type NamedFragments = HashMap<ast::Name, Node<ast::FragmentDefinition>>;
    let named_fragments: NamedFragments = document
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            ast::Definition::FragmentDefinition(fragment) => {
                Some((fragment.name.clone(), fragment.clone()))
            }
            _ => None,
        })
        .collect();

    fn walk_selections_inner<'ast, 'guard>(
        named_fragments: &'ast NamedFragments,
        selections: &'ast [ast::Selection],
        guard: &mut RecursionGuard<'guard>,
        f: &mut dyn FnMut(&ast::Selection),
    ) -> Result<(), RecursionLimitError> {
        for selection in selections {
            f(selection);
            match selection {
                ast::Selection::Field(field) => {
                    walk_selections_inner(named_fragments, &field.selection_set, guard, f)?;
                }
                ast::Selection::FragmentSpread(fragment) => {
                    // Prevent chasing a cyclical reference.
                    // Note we do not report `CycleError::Recursed` here, as that is already caught
                    // by the cyclical fragment validation--we just need to ensure that we don't
                    // overflow the stack.
                    if guard.contains(&fragment.fragment_name) {
                        continue;
                    }

                    if let Some(fragment_definition) = named_fragments.get(&fragment.fragment_name)
                    {
                        walk_selections_inner(
                            named_fragments,
                            &fragment_definition.selection_set,
                            &mut guard.push(&fragment.fragment_name)?,
                            f,
                        )?;
                    }
                }
                ast::Selection::InlineFragment(fragment) => {
                    walk_selections_inner(named_fragments, &fragment.selection_set, guard, f)?;
                }
            }
        }
        Ok(())
    }

    let mut stack = RecursionStack::new().with_limit(100);
    let result = walk_selections_inner(&named_fragments, selections, &mut stack.guard(), &mut f);
    result
}

fn variables_in_value(value: &ast::Value) -> impl Iterator<Item = ast::Name> + '_ {
    let mut value_stack = vec![value];
    std::iter::from_fn(move || {
        while let Some(value) = value_stack.pop() {
            match value {
                ast::Value::Variable(variable) => return Some(variable.clone()),
                ast::Value::List(list) => value_stack.extend(list.iter().map(|value| &**value)),
                ast::Value::Object(fields) => {
                    value_stack.extend(fields.iter().map(|(_, value)| &**value))
                }
                _ => (),
            }
        }
        None
    })
}

fn variables_in_arguments(args: &[Node<ast::Argument>]) -> impl Iterator<Item = ast::Name> + '_ {
    args.iter().flat_map(|arg| variables_in_value(&arg.value))
}

fn variables_in_directives(
    directives: &[Node<ast::Directive>],
) -> impl Iterator<Item = ast::Name> + '_ {
    directives
        .iter()
        .flat_map(|directive| variables_in_arguments(&directive.arguments))
}

// TODO add test:
// should NOT report a unused variable warning
// query ($var1: Boolean!, $var2: Boolean!) {
//   a: field (arg: $var1)
//   a: field (arg: $var2)
// }
pub(crate) fn validate_unused_variables(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    operation: Node<ast::OperationDefinition>,
) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let defined_vars: HashSet<_> = operation
        .variables
        .iter()
        .map(|var| var.name.clone())
        .collect();
    let locations: HashMap<_, _> = operation
        .variables
        .iter()
        .map(|var| {
            (
                &var.name,
                NodeLocation::recompose(var.location(), var.name.location()),
            )
        })
        .collect();
    let mut used_vars = HashSet::<ast::Name>::new();
    let walked =
        walk_selections(
            &db.ast(file_id),
            &operation.selection_set,
            |selection| match selection {
                ast::Selection::Field(field) => {
                    used_vars.extend(variables_in_directives(&field.directives));
                    used_vars.extend(variables_in_arguments(&field.arguments));
                }
                ast::Selection::FragmentSpread(fragment) => {
                    used_vars.extend(variables_in_directives(&fragment.directives));
                }
                ast::Selection::InlineFragment(fragment) => {
                    used_vars.extend(variables_in_directives(&fragment.directives));
                }
            },
        );
    if walked.is_err() {
        diagnostics.push(ApolloDiagnostic::new(
            db,
            None,
            DiagnosticData::RecursionError {},
        ));
        return diagnostics;
    }

    let unused_vars = defined_vars.difference(&used_vars);

    diagnostics.extend(unused_vars.map(|unused_var| {
        let loc = locations[unused_var];
        ApolloDiagnostic::new(
            db,
            loc,
            DiagnosticData::UnusedVariable {
                name: unused_var.to_string(),
            },
        )
        .label(Label::new(loc, "this variable is never used"))
    }));

    diagnostics
}

pub(crate) fn validate_variable_usage(
    db: &dyn ValidationDatabase,
    var_usage: Node<ast::InputValueDefinition>,
    var_defs: &[Node<ast::VariableDefinition>],
    argument: &Node<ast::Argument>,
) -> Result<(), ApolloDiagnostic> {
    if let ast::Value::Variable(var_name) = &*argument.value {
        // Let var_def be the VariableDefinition named
        // variable_name defined within operation.
        let var_def = var_defs.iter().find(|v| v.name == *var_name);
        if let Some(var_def) = var_def {
            let is_allowed = is_variable_usage_allowed(var_def, &var_usage);
            if !is_allowed {
                return Err(ApolloDiagnostic::new(
                    db,
                    argument.location(),
                    DiagnosticData::DisallowedVariableUsage {
                        var_name: var_def.name.to_string(),
                        arg_name: argument.name.to_string(),
                    },
                )
                .labels([
                    Label::new(
                        var_def.location(),
                        format!(
                            "variable `{}` of type `{}` is declared here",
                            var_def.name, var_def.ty,
                        ),
                    ),
                    Label::new(
                        argument.location(),
                        format!(
                            "argument `{}` of type `{}` is declared here",
                            argument.name, var_usage.ty,
                        ),
                    ),
                ]));
            }
        } else {
            return Err(ApolloDiagnostic::new(
                db,
                argument.location(),
                DiagnosticData::UndefinedVariable {
                    name: var_name.to_string(),
                },
            )
            .label(Label::new(
                argument.value.location(),
                "not found in this scope",
            )));
        }
    }
    // It's super confusing to produce a diagnostic here if either the
    // location_ty or variable_ty is missing, so just return Ok(());
    Ok(())
}

fn is_variable_usage_allowed(
    variable_def: &ast::VariableDefinition,
    variable_usage: &ast::InputValueDefinition,
) -> bool {
    // 1. Let variable_ty be the expected type of variable_def.
    let variable_ty = &variable_def.ty;
    // 2. Let location_ty be the expected type of the Argument,
    // ObjectField, or ListValue entry where variableUsage is
    // located.
    let location_ty = &variable_usage.ty;
    // 3. if location_ty is a non-null type AND variable_ty is
    // NOT a non-null type:
    if location_ty.is_non_null() && !variable_ty.is_non_null() {
        // 3.a. let hasNonNullVariableDefaultValue be true
        // if a default value exists for variableDefinition
        // and is not the value null.
        let has_non_null_default_value = variable_def.default_value.is_some();
        // 3.b. Let hasLocationDefaultValue be true if a default
        // value exists for the Argument or ObjectField where
        // variableUsage is located.
        let has_location_default_value = variable_usage.default_value.is_some();
        // 3.c. If hasNonNullVariableDefaultValue is NOT true
        // AND hasLocationDefaultValue is NOT true, return
        // false.
        if !has_non_null_default_value && !has_location_default_value {
            return false;
        }

        // 3.d. Let nullable_location_ty be the unwrapped
        // nullable type of location_ty.
        return variable_ty.is_assignable_to(&location_ty.as_ref().clone().nullable());
    }

    variable_ty.is_assignable_to(location_ty)
}
