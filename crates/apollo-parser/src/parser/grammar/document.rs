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
        _ => p.err("expected definition"),
    }
}

#[cfg(test)]
mod test {
    use crate::{ast::Definition, Parser};

    #[test]
    fn core_schema() {
        let schema = r#"schema
    @core(feature: "https://specs.apollo.dev/join/v0.1")
  {
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
            if let crate::ast::Definition::EnumTypeDefinition(enum_type) = definition {
                assert_eq!(
                    Some("join__Graph".to_string()),
                    enum_type
                        .name()
                        .and_then(|n| n.ident_token())
                        .map(|id| id.text().to_owned())
                );

                if enum_type
                    .name()
                    .and_then(|n| n.ident_token())
                    .as_ref()
                    .map(|id| id.text())
                    == Some("join__Graph")
                {
                    if let Some(enums) = enum_type.enum_values_definition() {
                        for enum_kind in enums.enum_value_definitions() {
                            assert_eq!(
                                Some("ACCOUNTS"),
                                enum_kind
                                    .enum_value()
                                    .and_then(|v| v.name())
                                    .and_then(|n| n.ident_token())
                                    .as_ref()
                                    .map(|id| id.text())
                            );

                            if let Some(directives) = enum_kind.directives() {
                                for directive in directives.directives() {
                                    if directive
                                        .name()
                                        .and_then(|n| n.ident_token())
                                        .as_ref()
                                        .map(|id| id.text())
                                        == Some("join__graph")
                                    {
                                        if let Some(arguments) = directive.arguments() {
                                            for argument in arguments.arguments() {
                                                assert_eq!(
                                                    "\"accounts\"".to_string(),
                                                    argument.value().unwrap().to_string()
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn query() {
        let input = "
query GraphQuery($graph_id: ID!, $variant: String) {
  service(id: $graph_id) {
    schema(tag: $variant) {
      document
    }
  }
}
";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert!(&ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::OperationDefinition(op_def) = def {
                assert_eq!(op_def.name().unwrap().text(), "GraphQuery");

                let variable_defs = op_def.variable_definitions();
                let variables: Vec<String> = variable_defs
                    .iter()
                    .map(|v| v.variable_definitions())
                    .flatten()
                    .filter_map(|v| Some(v.variable()?.name()?.text().to_string()))
                    .collect();
                assert_eq!(
                    variables.as_slice(),
                    ["graph_id".to_string(), "variant".to_string()]
                );
            }
        }
    }
}
