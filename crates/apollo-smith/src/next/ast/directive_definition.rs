use std::ops::Deref;

use apollo_compiler::ast::{
    Argument, Directive, DirectiveDefinition, DirectiveList, DirectiveLocation,
};
use apollo_compiler::{Node, Schema};

use crate::next::unstructured::Unstructured;

pub(crate) struct LocationFilter<I>(I, DirectiveLocation);

impl<'a, T> Iterator for LocationFilter<T>
where
    T: Iterator<Item = &'a Node<DirectiveDefinition>>,
{
    type Item = &'a Node<DirectiveDefinition>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.find(|d| d.locations.contains(&self.1))
    }
}

pub(crate) trait DirectiveDefinitionIterExt {
    fn with_location<'a>(self, location: DirectiveLocation) -> LocationFilter<Self>
    where
        Self: Iterator<Item = &'a Node<DirectiveDefinition>> + Sized;

    fn try_collect<'a>(
        self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> arbitrary::Result<DirectiveList>
    where
        Self: Iterator<Item = &'a Node<DirectiveDefinition>> + Sized;
}

impl<I: ?Sized> DirectiveDefinitionIterExt for I {
    fn with_location<'a>(self, location: DirectiveLocation) -> LocationFilter<Self>
    where
        I: Iterator<Item = &'a Node<DirectiveDefinition>>,
        Self: Sized,
    {
        LocationFilter(self, location)
    }

    fn try_collect<'a>(
        mut self,
        u: &mut Unstructured,
        schema: &Schema,
    ) -> arbitrary::Result<DirectiveList>
    where
        Self: Iterator<Item = &'a Node<DirectiveDefinition>> + Sized,
    {
        let mut directives = DirectiveList::new();
        while let Some(d) = self.next() {
            let mut arguments = Vec::new();
            for arg in &d.arguments {
                if arg.is_required() || u.arbitrary()? {
                    arguments.push(Node::new(Argument {
                        name: arg.name.clone(),
                        value: Node::new(u.arbitrary_value(schema, arg.ty.deref())?),
                    }))
                }
            }

            directives.push(Node::new(Directive {
                name: d.name.clone(),
                arguments,
            }))
        }
        Ok(directives)
    }
}
