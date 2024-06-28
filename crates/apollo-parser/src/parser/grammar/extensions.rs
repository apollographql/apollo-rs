use crate::parser::grammar::enum_;
use crate::parser::grammar::input;
use crate::parser::grammar::interface;
use crate::parser::grammar::object;
use crate::parser::grammar::scalar;
use crate::parser::grammar::schema;
use crate::parser::grammar::union_;
use crate::Parser;

pub(crate) fn extensions(p: &mut Parser) {
    // we already know the next node is 'extend', check for the node after that
    // to figure out which type system extension to apply.
    match p.peek_data_n(2) {
        Some("schema") => schema::schema_extension(p),
        Some("scalar") => scalar::scalar_type_extension(p),
        Some("type") => object::object_type_extension(p),
        Some("interface") => interface::interface_type_extension(p),
        Some("union") => union_::union_type_extension(p),
        Some("enum") => enum_::enum_type_extension(p),
        Some("input") => input::input_object_type_extension(p),
        _ => p.err_and_pop("Invalid Type System Extension. This extension cannot be applied."),
    }
}

#[cfg(test)]

mod test {
    use crate::cst;
    use crate::Parser;

    #[test]
    fn it_queries_graphql_extensions() {
        let gql = r#"
extend schema {
    mutation: MyMutationType
}
extend scalar UUID @specifiedBy(url: "https://tools.ietf.org/html/rfc4122")
extend type Business implements NamedEntity
extend interface NamedEntity {
    name: String
}
extend union SearchResult = Pet
extend enum Pet {
    GuineaPig
    Cat
}
extend input First @include(if: "first")
        "#;

        let parser = Parser::new(gql);
        let cst = parser.parse();

        assert!(cst.errors().len() == 0);

        let doc = cst.document();

        for definition in doc.definitions() {
            match definition {
                cst::Definition::SchemaExtension(schema_ext) => {
                    let root_operation_type: Vec<String> = schema_ext
                        .root_operation_type_definitions()
                        .filter_map(|def| Some(def.named_type()?.name()?.text().to_string()))
                        .collect();
                    assert_eq!(
                        root_operation_type.as_slice(),
                        ["MyMutationType".to_string()]
                    )
                }
                cst::Definition::ScalarTypeExtension(scalar_ext) => {
                    assert_eq!(
                        scalar_ext
                            .name()
                            .expect("Cannot get scalar type extension name.")
                            .text()
                            .as_ref(),
                        "UUID"
                    );
                }
                cst::Definition::ObjectTypeExtension(obj_ext) => {
                    assert_eq!(
                        obj_ext
                            .name()
                            .expect("Cannot get object type extension name.")
                            .text()
                            .as_ref(),
                        "Business"
                    );
                }
                cst::Definition::InterfaceTypeExtension(interface_ext) => {
                    assert_eq!(
                        interface_ext
                            .name()
                            .expect("Cannot get interface type extension name.")
                            .text()
                            .as_ref(),
                        "NamedEntity"
                    );
                }
                cst::Definition::UnionTypeExtension(union_ext) => {
                    assert_eq!(
                        union_ext
                            .name()
                            .expect("Cannot get union type extension name.")
                            .text()
                            .as_ref(),
                        "SearchResult"
                    );
                }
                cst::Definition::EnumTypeExtension(enum_ext) => {
                    assert_eq!(
                        enum_ext
                            .name()
                            .expect("Cannot get enum type extension name.")
                            .text()
                            .as_ref(),
                        "Pet"
                    );
                }
                cst::Definition::InputObjectTypeExtension(input_object_ext) => {
                    assert_eq!(
                        input_object_ext
                            .name()
                            .expect("Cannot get input object type extension name.")
                            .text()
                            .as_ref(),
                        "First"
                    );
                }
                _ => unimplemented!(),
            }
        }
    }

    #[test]
    fn it_reports_an_error_for_invalid_type_system_extension() {
        let gql = r#"
extend Cat
        "#;

        let parser = Parser::new(gql);
        let cst = parser.parse();

        assert!(cst.errors().len() == 2);
        assert_eq!(cst.document().definitions().count(), 0);
    }

    #[test]
    fn it_continuous_parsing_when_an_invalid_extension_is_given() {
        let gql = r#"
extend Cat

extend interface NamedEntity {
    name: String
}
        "#;

        let parser = Parser::new(gql);
        let cst = parser.parse();

        assert!(cst.errors().len() == 2);
        assert_eq!(cst.document().definitions().count(), 1);
    }
}
