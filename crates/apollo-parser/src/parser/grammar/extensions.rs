use crate::parser::grammar::{enum_, input, interface, object, scalar, schema, union_};
use crate::Parser;

pub(crate) fn extensions(p: &mut Parser) {
    // we already know the next node is 'extend', check for the node after that
    // to figure out which type system extension to apply.
    match p.peek_data_n(2) {
        Some(node) => match node.as_str() {
            "schema" => schema::schema_extension(p),
            "scalar" => scalar::scalar_type_extension(p),
            "type" => object::object_type_extension(p),
            "interface" => interface::interface_type_extension(p),
            "union" => union_::union_type_extension(p),
            "enum" => enum_::enum_type_extension(p),
            "input" => input::input_object_type_extension(p),
            _ => p.err("A Type System Extension cannot be applied"),
        },
        None => p.err("expected a Type System Extension"),
    }
}
