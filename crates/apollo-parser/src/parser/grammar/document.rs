use crate::parser::grammar::{
    directive, enum_, extensions, fragment, input, interface, object, operation, scalar, schema,
    union_,
};
use crate::{Parser, SyntaxKind};

pub(crate) fn document(p: &mut Parser) {
    let doc = p.start_node(SyntaxKind::DOCUMENT);

    while let Some(node) = p.peek_data() {
        match node.as_str() {
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
            _ => break,
        }
    }

    doc.finish_node();
}
