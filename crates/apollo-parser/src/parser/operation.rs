use crate::parser::{directive, name, selection, variable};
use crate::{format_err, Parser, SyntaxKind, TokenKind};

/// OperationTypeDefinition is used in a SchemaDefinition. Not to be confused
/// with OperationDefinition.
///
/// See: https://spec.graphql.org/June2018/#RootOperationTypeDefinition
///
/// ```txt
/// OperationTypeDefinition
///    OperationType : NamedType
/// ```
pub(crate) fn operation_type_definition(
    parser: &mut Parser,
    is_operation_type: bool,
) -> Result<(), crate::Error> {
    if let Some(TokenKind::Comma) = parser.peek() {
        parser.bump(SyntaxKind::COMMA);
        return operation_type_definition(parser, is_operation_type);
    }

    if let Some(TokenKind::Node) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::OPERATION_TYPE_DEFINITION);
        operation_type(parser)?;
        if let Some(TokenKind::Colon) = parser.peek() {
            parser.bump(SyntaxKind::COLON);
            name::named_type(parser)?;
            if parser.peek().is_some() {
                guard.finish_node();
                return operation_type_definition(parser, true);
            }
            return Ok(());
        } else {
            return format_err!(
                parser.peek_data().unwrap(),
                "Expected Operation Type to have a Name Type, got {}",
                parser.peek_data().unwrap()
            );
        }
    }

    if is_operation_type {
        Ok(())
    } else {
        return format_err!(
            parser.peek_data().unwrap(),
            "Expected Schema Definition to have an Operation Type, got {}",
            parser.peek_data().unwrap()
        );
    }
}

/// See: https://spec.graphql.org/June2018/#OperationDefinition
///
/// ```txt
/// OperationDefinition
///    OperationType Name VariableDefinitions Directives SelectionSet
///    Selection Set (TODO)
/// ```

pub(crate) fn operation_definition(parser: &mut Parser) -> Result<(), crate::Error> {
    let _guard = parser.start_node(SyntaxKind::OPERATION_DEFINITION);
    operation_type(parser)?;
    if let Some(TokenKind::Node) = parser.peek() {
        name::name(parser)?;
    }

    if let Some(TokenKind::LParen) = parser.peek() {
        let guard = parser.start_node(SyntaxKind::VARIABLE_DEFINITIONS);
        parser.bump(SyntaxKind::L_PAREN);
        if let Some(TokenKind::Dollar) = parser.peek() {
            variable::variable_definition(parser, false)?;
        }
        if let Some(TokenKind::RParen) = parser.peek() {
            parser.bump(SyntaxKind::R_PAREN);
            guard.finish_node();
        }
        // TODO @lrlna error: expected a variable definition to follow an opening brace
    }
    if let Some(TokenKind::At) = parser.peek() {
        directive::directives(parser)?;
    }
    // TODO @lrlna: parse SelectionSet
    if let Some(TokenKind::LCurly) = parser.peek() {
        selection::selection_set(parser)?;
    }
    Ok(())
}

/// See: https://spec.graphql.org/June2018/#OperationType
///
/// ```txt
/// OperationType : one of
///    query    mutation    subscription
/// ```
pub(crate) fn operation_type(parser: &mut Parser) -> Result<(), crate::Error> {
    if let Some(node) = parser.peek_data() {
        let _guard = parser.start_node(SyntaxKind::OPERATION_TYPE);
        match node.as_str() {
            "query" => parser.bump(SyntaxKind::query_KW),
            "subscription" => parser.bump(SyntaxKind::subscription_KW),
            "mutation" => parser.bump(SyntaxKind::mutation_KW),
            _ => {
                return format_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data")),
                    "Operation Type must be either 'mutation', 'query' or 'subscription', got {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no further data"))
                )
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    #[test]
    fn smoke_operation_definition() {
        let input = "query myQuery { animal: cat dog { panda { anotherCat @deprecated } } lion }";
        let parser = Parser::new(input);

        println!("{:?}", parser.parse());
    }

    #[test]
    fn it_parses_operation_definition() {
        let input = "query myQuery($var: Boolean, $variable: String) @example(reason: String, isTreat: Boolean) { animal: cat dog { panda { anotherCat @deprecated } } lion }";
        let parser = Parser::new(input);
        let output = parser.parse();

        assert!(output.errors().is_empty());
        assert_eq!(
            format!("{:?}", output),
            indoc! { r#"
            - DOCUMENT@0..131
            - OPERATION_DEFINITION@0..131
            - OPERATION_TYPE@0..5
            - query_KW@0..5 "query"
            - NAME@5..12
            - IDENT@5..12 "myQuery"
            - VARIABLE_DEFINITIONS@12..43
            - L_PAREN@12..13 "("
            - VARIABLE_DEFINITION@13..25
            - VARIABLE@13..17
            - DOLLAR@13..14 "$"
            - NAME@14..17
            - IDENT@14..17 "var"
            - COLON@17..18 ":"
            - TYPE@18..25 "Boolean"
            - COMMA@25..26 ","
            - VARIABLE_DEFINITION@26..42
            - VARIABLE@26..35
            - DOLLAR@26..27 "$"
            - NAME@27..35
            - IDENT@27..35 "variable"
            - COLON@35..36 ":"
            - TYPE@36..42 "String"
            - R_PAREN@42..43 ")"
            - DIRECTIVES@43..82
            - DIRECTIVE@43..82
            - AT@43..44 "@"
            - NAME@44..51
            - IDENT@44..51 "example"
            - ARGUMENTS@51..82
            - L_PAREN@51..52 "("
            - ARGUMENT@52..65
            - NAME@52..58
            - IDENT@52..58 "reason"
            - COLON@58..59 ":"
            - VALUE@59..65 "String"
            - COMMA@65..66 ","
            - ARGUMENT@66..81
            - NAME@66..73
            - IDENT@66..73 "isTreat"
            - COLON@73..74 ":"
            - VALUE@74..81 "Boolean"
            - R_PAREN@81..82 ")"
            - SELECTION_SET@82..131
            - L_CURLY@82..83 "{"
            - SELECTION@83..130
            - FIELD@83..93
            - ALIAS@83..90
            - NAME@83..89
            - IDENT@83..89 "animal"
            - COLON@89..90 ":"
            - NAME@90..93
            - IDENT@90..93 "cat"
            - FIELD@93..126
            - NAME@93..96
            - IDENT@93..96 "dog"
            - SELECTION_SET@96..126
            - L_CURLY@96..97 "{"
            - SELECTION@97..125
            - FIELD@97..125
            - NAME@97..102
            - IDENT@97..102 "panda"
            - SELECTION_SET@102..125
            - L_CURLY@102..103 "{"
            - SELECTION@103..124
            - FIELD@103..124
            - NAME@103..113
            - IDENT@103..113 "anotherCat"
            - DIRECTIVES@113..124
            - DIRECTIVE@113..124
            - AT@113..114 "@"
            - NAME@114..124
            - IDENT@114..124 "deprecated"
            - R_CURLY@124..125 "}"
            - R_CURLY@125..126 "}"
            - FIELD@126..130
            - NAME@126..130
            - IDENT@126..130 "lion"
            - R_CURLY@130..131 "}"
            "# }
        )
    }
}
