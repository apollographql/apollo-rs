use crate::{
    ast_src::{KindsSrc, KINDS_SRC},
    ensure_file_contents, reformat, root_path,
};
use anyhow::Result;
use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};
use structopt::StructOpt;
use ungrammar::Grammar;

#[derive(Debug, StructOpt)]
pub struct Codegen {}

impl Codegen {
    pub(crate) fn run(&self, _verbose: bool) -> Result<()> {
        let grammar_src = include_str!("../../../../graphql.ungram");
        let _grammar: Grammar = grammar_src.parse().unwrap();
        let syntax_kind_path =
            root_path().join("crates/apollo-parser/src/syntax_kind/generated.rs");
        let syntax_kinds = Self::generate_syntax_kinds(KINDS_SRC);
        ensure_file_contents(syntax_kind_path.as_path(), &syntax_kinds?)?;
        // dbg!(&syntax_kinds.unwrap_or("unable to generate syntax kinds".to_string()));
        Ok(())
    }

    fn generate_syntax_kinds(kinds: KindsSrc<'_>) -> Result<String> {
        let (single_byte_tokens_values, single_byte_tokens): (Vec<_>, Vec<_>) = kinds
            .punct
            .iter()
            .filter(|(token, _name)| token.len() == 1)
            .map(|(token, name)| (token.chars().next().unwrap(), format_ident!("{}", name)))
            .unzip();

        let punctuation_values = kinds.punct.iter().map(|(token, _name)| {
            if "{}[]()".contains(token) {
                let c = token.chars().next().unwrap();
                quote! { #c }
            } else {
                let cs = token.chars().map(|c| Punct::new(c, Spacing::Joint));
                quote! { #(#cs)* }
            }
        });
        let punctuation = kinds
            .punct
            .iter()
            .map(|(_token, name)| format_ident!("{}", name))
            .collect::<Vec<_>>();

        let full_keywords_values = &kinds.keywords;
        let full_keywords = full_keywords_values
            .iter()
            .map(|kw| format_ident!("{}_KW", kw));

        let all_keywords_values = kinds.keywords.iter().collect::<Vec<_>>();
        let all_keywords_idents = all_keywords_values.iter().map(|kw| format_ident!("{}", kw));
        let all_keywords = all_keywords_values
            .iter()
            .map(|name| format_ident!("{}_KW", name))
            .collect::<Vec<_>>();

        let literals = kinds
            .literals
            .iter()
            .map(|name| format_ident!("{}", name))
            .collect::<Vec<_>>();

        let tokens = kinds
            .tokens
            .iter()
            .map(|name| format_ident!("{}", name))
            .collect::<Vec<_>>();

        let nodes = kinds
            .nodes
            .iter()
            .map(|name| format_ident!("{}", name))
            .collect::<Vec<_>>();

        let ast = quote! {
            #![allow(bad_style, missing_docs, unreachable_pub)]
            #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
            #[repr(u16)]
            pub enum SyntaxKind {
                // Technical SyntaxKinds: they appear temporally during parsing,
                // but never end up in the final tree
                #[doc(hidden)]
                TOMBSTONE,
                #[doc(hidden)]
                EOF,
                #(#punctuation,)*
                #(#all_keywords,)*
                #(#literals,)*
                #(#tokens,)*
                #(#nodes,)*

                // Technical kind so that we can cast from u16 safely
                #[doc(hidden)]
                __LAST,
            }
            use self::SyntaxKind::*;

            impl SyntaxKind {
                pub fn is_keyword(self) -> bool {
                    match self {
                        #(#all_keywords)|* => true,
                        _ => false,
                    }
                }

                pub fn is_punct(self) -> bool {
                    match self {
                        #(#punctuation)|* => true,
                        _ => false,
                    }
                }

                pub fn is_literal(self) -> bool {
                    match self {
                        #(#literals)|* => true,
                        _ => false,
                    }
                }

                pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
                    let kw = match ident {
                        #(#full_keywords_values => #full_keywords,)*
                        _ => return None,
                    };
                    Some(kw)
                }

                pub fn from_char(c: char) -> Option<SyntaxKind> {
                    let tok = match c {
                        #(#single_byte_tokens_values => #single_byte_tokens,)*
                        _ => return None,
                    };
                    Some(tok)
                }
            }

            #[macro_export]
            macro_rules! T {
                #([#punctuation_values] => { $crate::SyntaxKind::#punctuation };)*
                #([#all_keywords_idents] => { $crate::SyntaxKind::#all_keywords };)*
                [lifetime_ident] => { $crate::SyntaxKind::LIFETIME_IDENT };
                [ident] => { $crate::SyntaxKind::IDENT };
                [shebang] => { $crate::SyntaxKind::SHEBANG };
            }
        };

        reformat(&ast.to_string())
    }
}
