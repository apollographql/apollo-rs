use std::collections::HashMap;
use crate::{
    diagnostics::UniqueDefinition,
    hir::InputValueDefinition,
    ApolloDiagnostic, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();

    for object_type in db.object_types().iter() {
        for field in object_type.fields_definition() {
            let mut seen: HashMap<&str, &InputValueDefinition> = HashMap::new();
            let input_values = field.arguments().input_values();
            for input_value in input_values {
                let name = input_value.name();
                if let Some(prev_def) = seen.get(name) {
                    let prev_offset: usize = prev_def.ast_node(db.upcast()).unwrap().text_range().start().into();
                    let prev_node_len: usize = prev_def.ast_node(db.upcast()).unwrap().text_range().len().into();

                    let current_offset: usize = input_value.ast_node(db.upcast()).unwrap().text_range().start().into();
                    let current_node_len: usize = input_value.ast_node(db.upcast()).unwrap().text_range().len().into();

                    diagnostics.push(ApolloDiagnostic::UniqueDefinition(UniqueDefinition {
                        ty: "argument".into(),
                        name: name.into(),
                        src: db.input(),
                        original_definition: (prev_offset, prev_node_len).into(),
                        redefined_definition: (current_offset, current_node_len).into(),
                        help: Some(format!(
                            "`{name}` argument must only be defined once."
                        )),
                    }));
                } else {
                    seen.insert(name, &input_value);
                }
            }
        }
    }

    diagnostics
}

#[cfg(test)]
mod test {
    use crate::ApolloCompiler;

    #[test]
    fn it_fails_validation_with_duplicate_argument_names() {
        let input = r#"
type Query {
  method(arg: Boolean, arg: Boolean): Int
}
"#;
        let ctx = ApolloCompiler::new(input);
        let diagnostics = ctx.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1);
    }
}
