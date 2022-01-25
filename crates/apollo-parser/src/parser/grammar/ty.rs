use std::collections::VecDeque;

use crate::{parser::grammar::name, Parser, SyntaxKind, Token, TokenKind, S, T};

/// See: https://spec.graphql.org/October2021/#InputValueDefinition
///
/// *Type*:
///     NamedType
///     ListType
///         **[** Type **]**
///     NonNullType
///         NamedType **!**
///         ListType **!**

// NOTE(lrlna): Because Type cannot be parsed in a typical LR fashion, the
// following parsing rule does not follow the same pattern as all other parsing
// rules in this library. The parent node type is determined based on what its
// last possible NonNullType.
//
// To make this work, we first collect all types in a double ended queue, and
// unwrap them once the last possible child has been parsed. Nodes are then
// created in the processing stage of this parsing rule.
pub(crate) fn ty(p: &mut Parser) {
    let mut types = parse(p);

    process(&mut types, p);

    return;

    fn parse(p: &mut Parser) -> VecDeque<(SyntaxKind, Token)> {
        let token = p.pop();
        let mut types = match token.kind() {
            T!['['] => {
                let mut types = parse(p);
                types.push_front((S!['['], token));
                if let Some(T![']']) = p.peek() {
                    types.push_back((S![']'], p.pop()));
                }

                types
            }
            TokenKind::Name => {
                let mut types = VecDeque::new();
                types.push_back((SyntaxKind::NAMED_TYPE, token));

                types
            }
            // TODO(@lrlna): this should not panic
            token => panic!("unexpected token, {:?}", token),
        };

        if let Some(T![!]) = p.peek() {
            types.push_front((SyntaxKind::NON_NULL_TYPE, p.pop()));
        }

        // deal with ignored tokens
        if let Some(TokenKind::Whitespace) = p.peek() {
            types.push_back((SyntaxKind::WHITESPACE, p.pop()));
        }

        types
    }

    fn process(types: &mut VecDeque<(SyntaxKind, Token)>, p: &mut Parser) {
        dbg!(&types);
        match types.pop_front() {
            Some((kind @ S!['['], token)) => {
                let _list_g = p.start_node(SyntaxKind::LIST_TYPE);
                p.push_ast(kind, token);
                process(types, p);
                while let Some((_kind @ S![']'], _t)) | Some((_kind @ SyntaxKind::WHITESPACE, _t)) =
                    peek(types)
                {
                    process(types, p);
                }
            }
            Some((kind @ SyntaxKind::NON_NULL_TYPE, token)) => {
                let _non_null_g = p.start_node(kind);
                process(types, p);
                p.push_ast(S![!], token);
                while let Some((_kind @ SyntaxKind::WHITESPACE, _token)) = peek(types) {
                    process(types, p);
                }
            }
            // Cannot use `name::name` or `named_type` function here as we
            // cannot bump from this function. Instead, the process function has
            // already popped Tokens off the token vec, and we are simply adding
            // to the AST.
            Some((SyntaxKind::NAMED_TYPE, token)) => {
                let named_g = p.start_node(SyntaxKind::NAMED_TYPE);
                let name_g = p.start_node(SyntaxKind::NAME);
                name::validate_name(token.data().to_string(), p);
                p.push_ast(SyntaxKind::IDENT, token);

                while let Some((_kind @ SyntaxKind::WHITESPACE, _token)) = peek(types) {
                    process(types, p);
                }

                name_g.finish_node();
                named_g.finish_node();
            }
            Some((SyntaxKind::WHITESPACE, token)) => p.push_ast(SyntaxKind::WHITESPACE, token),
            Some((kind @ S![']'], token)) => {
                p.push_ast(kind, token);
            }
            _ => p.err("Internal apollo-parser error: unexpected when creating a Type"),
        }
    }
}

/// See: https://spec.graphql.org/October2021/#NamedType
///
/// *NamedType*:
///     Name
pub(crate) fn named_type(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::NAMED_TYPE);
    name::name(p);
}

fn peek<T>(target: &VecDeque<T>) -> Option<&T> {
    match target.len() {
        0 => None,
        len => target.get(len - 1),
    }
}

#[cfg(test)]
mod test {
    use crate::{ast, Parser};

    #[test]
    fn it_parses_nested_wrapped_types_in_op_def_and_returns_matching_stringified_doc() {
        let mutation = r#"
mutation MyMutation($custId: [Int!]!) {
  myMutation(custId: $custId)
}"#;
        let parser = Parser::new(mutation);
        let ast = parser.parse();
        assert!(ast.errors.is_empty());

        let doc = ast.document();
        assert_eq!(&mutation, &doc.to_string());

        for definition in doc.definitions() {
            if let ast::Definition::OperationDefinition(op_type) = definition {
                for var in op_type
                    .variable_definitions()
                    .unwrap()
                    .variable_definitions()
                {
                    if let ast::Type::NamedType(name) = var.ty().unwrap() {
                        assert_eq!(name.to_string(), "[Int!]!")
                    }
                }
            }
        }
    }

    #[test]
    fn stringified_ast_matches_input_with_deeply_nested_wrapped_types() {
        let mutation = r#"
mutation MyMutation($a: Int $b: [Int] $c: String! $d: [Int!]!

    $e: String
    $f: [String]
    $g: String!
    $h: [String!]!
) {
  myMutation(custId: $a)
}"#;
        let parser = Parser::new(mutation);
        let ast = parser.parse();

        let doc = ast.document();
        assert_eq!(&mutation, &doc.to_string());
    }

    #[test]
    fn stringified_ast_matches_input_with_nested_wrapped_types() {
        let mutation = r#"
mutation MyMutation($a: String! ) {
  myMutation(custId: $a)
}"#;
        let parser = Parser::new(mutation);
        let ast = parser.parse();

        let doc = ast.document();
        assert_eq!(&mutation, &doc.to_string());
    }
}
