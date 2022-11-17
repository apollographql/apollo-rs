#![allow(unused)]

use crate::ApolloDiagnostic;
use apollo_parser::ast;
use apollo_parser::ast::Definition;
use apollo_parser::SyntaxTree;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

// This would be a Salsa input
pub struct Document {
    ast: ast::Document,

    // TODO: other kinds of definitions
    object_types: NamedItems<ObjectTypeDefinition>,
    // ...
}

pub struct Builder<'a> {
    diagnostics: Option<&'a mut Vec<ApolloDiagnostic>>,
    object_types: NamedItems<ObjectTypeDefinition>,
    // ...
}

type Name = String;

struct NamedItems<T> {
    items: Vec<T>,
    indices: HashMap<Name, usize>,
}

pub struct ObjectTypeDefinition {
    // TODO
}

impl Document {
    // This part is push-oriented, Salsa (pull-oriented) is only used after it returns
    pub fn new(ast: ast::Document, diagnostics: Option<&mut Vec<ApolloDiagnostic>>) -> Self {
        let mut builder = Builder {
            diagnostics,
            object_types: NamedItems::new(),
        };
        for def in ast.definitions() {
            let _ = builder.definition(def);
        }
        Document {
            ast,
            object_types: builder.object_types,
            // ...
        }
    }

    // This is not a Salsa query since it’s about as fast as Salsa’s cache
    pub fn object_type_by_name(&self, name: &str) -> Option<&ObjectTypeDefinition> {
        self.object_types.find(name)
    }
}

impl Builder<'_> {
    fn definition(&mut self, ast: ast::Definition) -> Result<(), ()> {
        match ast {
            Definition::OperationDefinition(_) => todo!(),
            Definition::FragmentDefinition(_) => todo!(),
            Definition::DirectiveDefinition(_) => todo!(),
            Definition::SchemaDefinition(_) => todo!(),
            Definition::ScalarTypeDefinition(_) => todo!(),
            Definition::ObjectTypeDefinition(def) => self.object_type_definition(def),
            Definition::InterfaceTypeDefinition(_) => todo!(),
            Definition::UnionTypeDefinition(_) => todo!(),
            Definition::EnumTypeDefinition(_) => todo!(),
            Definition::InputObjectTypeDefinition(_) => todo!(),
            Definition::SchemaExtension(_) => todo!(),
            Definition::ScalarTypeExtension(_) => todo!(),
            Definition::ObjectTypeExtension(_) => todo!(),
            Definition::InterfaceTypeExtension(_) => todo!(),
            Definition::UnionTypeExtension(_) => todo!(),
            Definition::EnumTypeExtension(_) => todo!(),
            Definition::InputObjectTypeExtension(_) => todo!(),
        }
    }

    fn name(&mut self, ast: Option<ast::Name>) -> Result<String, ()> {
        if let Some(name) = ast {
            Ok(name.text().to_string())
        } else {
            if let Some(diagnostics) = &mut self.diagnostics {
                diagnostics.push(ApolloDiagnostic::MissingIdent(todo!()));
            }
            Err(())
        }
    }

    fn object_type_definition(&mut self, ast: ast::ObjectTypeDefinition) -> Result<(), ()> {
        let name = self.name(ast.name())?;
        let new = ObjectTypeDefinition {};
        self.object_types.add(name, new).map_err(|old| {
            if let Some(diagnostics) = &mut self.diagnostics {
                diagnostics.push(ApolloDiagnostic::UniqueDefinition(todo!()));
            }
        })
    }
}

impl<T> NamedItems<T> {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            indices: HashMap::new(),
        }
    }

    fn find(&self, name: &str) -> Option<&T> {
        self.indices.get(name).map(|&index| &self.items[index])
    }

    /// Returns `Err` and doesn’t add if the name is already used
    fn add(&mut self, name: String, item: T) -> Result<(), &T> {
        let next_index = self.items.len();
        match self.indices.entry(name) {
            Entry::Vacant(v) => {
                v.insert(next_index);
                self.items.push(item);
                Ok(())
            }
            Entry::Occupied(o) => Err(&self.items[*o.get()]),
        }
    }
}
