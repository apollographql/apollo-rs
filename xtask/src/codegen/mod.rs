mod gen_syntax_kinds;
mod gen_syntax_nodes;

use crate::cst_src::Cardinality;
use crate::cst_src::CstEnumSrc;
use crate::cst_src::CstNodeSrc;
use crate::cst_src::CstSrc;
use crate::cst_src::Field;
use crate::cst_src::KINDS_SRC;
use crate::ensure_file_contents;
use crate::root_path;
use crate::utils::pluralize;
use crate::utils::project_root;
use crate::utils::to_lower_snake_case;
use anyhow::Result;
use clap::Parser;
use gen_syntax_kinds::generate_kinds;
use gen_syntax_nodes::generate_nodes;
use std::collections::BTreeSet;
use ungrammar::Grammar;
use ungrammar::Rule;

#[derive(Debug, Parser)]
pub struct Codegen {}

impl Codegen {
    pub(crate) fn run(&self, _verbose: bool) -> Result<()> {
        let grammar_src = include_str!("../../../graphql.ungram");
        let grammar: Grammar = grammar_src.parse().unwrap();
        let cst = lower(&grammar);

        let syntax_kind_path =
            root_path().join("crates/apollo-parser/src/parser/generated/syntax_kind.rs");
        let syntax_kinds = generate_kinds(KINDS_SRC);
        ensure_file_contents(syntax_kind_path.as_path(), &syntax_kinds?)?;

        let cst_nodes_file = project_root().join("crates/apollo-parser/src/cst/generated/nodes.rs");
        let contents = generate_nodes(KINDS_SRC, &cst)?;
        ensure_file_contents(cst_nodes_file.as_path(), &contents)?;
        Ok(())
    }
}

fn lower(grammar: &Grammar) -> CstSrc {
    let tokens = "Whitespace Comment String ByteString IntNumber FloatNumber"
        .split_ascii_whitespace()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let mut res = CstSrc {
        tokens,
        ..Default::default()
    };

    let nodes = grammar.iter().collect::<Vec<_>>();

    for &node in &nodes {
        let name = grammar[node].name.clone();
        let rule = &grammar[node].rule;
        match lower_enum(grammar, rule) {
            Some(variants) => {
                let enum_src = CstEnumSrc {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    variants,
                };
                res.enums.push(enum_src);
            }
            None => {
                let mut fields = Vec::new();
                lower_rule(&mut fields, grammar, None, rule);
                res.nodes.push(CstNodeSrc {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    fields,
                });
            }
        }
    }

    deduplicate_fields(&mut res);
    extract_enums(&mut res);
    extract_struct_traits(&mut res);
    extract_enum_traits(&mut res);
    res
}

fn lower_enum(grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    let alternatives = match rule {
        Rule::Alt(it) => it,
        _ => return None,
    };
    let mut variants = Vec::new();
    for alternative in alternatives {
        match alternative {
            Rule::Node(it) => variants.push(grammar[*it].name.clone()),
            Rule::Token(it) if grammar[*it].name == ";" => (),
            _ => return None,
        }
    }
    Some(variants)
}

fn lower_rule(acc: &mut Vec<Field>, grammar: &Grammar, label: Option<&String>, rule: &Rule) {
    if lower_comma_list(acc, grammar, label, rule) {
        return;
    }

    match rule {
        Rule::Node(node) => {
            let ty = grammar[*node].name.clone();
            let name = label.cloned().unwrap_or_else(|| to_lower_snake_case(&ty));
            let field = Field::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            };
            acc.push(field);
        }
        Rule::Token(token) => {
            assert!(label.is_none());
            let mut name = grammar[*token].name.clone();
            if name != "int_number" && name != "string" {
                if "[]{}()".contains(&name) {
                    name = format!("'{name}'");
                }
                let field = Field::Token(name);
                acc.push(field);
            }
        }
        Rule::Rep(inner) => {
            if let Rule::Node(node) = &**inner {
                let ty = grammar[*node].name.clone();
                let name = label
                    .cloned()
                    .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
                let field = Field::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Many,
                };
                acc.push(field);
                return;
            }
            todo!("{:?}", rule)
        }
        Rule::Labeled { label: l, rule } => {
            assert!(label.is_none());
            let manually_implemented = matches!(
                l.as_str(),
                "lhs"
                    | "rhs"
                    | "then_branch"
                    | "else_branch"
                    | "start"
                    | "end"
                    | "op"
                    | "index"
                    | "base"
                    | "value"
                    | "trait"
                    | "self_ty"
            );
            if manually_implemented {
                return;
            }
            lower_rule(acc, grammar, Some(l), rule);
        }
        Rule::Seq(rules) | Rule::Alt(rules) => {
            for rule in rules {
                lower_rule(acc, grammar, label, rule)
            }
        }
        Rule::Opt(rule) => lower_rule(acc, grammar, label, rule),
    }
}

