use crate::{
    parser::grammar::{argument, description, input, name},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#DirectiveDefinition
///
/// *DirectiveDefinition*:
///     Description<sub>opt</sub> **directive @** Name ArgumentsDefinition<sub>opt</sub> **repeatable**<sub>opt</sub> **on** DirectiveLocations
pub(crate) fn directive_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DIRECTIVE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("directive") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::directive_KW);
    }

    match p.peek() {
        Some(T![@]) => p.bump(S![@]),
        _ => p.err("expected @ symbol"),
    }
    name::name(p);

    if let Some(T!['(']) = p.peek() {
        let _g = p.start_node(SyntaxKind::ARGUMENTS_DEFINITION);
        p.bump(S!['(']);
        input::input_value_definition(p, false);
        p.expect(T![')'], S![')']);
    }

    if let Some(node) = p.peek_data() {
        if node.as_str() == "repeatable" {
            p.bump(SyntaxKind::repeatable_KW);
        }
    }

    if let Some(node) = p.peek_data() {
        match node.as_str() {
            "on" => p.bump(SyntaxKind::on_KW),
            _ => p.err("expected Directive Locations"),
        }
    }

    if let Some(TokenKind::Name | T![|]) = p.peek() {
        let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATIONS);
        directive_locations(p, false);
    } else {
        p.err("expected valid Directive Location");
    }
}

/// See: https://spec.graphql.org/draft/#DirectiveLocations
///
/// *DirectiveLocations*:
///     DirectiveLocations **|** DirectiveLocation
///     **|**<sub>opt</sub> DirectiveLocation
pub(crate) fn directive_locations(p: &mut Parser, is_location: bool) {
    if let Some(T![|]) = p.peek() {
        p.bump(S![|]);
        directive_locations(p, is_location)
    }

    if let Some(TokenKind::Name) = p.peek() {
        let loc = p.peek_data().unwrap();
        match loc.as_str() {
            "QUERY" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::QUERY_KW);
            }
            "MUTATION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::MUTATION_KW);
            }
            "SUBSCRIPTION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SUBSCRIPTION_KW);
            }
            "FIELD" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FIELD_KW);
            }
            "FRAGMENT_DEFINITION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FRAGMENT_DEFINITION_KW);
            }
            "FRAGMENT_SPREAD" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FRAGMENT_DEFINITION_KW);
            }
            "INLINE_FRAGMENT" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INLINE_FRAGMENT_KW);
            }
            "VARIABLE_DEFINITION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::VARIABLE_DEFINITION_KW);
            }
            "SCHEMA" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SCHEMA_KW);
            }
            "SCALAR" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::SCALAR_KW);
            }
            "OBJECT" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::OBJECT_KW);
            }
            "FIELD_DEFINITION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::FIELD_DEFINITION_KW);
            }
            "ARGUMENT_DEFINITION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ARGUMENT_DEFINITION_KW);
            }
            "INTERFACE" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INTERFACE_KW);
            }
            "UNION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::UNION_KW);
            }
            "ENUM" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ENUM_KW);
            }
            "ENUM_VALUE" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::ENUM_VALUE_KW);
            }
            "INPUT_OBJECT" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INPUT_OBJECT_KW);
            }
            "INPUT_FIELD_DEFINITION" => {
                let _g = p.start_node(SyntaxKind::DIRECTIVE_LOCATION);
                p.bump(SyntaxKind::INPUT_FIELD_DEFINITION_KW);
            }
            _ => {
                if !is_location {
                    p.err("expected valid Directive Location");
                }
                return;
            }
        }
        if p.peek_data().is_some() {
            return directive_locations(p, true);
        }
    }
    if !is_location {
        p.err("expected Directive Locations");
    }
}

/// See: https://spec.graphql.org/draft/#Directive
///
/// *Directive*<sub>\[Const\]</sub>:
///     **@** Name Arguments<sub>\[Const\] opt</sub>
pub(crate) fn directive(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DIRECTIVE);

    p.expect(T![@], S![@]);
    name::name(p);

    if let Some(T!['(']) = p.peek() {
        argument::arguments(p);
    }
}

/// See: https://spec.graphql.org/draft/#Directives
///
/// *Directives*<sub>\[Const\]</sub>:
///     Directive<sub>\[Const\] list</sub>
pub(crate) fn directives(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::DIRECTIVES);
    while let Some(T![@]) = p.peek() {
        directive(p);
    }
}
