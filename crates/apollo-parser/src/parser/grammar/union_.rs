#![allow(clippy::needless_return)]

use crate::{
    parser::grammar::{description, directive, document::is_definition, name, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/October2021/#UnionTypeDefinition
///
/// *UnionTypeDefinition*:
///     Description? **union** Name Directives? UnionDefMemberTypes?
pub(crate) fn union_type_definition(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_TYPE_DEFINITION);

    if let Some(TokenKind::StringValue) = p.peek() {
        description::description(p);
    }

    if let Some("union") = p.peek_data().as_deref() {
        p.bump(SyntaxKind::union_KW);
    }

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        directive::directives(p);
    }

    if let Some(T![=]) = p.peek() {
        union_member_types(p);
    }
}

/// See: https://spec.graphql.org/October2021/#UnionTypeExtension
///
/// *UnionTypeExtension*:
///     **extend** **union** Name Directives? UnionDefMemberTypes
///     **extend** **union** Name Directives
pub(crate) fn union_type_extension(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_TYPE_EXTENSION);
    p.bump(SyntaxKind::extend_KW);
    p.bump(SyntaxKind::union_KW);

    let mut meets_requirements = false;

    match p.peek() {
        Some(TokenKind::Name) => name::name(p),
        _ => p.err("expected a Name"),
    }

    if let Some(T![@]) = p.peek() {
        meets_requirements = true;
        directive::directives(p);
    }

    if let Some(T![=]) = p.peek() {
        meets_requirements = true;
        union_member_types(p);
    }

    if !meets_requirements {
        p.err("expected Directives or Union Member Types");
    }
}

/// See: https://spec.graphql.org/October2021/#UnionMemberTypes
///
/// *UnionMemberTypes*:
///     **=** **|**? NamedType
///     UnionMemberTypes **|** NamedType
pub(crate) fn union_member_types(p: &mut Parser) {
    let _g = p.start_node(SyntaxKind::UNION_MEMBER_TYPES);
    p.bump(S![=]);

    union_member_type(p, false);
}

fn union_member_type(p: &mut Parser, is_union: bool) {
    match p.peek() {
        Some(T![|]) => {
            p.bump(S![|]);
            union_member_type(p, is_union);
        }
        Some(TokenKind::Name) => {
            ty::named_type(p);
            if let Some(node) = p.peek_data() {
                if !is_definition(node) {
                    // TODO: use a loop instead of recursion
                    if p.recursion_limit.check_and_increment() {
                        p.limit_err("parser recursion limit reached");
                        return;
                    }
                    union_member_type(p, true);
                    p.recursion_limit.decrement();
                }

                return;
            }
        }
        _ => {
            if !is_union {
                p.err("expected Union Member Types");
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::cst;

    #[test]
    fn union_member_types() {
        let input = "union SearchResult = Photo | Person | Cat | Dog";
        let parser = Parser::new(input);
        let cst = parser.parse();
        assert_eq!(0, cst.errors().len());

        let doc = cst.document();

        for def in doc.definitions() {
            if let cst::Definition::UnionTypeDefinition(union_type) = def {
                assert_eq!(union_type.name().unwrap().text(), "SearchResult");
                for union_member in union_type.union_member_types().unwrap().named_types() {
                    println!("{}", union_member.name().unwrap().text()); // Photo Person Cat Dog
                }
            }
        }
    }
}