// (T (',' T)* ','?)
fn lower_comma_list(
    acc: &mut Vec<Field>,
    grammar: &Grammar,
    label: Option<&String>,
    rule: &Rule,
) -> bool {
    let rule = match rule {
        Rule::Seq(it) => it,
        _ => return false,
    };
    let (node, repeat, trailing_comma) = match rule.as_slice() {
        [Rule::Node(node), Rule::Rep(repeat), Rule::Opt(trailing_comma)] => {
            (node, repeat, trailing_comma)
        }
        _ => return false,
    };
    let repeat = match &**repeat {
        Rule::Seq(it) => it,
        _ => return false,
    };
    match repeat.as_slice() {
        [comma, Rule::Node(n)] if comma == &**trailing_comma && n == node => (),
        _ => return false,
    }
    let ty = grammar[*node].name.clone();
    let name = label
        .cloned()
        .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
    let field = Field::Node {
        name,
        ty,
        cardinality: Cardinality::Many,
    };
    acc.push(field);
    true
}

fn deduplicate_fields(cst: &mut CstSrc) {
    for node in &mut cst.nodes {
        let mut i = 0;
        'outer: while i < node.fields.len() {
            for j in 0..i {
                let f1 = &node.fields[i];
                let f2 = &node.fields[j];
                if f1 == f2 {
                    node.fields.remove(i);
                    continue 'outer;
                }
            }
            i += 1;
        }
    }
}

fn extract_enums(cst: &mut CstSrc) {
    for node in &mut cst.nodes {
        for enm in &cst.enums {
            let mut to_remove = Vec::new();
            for (i, field) in node.fields.iter().enumerate() {
                let ty = field.ty().to_string();
                if enm.variants.iter().any(|it| it == &ty) {
                    to_remove.push(i);
                }
            }
            if to_remove.len() == enm.variants.len() {
                node.remove_field(to_remove);
                let ty = enm.name.clone();
                let name = to_lower_snake_case(&ty);
                node.fields.push(Field::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Optional,
                });
            }
        }
    }
}

fn extract_struct_traits(cst: &mut CstSrc) {
    // TODO @lrlna: add common accessor traits here.
    let traits: &[(&str, &[&str])] = &[];

    for node in &mut cst.nodes {
        for (name, methods) in traits {
            extract_struct_trait(node, name, methods);
        }
    }
}

fn extract_struct_trait(node: &mut CstNodeSrc, trait_name: &str, methods: &[&str]) {
    let mut to_remove = Vec::new();
    for (i, field) in node.fields.iter().enumerate() {
        let method_name = field.method_name().to_string();
        if methods.iter().any(|&it| it == method_name) {
            to_remove.push(i);
        }
    }
    if to_remove.len() == methods.len() {
        node.traits.push(trait_name.to_string());
        node.remove_field(to_remove);
    }
}

fn extract_enum_traits(cst: &mut CstSrc) {
    let enums = cst.enums.clone();
    for enm in &mut cst.enums {
        if enm.name == "Stmt" {
            continue;
        }
        let nodes = &cst.nodes;

        let mut variant_traits = enm.variants.iter().map(|var| {
            nodes
                .iter()
                .find_map(|node| {
                    if &node.name != var {
                        return None;
                    }
                    Some(node.traits.iter().cloned().collect::<BTreeSet<_>>())
                })
                .unwrap_or_else(|| {
                    enums
                        .iter()
                        .find_map(|node| {
                            if &node.name != var {
                                return None;
                            }
                            Some(node.traits.iter().cloned().collect::<BTreeSet<_>>())
                        })
                        .unwrap_or_else(|| {
                            panic!("{}", {
                                &format!(
                                    "Could not find a struct `{}` for enum `{}::{}`",
                                    var, enm.name, var
                                )
                            })
                        })
                })
        });

        let mut enum_traits = match variant_traits.next() {
            Some(it) => it,
            None => continue,
        };
        for traits in variant_traits {
            enum_traits = enum_traits.intersection(&traits).cloned().collect();
        }
        enm.traits = enum_traits.into_iter().collect();
    }
}

impl CstNodeSrc {
    fn remove_field(&mut self, to_remove: Vec<usize>) {
        to_remove.into_iter().rev().for_each(|idx| {
            self.fields.remove(idx);
        });
    }
}
