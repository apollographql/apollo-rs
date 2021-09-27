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
