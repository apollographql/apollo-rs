use crate::{
    parser::grammar::{description, directive, document::is_definition, field, name, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#ObjectTypeDefinition
///
/// *ObjectTypeDefinition*:
///     Description<sub>opt</sub> **type** Name ImplementsInterfaces<sub>opt</sub> Directives<sub>\[Const\] opt</sub> FieldsDefinition<sub>opt</sub>
pub(crate) fn object_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("type") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::type_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a name"),
    }

    if let Some(TokenKind::Name) = p.peek() {
        if p.peek_data().unwrap() == "implements" {
            implements_interfaces(p);
        } else {
            p.err("unexpected Name");
        }
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T!['{']) = p.peek() {
        field::fields_definition(p);
    }
}

/// See: https://spec.graphql.org/draft/#ObjectTypeExtension
///
/// *ObjectTypeExtension*:
///     **extend** **type** Name ImplementsInterfaces<sub>opt</sub> Directives<sub>\[Const\] opt</sub> FieldsDefinition
///     **extend** **type** Name ImplementsInterfaces<sub>opt</sub> Directives<sub>\[Const\]</sub>
///     **extend** **type** Name ImplementsInterfaces
pub(crate) fn object_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::OBJECT_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::type_KW);

    // Use this variable to see if any of ImplementsInterfacs, Directives or
    // FieldsDefinitions is provided. If none are present, we push an error.
    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some("implements") = p.peek_data().as_deref() {
        meets_requirements = true;
        implements_interfaces(p);
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p)
    }

    if let Some(T!['{']) = p.peek() {
        meets_requirements = true;
        field::fields_definition(p)
    }

    if !meets_requirements {
        p.err("expected an Implements Interface, Directives or a Fields Definition");
    }
}

/// See: https://spec.graphql.org/draft/#ImplementsInterfaces
///
/// *ImplementsInterfaces*:
///     **implements** **&**<sub>opt</sub> NamedType
///     ImplementsInterfaces **&** NamedType
pub(crate) fn implements_interfaces(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::IMPLEMENTS_INTERFACES);
    p.bump(SyntaxKind::implements_KW);

    implements_interface(p, false);
}

