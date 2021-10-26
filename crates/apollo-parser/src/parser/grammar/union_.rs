use crate::{
    parser::grammar::{description, directive, document::is_definition, name, ty},
    Parser, SyntaxKind, TokenKind, S, T,
};

/// See: https://spec.graphql.org/draft/#UnionTypeDefinition
///
/// *UnionTypeDefinition*:
///     Description<sub>opt</sub> **union** Name Directives<sub>\[Const\] opt</sub> UnionDefMemberTypes<sub>opt</sub>
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

/// See: https://spec.graphql.org/draft/#UnionTypeExtension
///
/// *UnionTypeExtension*:
///     **extend** **union** Name Directives<sub>\[Const\] opt</sub> UnionDefMemberTypes
///     **extend** **union** Name Directives<sub>\[Const\]</sub>
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

/// See: https://spec.graphql.org/draft/#UnionMemberTypes
///
/// *UnionMemberTypes*:
///     **=** **|**<sub>opt</sub> NamedType
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
                    union_member_type(p, true);
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
    use crate::ast;

    #[test]
    fn union_member_types() {
        let input = "union SearchResult = Photo | Person | Cat | Dog";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert!(ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let ast::Definition::UnionTypeDefinition(union_type) = def {
                assert_eq!(union_type.name().unwrap().text(), "SearchResult");
                for union_member in union_type.union_member_types().unwrap().named_types() {
                    println!("{}", union_member.name().unwrap().text()); // Photo Person Cat Dog
                }
            }
        }
    }
}
