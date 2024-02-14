//static Atomic integer
use std::sync::atomic::{AtomicUsize, Ordering};

use arbitrary::Unstructured;

use apollo_compiler::ast::{DirectiveLocation, Name};
use apollo_compiler::NodeStr;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub(crate) trait Common {
    fn unique_name(&self) -> Name {
        Name::new(NodeStr::new(&format!(
            "f{}",
            COUNTER.fetch_add(1, Ordering::SeqCst)
        )))
        .expect("valid name")
    }
    fn arbitrary_name(&self, u: &mut Unstructured) -> arbitrary::Result<Name> {
        loop {
            let s: String = u.arbitrary()?;
            let idx = s
                .char_indices()
                .nth(10)
                .map(|(s, _c)| s)
                .unwrap_or_else(|| s.len());
            if let Ok(name) = Name::new(s[..idx].to_string()) {
                return Ok(name);
            }
        }
    }

    fn arbitrary_node_str(&self, u: &mut Unstructured) -> arbitrary::Result<NodeStr> {
        let s: String = u.arbitrary()?;
        let idx = s
            .char_indices()
            .nth(10)
            .map(|(s, _c)| s)
            .unwrap_or_else(|| s.len());
        Ok(NodeStr::new(&s[..idx]))
    }

    fn arbitrary_directive_locations(
        &self,
        u: &mut Unstructured,
    ) -> arbitrary::Result<Vec<DirectiveLocation>> {
        let mut locations = Vec::new();
        for _ in 0..u.int_in_range(1..=5)? {
            locations.push(
                u.choose(&[
                    DirectiveLocation::Query,
                    DirectiveLocation::Mutation,
                    DirectiveLocation::Subscription,
                    DirectiveLocation::Field,
                    DirectiveLocation::FragmentDefinition,
                    DirectiveLocation::FragmentSpread,
                    DirectiveLocation::InlineFragment,
                    DirectiveLocation::Schema,
                    DirectiveLocation::Scalar,
                    DirectiveLocation::Object,
                    DirectiveLocation::FieldDefinition,
                    DirectiveLocation::ArgumentDefinition,
                    DirectiveLocation::Interface,
                    DirectiveLocation::Union,
                    DirectiveLocation::Enum,
                    DirectiveLocation::EnumValue,
                    DirectiveLocation::InputObject,
                    DirectiveLocation::InputFieldDefinition,
                ])?
                .clone(),
            );
        }
        Ok(locations)
    }
}