fn implements_interface(p: &mut Parser, is_interfaces: bool) {
    match p.peek() {
        Some(T![&]) => {
            p.bump(S![&]);
            implements_interface(p, is_interfaces)
        }
        Some(TokenKind::Name) => {
            ty::named_type(p);
            if let Some(node) = p.peek_data() {
                if !is_definition(node) {
                    implements_interface(p, true);
                }

                return;
            }
        }
        _ => {
            if !is_interfaces {
                p.err("expected an Object Type Definition");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast;
    use crate::parser::utils;

    #[test]
    fn object_type_definition() {
        let input = "
type Business implements NamedEntity & ValuedEntity & CatEntity {
  name: String
}";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert!(ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let ast::Definition::ObjectTypeDefinition(interface_type) = def {
                assert_eq!(interface_type.name().unwrap().text(), "Business");
                for implements_interfaces in interface_type
                    .implements_interfaces()
                    .unwrap()
                    .named_types()
                {
                    // NamedEntity ValuedEntity CatEntity
                    println!("{}", implements_interfaces.name().unwrap().text());
                }
            }
        }
    }

    #[test]
    fn it_parses_object_type_definition() {
        utils::check_ast(
            "
            \"description of type\"
            type Person implements Human {
              \"\"\"
              description of field
              \"\"\"
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..238
                - WHITESPACE@0..13 "\n            "
                - OBJECT_TYPE_DEFINITION@13..238
                    - DESCRIPTION@13..47
                        - STRING_VALUE@13..34 "\"description of type\""
                        - WHITESPACE@34..47 "\n            "
                    - type_KW@47..51 "type"
                    - WHITESPACE@51..52 " "
                    - NAME@52..59
                        - IDENT@52..58 "Person"
                        - WHITESPACE@58..59 " "
                    - IMPLEMENTS_INTERFACES@59..76
                        - implements_KW@59..69 "implements"
                        - WHITESPACE@69..70 " "
                        - NAMED_TYPE@70..76
                            - NAME@70..76
                                - IDENT@70..75 "Human"
                                - WHITESPACE@75..76 " "
                    - FIELDS_DEFINITION@76..238
                        - L_CURLY@76..77 "{"
                        - WHITESPACE@77..92 "\n              "
                        - FIELD_DEFINITION@92..189
                            - DESCRIPTION@92..162
                                - STRING_VALUE@92..147 "\"\"\n              description of field\n              \"\"\""
                                - WHITESPACE@147..162 "\n              "
                            - NAME@162..166
                                - IDENT@162..166 "name"
                            - COLON@166..167 ":"
                            - WHITESPACE@167..168 " "
                            - TYPE@168..189
                                - WHITESPACE@168..183 "\n              "
                                - NAMED_TYPE@183..189
                                    - NAME@183..189
                                        - IDENT@183..189 "String"
                        - FIELD_DEFINITION@189..212
                            - NAME@189..192
                                - IDENT@189..192 "age"
                            - COLON@192..193 ":"
                            - WHITESPACE@193..194 " "
                            - TYPE@194..212
                                - WHITESPACE@194..209 "\n              "
                                - NAMED_TYPE@209..212
                                    - NAME@209..212
                                        - IDENT@209..212 "Int"
                        - FIELD_DEFINITION@212..237
                            - NAME@212..219
                                - IDENT@212..219 "picture"
                            - COLON@219..220 ":"
                            - WHITESPACE@220..221 " "
                            - TYPE@221..237
                                - WHITESPACE@221..234 "\n            "
                                - NAMED_TYPE@234..237
                                    - NAME@234..237
                                        - IDENT@234..237 "Url"
                        - R_CURLY@237..238 "}"
            "#,
        )
    }

    #[test]
    fn it_parses_extension() {
        utils::check_ast(
            "
            extend type Person implements Human @deprecated {
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..153
                - WHITESPACE@0..13 "\n            "
                - OBJECT_TYPE_EXTENSION@13..153
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - type_KW@20..24 "type"
                    - WHITESPACE@24..25 " "
                    - NAME@25..32
                        - IDENT@25..31 "Person"
                        - WHITESPACE@31..32 " "
                    - IMPLEMENTS_INTERFACES@32..49
                        - implements_KW@32..42 "implements"
                        - WHITESPACE@42..43 " "
                        - NAMED_TYPE@43..49
                            - NAME@43..49
                                - IDENT@43..48 "Human"
                                - WHITESPACE@48..49 " "
                    - DIRECTIVES@49..61
                        - DIRECTIVE@49..61
                            - AT@49..50 "@"
                            - NAME@50..61
                                - IDENT@50..60 "deprecated"
                                - WHITESPACE@60..61 " "
                    - FIELDS_DEFINITION@61..153
                        - L_CURLY@61..62 "{"
                        - WHITESPACE@62..77 "\n              "
                        - FIELD_DEFINITION@77..104
                            - NAME@77..81
                                - IDENT@77..81 "name"
                            - COLON@81..82 ":"
                            - WHITESPACE@82..83 " "
                            - TYPE@83..104
                                - WHITESPACE@83..98 "\n              "
                                - NAMED_TYPE@98..104
                                    - NAME@98..104
                                        - IDENT@98..104 "String"
                        - FIELD_DEFINITION@104..127
                            - NAME@104..107
                                - IDENT@104..107 "age"
                            - COLON@107..108 ":"
                            - WHITESPACE@108..109 " "
                            - TYPE@109..127
                                - WHITESPACE@109..124 "\n              "
                                - NAMED_TYPE@124..127
                                    - NAME@124..127
                                        - IDENT@124..127 "Int"
                        - FIELD_DEFINITION@127..152
                            - NAME@127..134
                                - IDENT@127..134 "picture"
                            - COLON@134..135 ":"
                            - WHITESPACE@135..136 " "
                            - TYPE@136..152
                                - WHITESPACE@136..149 "\n            "
                                - NAMED_TYPE@149..152
                                    - NAME@149..152
                                        - IDENT@149..152 "Url"
                        - R_CURLY@152..153 "}"
            "#,
        )
    }

    #[test]
    fn it_errors_when_extesion_name_is_missing() {
        utils::check_ast(
            "
            extend type {
              name: String
              age: Int
              picture: Url
            }",
            r#"
            - DOCUMENT@0..117
                - WHITESPACE@0..13 "\n            "
                - OBJECT_TYPE_EXTENSION@13..117
                    - extend_KW@13..19 "extend"
                    - WHITESPACE@19..20 " "
                    - type_KW@20..24 "type"
                    - WHITESPACE@24..25 " "
                    - FIELDS_DEFINITION@25..117
                        - L_CURLY@25..26 "{"
                        - WHITESPACE@26..41 "\n              "
                        - FIELD_DEFINITION@41..68
                            - NAME@41..45
                                - IDENT@41..45 "name"
                            - COLON@45..46 ":"
                            - WHITESPACE@46..47 " "
                            - TYPE@47..68
                                - WHITESPACE@47..62 "\n              "
                                - NAMED_TYPE@62..68
                                    - NAME@62..68
                                        - IDENT@62..68 "String"
                        - FIELD_DEFINITION@68..91
                            - NAME@68..71
                                - IDENT@68..71 "age"
                            - COLON@71..72 ":"
                            - WHITESPACE@72..73 " "
                            - TYPE@73..91
                                - WHITESPACE@73..88 "\n              "
                                - NAMED_TYPE@88..91
                                    - NAME@88..91
                                        - IDENT@88..91 "Int"
                        - FIELD_DEFINITION@91..116
                            - NAME@91..98
                                - IDENT@91..98 "picture"
                            - COLON@98..99 ":"
                            - WHITESPACE@99..100 " "
                            - TYPE@100..116
                                - WHITESPACE@100..113 "\n            "
                                - NAMED_TYPE@113..116
                                    - NAME@113..116
                                        - IDENT@113..116 "Url"
                        - R_CURLY@116..117 "}"
            - ERROR@25:26 "expected a Name"
            "#,
        )
    }

    #[test]
    fn it_errors_when_extesion_is_missing_required_syntax() {
        utils::check_ast(
            "extend type Person",
            r#"
            - DOCUMENT@0..18
                - OBJECT_TYPE_EXTENSION@0..18
                    - extend_KW@0..6 "extend"
                    - WHITESPACE@6..7 " "
                    - type_KW@7..11 "type"
                    - WHITESPACE@11..12 " "
                    - NAME@12..18
                        - IDENT@12..18 "Person"
            - ERROR@18:18 "expected an Implements Interface, Directives or a Fields Definition"
            "#,
        )
    }
}
