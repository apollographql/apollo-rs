use crate::{
    diagnostics::IntrospectionField, hir::Selection, ApolloDiagnostic, FileId, ValidationDatabase,
};

pub fn check(db: &dyn ValidationDatabase, file_id: FileId) -> Vec<ApolloDiagnostic> {
    let mut diagnostics = Vec::new();
    let subscription_operations = db.upcast().subscription_operations(file_id);

    // Subscription fields must not have an introspection field in the selection set.
    //
    // Return an Introspection Field error in case of an introspection field existing as the root field in the set.
    for op in subscription_operations.iter() {
        for selection in op.selection_set().selection() {
            if let Selection::Field(field) = selection {
                if field.name().starts_with("__") {
                    diagnostics.push(ApolloDiagnostic::IntrospectionField(IntrospectionField {
                        field: field.name().into(),
                        src: db.source_code(op.loc().file_id()),
                        definition: (field.loc.offset(), field.loc.node_len()).into(),
                        help: Some(format!(
                            "`{}` is an introspection field. This is not allowed as the root field for Subscription operations.", field.name()
                        )),
                    }));
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
    fn it_raises_an_error_for_illegal_introspection_root_field() {
        let input = r#"
            subscription sub {
                __typename
            }

            type Subscription {
                __typename: String
            }
        "#;
        let mut compiler = ApolloCompiler::new();
        compiler.add_document(input, "schema.graphql");

        let diagnostics = compiler.validate();
        for diagnostic in &diagnostics {
            println!("{}", diagnostic)
        }
        assert_eq!(diagnostics.len(), 1)
    }
}
