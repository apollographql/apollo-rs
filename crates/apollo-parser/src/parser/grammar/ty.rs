use crate::parser::grammar::name;
use crate::Parser;
use crate::SyntaxKind;
use crate::Token;
use crate::TokenKind;
use crate::S;
use crate::T;

/// See: https://spec.graphql.org/October2021/#InputValueDefinition
///
/// *Type*:
///     NamedType
///     ListType
///         **[** Type **]**
///     NonNullType
///         NamedType **!**
///         ListType **!**
//
// NOTE(lrlna): Because Type cannot be parsed in a typical LR fashion, the
// following parsing rule does not follow the same pattern as all other parsing
// rules in this library. The parent node type is determined based on what its
// last possible NonNullType.
//
// To make this work, we first collect all types in a double ended queue, and
// unwrap them once the last possible child has been parsed. Nodes are then
// created in the processing stage of this parsing rule.
pub(crate) fn ty(p: &mut Parser) {
    match parse(p) {
        Ok(_) => (),
        Err(Some(token)) => p.err_at_token(&token, "expected a type"),
        Err(None) => p.err("expected a type"),
    }
}

/// Returns the type on success, or the TokenKind that caused an error.
///
/// When errors occur deeper inside nested types like lists, this function
/// pushes errors *inside* the list to the parser, and returns an Ok() with
/// an incomplete type.
fn parse<'a>(p: &mut Parser<'a>) -> Result<(), Option<Token<'a>>> {
    let checkpoint = p.checkpoint_node();
    match p.peek() {
        Some(T!['[']) => {
            let _guard = p.start_node(SyntaxKind::LIST_TYPE);
            p.bump(S!['[']);

            if p.recursion_limit.check_and_increment() {
                p.limit_err("parser recursion limit reached");
                return Ok(()); // TODO: is this right?
            }
            let result = parse(p);
            p.recursion_limit.decrement();

            if let Err(Some(token)) = result {
                // TODO(@goto-bus-stop) ideally the span here would point to the entire list
                // type, so both opening and closing brackets `[]`.
                p.err_at_token(&token, "expected item type");
            }
            p.expect(T![']'], S![']']);
        }
        Some(TokenKind::Name) => {
            let _guard = p.start_node(SyntaxKind::NAMED_TYPE);
            let _name_node_guard = p.start_node(SyntaxKind::NAME);

            let token = p.pop();
            name::validate_name(token.data(), p);
            p.push_token(SyntaxKind::IDENT, token);
        }
        Some(_) => return Err(Some(p.pop())),
        None => return Err(None),
    };

    // There may be whitespace inside a list node or between the type and the non-null `!`.
    p.skip_ignored();

    // Deal with nullable types
    if let Some(T![!]) = p.peek() {
        let _guard = checkpoint.wrap_node(SyntaxKind::NON_NULL_TYPE);

        p.eat(S![!]);
    }

    // Handle post-node commas, whitespace, comments
    // TODO(@goto-bus-stop) This should maybe be done further up the parse tree? the type node is
    // parsed completely at this point.
    p.skip_ignored();

    Ok(())
}

/// See: https://spec.graphql.org/October2021/#NamedType
///
/// *NamedType*:
///     Name
pub(crate) fn named_type(p: &mut Parser) {
    // TODO(@goto-bus-stop) can we make this error instead if no name is found?
    if let Some(TokenKind::Name) = p.peek() {
        let _g = p.start_node(SyntaxKind::NAMED_TYPE);
        name::name(p);
    }
}

#[cfg(test)]
mod test {
    use crate::cst;
    use crate::cst::CstNode;
    use crate::Parser;

    #[test]
    fn it_parses_nested_wrapped_types_in_op_def_and_returns_matching_stringified_doc() {
        let mutation = r#"
mutation MyMutation($custId: [Int!]!) {
  myMutation(custId: $custId)
}"#;
        let parser = Parser::new(mutation);
        let cst = parser.parse();
        assert!(cst.errors.is_empty());

        let doc = cst.document();
        assert_eq!(&mutation, &doc.source_string());

        for definition in doc.definitions() {
            if let cst::Definition::OperationDefinition(op_type) = definition {
                for var in op_type
                    .variable_definitions()
                    .unwrap()
                    .variable_definitions()
                {
                    if let cst::Type::NamedType(name) = var.ty().unwrap() {
                        assert_eq!(name.source_string(), "[Int!]!")
                    }
                }
            }
        }
    }

    #[test]
    fn stringified_cst_matches_input_with_deeply_nested_wrapped_types() {
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
        let cst = parser.parse();

        let doc = cst.document();
        assert_eq!(&mutation, &doc.source_string());
    }

    #[test]
    fn stringified_cst_matches_input_with_deeply_nested_wrapped_types_with_commas() {
        let mutation = r#"
mutation MyMutation($a: Int, $b: [Int], $c: String!, $d: [Int!]!,

    $e: String,
    $f: [String],
    $g: String!,
    $h: [String!]!,
) {
  myMutation(custId: $a)
}"#;
        let parser = Parser::new(mutation);
        let cst = parser.parse();

        let doc = cst.document();
        assert_eq!(&mutation, &doc.source_string());
    }
}
