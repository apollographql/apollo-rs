use crate::parser::{enum_, input, interface, object, scalar, schema, union_};
use crate::{create_err, Parser};

pub(crate) fn extensions(parser: &mut Parser) {
    // we already know the next node is 'extend', check for the node after that
    // to figure out which type system extension to apply.
    match parser.peek_data_n(2) {
        Some(node) => match node.as_str() {
            "schema" => schema::schema_extension(parser),
            "scalar" => scalar::scalar_type_extension(parser),
            "type" => object::object_type_extension(parser),
            // "interface" => interface::interface_type_definition(parser),
            // "union" => union_::union_type_definition(parser),
            // "enum" => enum_::enum_type_definition(parser),
            // "input" => input::input_object_type_definition(parser),
            _ => {
                parser.push_err(create_err!(
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no data")),
                    "A Type System Extension cannot be applied to {}",
                    parser
                        .peek_data()
                        .unwrap_or_else(|| String::from("no data")),
                ));
            }
        },
        None => {
            parser.push_err(create_err!(
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
                "Expect a Type System Extension to follow 'extend' keyword, got {}",
                parser
                    .peek_data()
                    .unwrap_or_else(|| String::from("no further data")),
            ));
        }
    }
}
