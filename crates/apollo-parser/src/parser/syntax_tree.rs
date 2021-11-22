use std::fmt;

use rowan::GreenNodeBuilder;

use crate::{ast::Document, Error, SyntaxElement, SyntaxKind};

use super::GraphQLLanguage;

/// An AST generated by the parser.
pub struct SyntaxTree {
    pub(crate) ast: rowan::SyntaxNode<GraphQLLanguage>,
    pub(crate) errors: Vec<crate::Error>,
}

impl SyntaxTree {
    /// Get a reference to the syntax tree's errors.
    pub fn errors(&self) -> &[crate::Error] {
        self.errors.as_ref()
    }

    /// Return the root typed `Document` node.
    pub fn document(self) -> Document {
        Document { syntax: self.ast }
    }
}

impl fmt::Debug for SyntaxTree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn print(f: &mut fmt::Formatter<'_>, indent: usize, element: SyntaxElement) -> fmt::Result {
            let kind: SyntaxKind = element.kind();
            write!(f, "{:indent$}", "", indent = indent)?;
            match element {
                rowan::NodeOrToken::Node(node) => {
                    writeln!(f, "- {:?}@{:?}", kind, node.text_range())?;
                    for child in node.children_with_tokens() {
                        print(f, indent + 4, child)?;
                    }
                    Ok(())
                }

                rowan::NodeOrToken::Token(token) => {
                    writeln!(
                        f,
                        "- {:?}@{:?} {:?}",
                        kind,
                        token.text_range(),
                        token.text()
                    )
                }
            }
        }

        fn print_err(f: &mut fmt::Formatter<'_>, errors: Vec<Error>) -> fmt::Result {
            for err in errors {
                writeln!(f, "- {:?}", err)?;
            }

            write!(f, "")
        }

        print(f, 0, self.ast.clone().into())?;
        print_err(f, self.errors.clone())
    }
}

#[derive(Debug)]
pub(crate) struct SyntaxTreeBuilder {
    builder: GreenNodeBuilder<'static>,
}

impl SyntaxTreeBuilder {
    /// Create a new instance of `SyntaxBuilder`.
    pub(crate) fn new() -> Self {
        Self {
            builder: GreenNodeBuilder::new(),
        }
    }

    /// Start new node and make it current.
    pub(crate) fn start_node(&mut self, kind: SyntaxKind) {
        self.builder.start_node(rowan::SyntaxKind(kind as u16));
    }

    /// Finish current branch and restore previous branch as current.
    pub(crate) fn finish_node(&mut self) {
        self.builder.finish_node();
    }

    /// Adds new token to the current branch.
    pub(crate) fn token(&mut self, kind: SyntaxKind, text: &str) {
        self.builder.token(rowan::SyntaxKind(kind as u16), text);
    }

    pub(crate) fn finish(self, errors: Vec<Error>) -> SyntaxTree {
        SyntaxTree {
            ast: rowan::SyntaxNode::new_root(self.builder.finish()),
            // TODO: keep the errors in the builder rather than pass it in here?
            errors,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast::Definition;
    use crate::Parser;

    #[test]
    fn schema_with_errors() {
        let input = "directive @example(isTreat: Boolean, treatKind: * String) on";
        let parser = Parser::new(input);
        let ast = parser.parse();

        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::DirectiveDefinition(directive) = def {
                assert_eq!(directive.name().unwrap().text(), "example");
            }
        }
    }

    #[test]
    fn directive_name() {
        let input = "directive @example(isTreat: Boolean, treatKind: String) on FIELD | MUTATION";
        let parser = Parser::new(input);
        let ast = parser.parse();
        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::DirectiveDefinition(directive) = def {
                assert_eq!(directive.name().unwrap().text(), "example");
            }
        }
    }

    #[test]
    fn object_type_definition() {
        let input = "
        type ProductDimension {
          size: String
          weight: Float @tag(name: \"hi from inventory value type field\")
        }
        ";
        let parser = Parser::new(input);
        let ast = parser.parse();
        assert!(ast.errors().is_empty());

        let doc = ast.document();

        for def in doc.definitions() {
            if let Definition::ObjectTypeDefinition(object_type) = def {
                assert_eq!(object_type.name().unwrap().text(), "ProductDimension");
                for field_def in object_type.fields_definition().unwrap().field_definitions() {
                    println!("{}", field_def.name().unwrap().text()); // size weight
                }
            }
        }
    }
}
