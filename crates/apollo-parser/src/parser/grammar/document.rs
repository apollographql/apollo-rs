use crate::parser::grammar::{
    directive, enum_, extensions, fragment, input, interface, object, operation, scalar, schema,
    union_,
};
use crate::{Parser, SyntaxKind, TokenKind};

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
        _ => p.err("expected definition"),
    }
}

#[cfg(test)]
mod test {
    use crate::parser::utils;
    use indoc::indoc;

    #[test]
    fn smoke_subgraph_test() {
        utils::check_ast(
            indoc! { r#"
"""
A simple GraphQL schema which is well described.
"""
schema {
  query: Query
}

"""
Root type for all your query operations
"""
type Query {
  """
  Translates a string from a given language into a different language.
  """
  translate(
    "The original language that `text` is provided in."
    fromLanguage: Language

    "The translated language to be returned."
    toLanguage: Language

    "The text to be translated."
    text: String
  ): String
}

"""
The set of languages supported by `translate`.
"""
enum Language {
  "English"
  EN

  "French"
  FR

  "Chinese"
  CH
}"#},
            r#"
- DOCUMENT@0..582
    - SCHEMA_DEFINITION@0..83
        - DESCRIPTION@0..56
            - STRING_VALUE@0..55 "\"\"\nA simple GraphQL schema which is well described.\n\"\"\""
            - WHITESPACE@55..56 "\n"
        - schema_KW@56..62 "schema"
        - WHITESPACE@62..63 " "
        - L_CURLY@63..64 "{"
        - WHITESPACE@64..67 "\n  "
        - OPERATION_TYPE_DEFINITION@67..80
            - OPERATION_TYPE@67..72
                - query_KW@67..72 "query"
            - COLON@72..73 ":"
            - WHITESPACE@73..74 " "
            - NAMED_TYPE@74..80
                - NAME@74..80
                    - IDENT@74..79 "Query"
                    - WHITESPACE@79..80 "\n"
        - R_CURLY@80..81 "}"
        - WHITESPACE@81..83 "\n\n"
    - OBJECT_TYPE_DEFINITION@83..459
        - DESCRIPTION@83..130
            - STRING_VALUE@83..129 "\"\"\nRoot type for all your query operations\n\"\"\""
            - WHITESPACE@129..130 "\n"
        - type_KW@130..134 "type"
        - WHITESPACE@134..135 " "
        - NAME@135..141
            - IDENT@135..140 "Query"
            - WHITESPACE@140..141 " "
        - FIELDS_DEFINITION@141..459
            - L_CURLY@141..142 "{"
            - WHITESPACE@142..145 "\n  "
            - FIELD_DEFINITION@145..456
                - DESCRIPTION@145..227
                    - STRING_VALUE@145..224 "\"\"\n  Translates a string from a given language into a different language.\n  \"\"\""
                    - WHITESPACE@224..227 "\n  "
                - NAME@227..236
                    - IDENT@227..236 "translate"
                - ARGUMENTS@236..447
                    - L_PAREN@236..237 "("
                    - WHITESPACE@237..242 "\n    "
                    - INPUT_VALUE_DEFINITION@242..326
                        - DESCRIPTION@242..298
                            - STRING_VALUE@242..293 "\"The original language that `text` is provided in.\""
                            - WHITESPACE@293..298 "\n    "
                        - NAME@298..310
                            - IDENT@298..310 "fromLanguage"
                        - COLON@310..311 ":"
                        - WHITESPACE@311..312 " "
                        - TYPE@312..326
                            - WHITESPACE@312..318 "\n\n    "
                            - NAMED_TYPE@318..326
                                - NAME@318..326
                                    - IDENT@318..326 "Language"
                    - INPUT_VALUE_DEFINITION@326..398
                        - DESCRIPTION@326..372
                            - STRING_VALUE@326..367 "\"The translated language to be returned.\""
                            - WHITESPACE@367..372 "\n    "
                        - NAME@372..382
                            - IDENT@372..382 "toLanguage"
                        - COLON@382..383 ":"
                        - WHITESPACE@383..384 " "
                        - TYPE@384..398
                            - WHITESPACE@384..390 "\n\n    "
                            - NAMED_TYPE@390..398
                                - NAME@390..398
                                    - IDENT@390..398 "Language"
                    - INPUT_VALUE_DEFINITION@398..446
                        - DESCRIPTION@398..431
                            - STRING_VALUE@398..426 "\"The text to be translated.\""
                            - WHITESPACE@426..431 "\n    "
                        - NAME@431..435
                            - IDENT@431..435 "text"
                        - COLON@435..436 ":"
                        - WHITESPACE@436..437 " "
                        - TYPE@437..446
                            - WHITESPACE@437..440 "\n  "
                            - NAMED_TYPE@440..446
                                - NAME@440..446
                                    - IDENT@440..446 "String"
                    - R_PAREN@446..447 ")"
                - COLON@447..448 ":"
                - WHITESPACE@448..449 " "
                - TYPE@449..456
                    - WHITESPACE@449..450 "\n"
                    - NAMED_TYPE@450..456
                        - NAME@450..456
                            - IDENT@450..456 "String"
            - R_CURLY@456..457 "}"
            - WHITESPACE@457..459 "\n\n"
    - ENUM_TYPE_DEFINITION@459..582
        - DESCRIPTION@459..513
            - STRING_VALUE@459..512 "\"\"\nThe set of languages supported by `translate`.\n\"\"\""
            - WHITESPACE@512..513 "\n"
        - enum_KW@513..517 "enum"
        - WHITESPACE@517..518 " "
        - NAME@518..527
            - IDENT@518..526 "Language"
            - WHITESPACE@526..527 " "
        - ENUM_VALUES_DEFINITION@527..582
            - L_CURLY@527..528 "{"
            - WHITESPACE@528..531 "\n  "
            - ENUM_VALUE_DEFINITION@531..549
                - DESCRIPTION@531..543
                    - STRING_VALUE@531..540 "\"English\""
                    - WHITESPACE@540..543 "\n  "
                - ENUM_VALUE@543..549
                    - NAME@543..549
                        - IDENT@543..545 "EN"
                        - WHITESPACE@545..549 "\n\n  "
            - ENUM_VALUE_DEFINITION@549..566
                - DESCRIPTION@549..560
                    - STRING_VALUE@549..557 "\"French\""
                    - WHITESPACE@557..560 "\n  "
                - ENUM_VALUE@560..566
                    - NAME@560..566
                        - IDENT@560..562 "FR"
                        - WHITESPACE@562..566 "\n\n  "
            - ENUM_VALUE_DEFINITION@566..581
                - DESCRIPTION@566..578
                    - STRING_VALUE@566..575 "\"Chinese\""
                    - WHITESPACE@575..578 "\n  "
                - ENUM_VALUE@578..581
                    - NAME@578..581
                        - IDENT@578..580 "CH"
                        - WHITESPACE@580..581 "\n"
            - R_CURLY@581..582 "}"
            "#,
        );
    }
}
