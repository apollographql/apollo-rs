use crate::{
    parser::grammar::{enum_, input, interface, object, scalar, schema, union_},
    Parser,
};

pub(crate) fn extensions(p: &mut Parser) {
    // we already know the next node is 'extend', check for the node after that
    // to figure out which type system extension to apply.
    match p.peek_data_n(2).as_deref() {
        Some("schema") => schema::schema_extension(p),
        Some("scalar") => scalar::scalar_type_extension(p),
        Some("type") => object::object_type_extension(p),
        Some("interface") => interface::interface_type_extension(p),
        Some("union") => union_::union_type_extension(p),
        Some("enum") => enum_::enum_type_extension(p),
        Some("input") => input::input_object_type_extension(p),
        _ => p.err("A Type System Extension cannot be applied"),
    }
}
