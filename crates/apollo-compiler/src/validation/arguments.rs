use std::collections::HashMap;
use crate::{
    diagnostics::UniqueArgument,
    hir::InputValueDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

fn check_input_value_definition_uniqueness(db: &dyn ValidationDatabase, input_values: &[InputValueDefinition], diagnostics: &mut Vec<ApolloDiagnostic>) {
    let mut seen: HashMap<&str, &InputValueDefinition> = HashMap::new();
    for input_value in input_values {
        let name = input_value.name();
        if let Some(prev_def) = seen.get(name) {
            let prev_offset: usize = prev_def.ast_node(db.upcast()).unwrap().text_range().start().into();
            let prev_node_len: usize = prev_def.ast_node(db.upcast()).unwrap().text_range().len().into();

            let current_offset: usize = input_value.ast_node(db.upcast()).unwrap().text_range().start().into();
            let current_node_len: usize = input_value.ast_node(db.upcast()).unwrap().text_range().len().into();

            diagnostics.push(ApolloDiagnostic::UniqueArgument(UniqueArgument {
                name: name.into(),
                src: db.input(),
                original_definition: (prev_offset, prev_node_len).into(),
                redefined_definition: (current_offset, current_node_len).into(),
                help: Some(format!("`{name}` argument must only be defined once.")),
            }));
        } else {
            seen.insert(name, &input_value);
        }
    }
}

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    let object_types = db.object_types();
    let directive_definitions = db.directive_definitions();

    let object_input_values = object_types.iter()
        .flat_map(|object_type| object_type.fields_definition())
        .map(|field| field.arguments().input_values());

    let directive_input_values = directive_definitions.iter()
        // Builtin directives do not have a backing AST Node. We don't need to check those.
        .filter(|directive_definition| directive_definition.ast_node(db.upcast()).is_some())
        .map(|directive_definition| directive_definition.arguments().input_values());

    for input_values in object_input_values.chain(directive_input_values) {
        check_input_value_definition_uniqueness(db, input_values, &mut diagnostics);
    }

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_duplicate_field_argument_names() {
        let input = r#"
type Query {
  single(arg: Boolean): Int
  duplicate(arg: Boolean, arg: Boolean): Int
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn it_fails_validation_with_duplicate_directive_argument_names() {
        let input = r#"directive @example(arg: Boolean, arg: Boolean) on FIELD"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }
}
