use crate::{
    parser::grammar::{
        directive, enum_, extensions, fragment, input, interface, object, operation, scalar,
        schema, union_,
    },
    Parser, SyntaxKind, TokenKind,
};

/// See: https://spec.graphql.org/draft/#Document
///
/// *Document*
///     Definition<sub>list</sub>
pub(crate) fn document(p: &mut Parser) {
    let doc = p.start_node(SyntaxKind::DOCUMENT);

    while let Some(node) = p.peek() {
        match node {
            TokenKind::StringValue => {
                let def = p.peek_data_n(2).unwrap();
                select_definition(def, p);
            }
            TokenKind::Name => {
                let def = p.peek_data().unwrap();
                select_definition(def, p);
            }
            TokenKind::LCurly => {
                let def = p.peek_data().unwrap();
                select_definition(def, p);
            }
            _ => break,
        }
    }

    doc.finish_node();
}

fn select_definition(def: String, p: &mut Parser) {
    match def.as_str() {
        "directive" => directive::directive_definition(p),
        "enum" => enum_::enum_type_definition(p),
        "extend" => extensions::extensions(p),
        "fragment" => fragment::fragment_definition(p),
        "input" => input::input_object_type_definition(p),
        "interface" => interface::interface_type_definition(p),
        "type" => object::object_type_definition(p),
        "query" | "mutation" | "subscription" | "{" => operation::operation_definition(p),
        "scalar" => scalar::scalar_type_definition(p),
        "schema" => schema::schema_definition(p),
        "union" => union_::union_type_definition(p),
        _ => p.err_and_pop("expected definition"),
    }
}

pub(crate) fn is_definition(def: String) -> bool {
    matches!(
        def.as_str(),
        "directive"
            | "enum"
            | "extend"
            | "fragment"
            | "input"
            | "interface"
            | "type"
            | "query"
            | "mutation"
            | "subscription"
            | "{"
            | "scalar"
            | "schema"
            | "union"
    )
}

#[cfg(test)]
mod test {
    use crate::{ast, Parser};

    #[test]
    fn it_creates_error_for_invalid_definition_and_has_nodes_for_valid_definition() {
        let schema = r#"
uasdf21230jkdw

{
    pet
    faveSnack
}
        "#;
        let parser = Parser::new(schema);

        let ast = parser.parse();
        assert_eq!(ast.errors().len(), 1);

        let doc = ast.document();
        let nodes: Vec<_> = doc.definitions().into_iter().collect();
        assert_eq!(nodes.len(), 1);
    }

    #[test]
    fn it_creates_an_error_for_a_document_with_only_an_invalid_definition() {
        let schema = r#"dtzt7777777777t7777777777z7"#;
        let parser = Parser::new(schema);

        let ast = parser.parse();
        assert_eq!(ast.errors().len(), 1);

        let doc = ast.document();
        let nodes: Vec<_> = doc.definitions().into_iter().collect();
        assert!(nodes.is_empty());
    }

    #[test]
    fn core_schema() {
        let schema = r#"
schema @core(feature: "https://specs.apollo.dev/join/v0.1") {
  query: Query
  mutation: Mutation
}

enum join__Graph {
  ACCOUNTS @join__graph(name: "accounts")
}
        "#;
        let parser = crate::Parser::new(schema);
        let ast = parser.parse();

        assert!(ast.errors.is_empty());

        let document = ast.document();
        for definition in document.definitions() {
            if let ast::Definition::EnumTypeDefinition(enum_type) = definition {
                let enum_name = enum_type
                    .name()
                    .expect("Could not get Enum Type Definition's Name");

                assert_eq!("join__Graph", enum_name.text().as_ref());

                if enum_name.text().as_ref() == "join__Graph" {
                    if let Some(enums) = enum_type.enum_values_definition() {
                        for enum_kind in enums.enum_value_definitions() {
                            assert_eq!(
                                "ACCOUNTS",
                                enum_kind
                                    .enum_value()
                                    .unwrap()
                                    .name()
                                    .unwrap()
                                    .text()
                                    .as_ref()
                            );
                            check_directive_arguments(enum_kind);
                        }
                    }
                }
            }
        }

        fn check_directive_arguments(enum_kind: ast::EnumValueDefinition) {
            if let Some(directives) = enum_kind.directives() {
                for directive in directives.directives() {
                    if directive
                        .name()
                        .and_then(|n| n.ident_token())
                        .as_ref()
                        .map(|id| id.text())
                        == Some("join__graph")
                    {
                        for argument in directive.arguments().unwrap().arguments() {
                            if let ast::Value::StringValue(val) =
                                argument.value().expect("Cannot get argument value.")
                            {
                                let val: String = val.into();
                                assert_eq!("accounts".to_string(), val);
                            }
                        }
                    }
                }
            }
        }
    }

}
