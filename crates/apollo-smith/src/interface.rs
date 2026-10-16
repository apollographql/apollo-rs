use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::field::FieldDef;
use crate::name::Name;
use crate::DocumentBuilder;
use crate::StackedEntity;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;
use std::collections::HashMap;

/// InterfaceTypeDef is an abstract type where there are common fields declared.
///
/// Any type that implements an interface must define all the fields with names
/// and types exactly matching. The implementations of this interface are
/// explicitly listed out in possibleTypes.
///
/// *InterfaceTypeDefinition*:
///     Description? **interface** Name ImplementsInterfaces? Directives?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/September2025/#InterfaceTypeDefinition).
#[derive(Debug, Clone)]
pub struct InterfaceTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) interfaces: IndexSet<Name>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<InterfaceTypeDef> for ast::Definition {
    fn from(x: InterfaceTypeDef) -> Self {
        if x.extend {
            ast::InterfaceTypeExtension {
                name: x.name.into(),
                implements_interfaces: x.interfaces.into_iter().map(Into::into).collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        } else {
            ast::InterfaceTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                implements_interfaces: x.interfaces.into_iter().map(Into::into).collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        }
    }
}

impl TryFrom<apollo_parser::cst::InterfaceTypeDefinition> for InterfaceTypeDef {
    type Error = crate::FromError;

    fn try_from(
        interface_def: apollo_parser::cst::InterfaceTypeDefinition,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: interface_def.description().map(Description::from),
            directives: interface_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: false,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl TryFrom<apollo_parser::cst::InterfaceTypeExtension> for InterfaceTypeDef {
    type Error = crate::FromError;

    fn try_from(
        interface_def: apollo_parser::cst::InterfaceTypeExtension,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            name: interface_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: interface_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            extend: true,
            fields_def: interface_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            interfaces: interface_def
                .implements_interfaces()
                .map(|itfs| {
                    itfs.named_types()
                        .map(|named_type| named_type.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
        })
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `InterfaceTypeDef`
    pub fn interface_type_definition(&mut self) -> ArbitraryResult<InterfaceTypeDef> {
        // Types that implement an interface already declare its fields, so
        // only interfaces nothing implements yet can gain fields or parents.
        let extendable: Vec<Name> = self
            .interface_type_defs
            .iter()
            .filter(|itf| !itf.extend && !self.is_implemented(&itf.name))
            .map(|itf| itf.name.clone())
            .collect();
        let extend = !extendable.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self.optional_description()?;
        let name = if extend {
            self.u.choose(&extendable)?.clone()
        } else {
            self.type_name()?
        };
        // Extensions can add new `implements` clauses, but must not
        // duplicate prior ones, form a cycle, or overwrite a field
        // signature the type already commits to. `additional_implements`
        // enforces all three given the existing signatures.
        let existing_field_signatures = field_signatures_for(&self.interface_type_defs, &name);
        let interfaces = self.additional_implements(&existing_field_signatures, Some(&name))?;
        let fields_def =
            self.fields_implementing(&name, &interfaces, &existing_field_signatures)?;
        let applied: IndexSet<Name> = self
            .interface_type_defs
            .iter()
            .filter(|def| def.name == name)
            .flat_map(|def| def.directives.keys().cloned())
            .collect();
        let directives = self.type_directives(DirectiveLocation::Interface, &applied)?;

        if extend && directives.is_empty() && fields_def.is_empty() && interfaces.is_empty() {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(InterfaceTypeDef {
            description,
            name,
            fields_def,
            directives,
            extend,
            interfaces,
        })
    }

    /// Pick a random set of interfaces to implement, expanded to its
    /// transitive closure so the validator's "must also implement"
    /// rule is satisfied. A candidate is dropped if its closure would
    /// conflict with parents already picked.
    pub fn implements_interfaces(&mut self) -> ArbitraryResult<IndexSet<Name>> {
        self.additional_implements(&IndexMap::new(), None)
    }

    /// Pick interfaces to add to an existing `implements` clause.
    ///
    /// - `existing_field_signatures`: field signatures the type
    ///   already commits to. A pick whose closure would overwrite any
    ///   of them is rejected.
    /// - `self_name`: the type's own name when set, so a candidate
    ///   whose closure loops back at this type is rejected.
    ///
    /// Returns only the newly accepted interfaces and their transitive
    /// parents.
    pub(crate) fn additional_implements(
        &mut self,
        existing_field_signatures: &IndexMap<String, FieldDef>,
        self_name: Option<&Name>,
    ) -> ArbitraryResult<IndexSet<Name>> {
        if self.interface_type_defs.is_empty() {
            return Ok(IndexSet::new());
        }
        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let fields_by_type_name = fields_from_all_definitions(&self.interface_type_defs);

        let already_implemented_parents = match self_name {
            Some(n) => self.implements_graph.direct_parents(n),
            None => IndexSet::new(),
        };
        let mut accepted = already_implemented_parents.clone();
        // Seed the conflict guard with each existing parent's fields so a
        // new candidate whose closure carries a clashing signature is
        // rejected before it joins `accepted`.
        let mut accumulated_signatures = existing_field_signatures.clone();
        for parent in &already_implemented_parents {
            if let Some(fields) = fields_by_type_name.get(parent) {
                for (fname, fdef) in fields {
                    accumulated_signatures
                        .entry(fname.clone())
                        .or_insert_with(|| fdef.clone());
                }
            }
        }

        for _ in 0..num_itf {
            let candidate = self.u.choose(&self.interface_type_defs)?.name.clone();
            try_accept_candidate(
                &candidate,
                &self.implements_graph,
                &fields_by_type_name,
                &mut accepted,
                &mut accumulated_signatures,
                self_name,
            );
        }

        accepted.retain(|n| !already_implemented_parents.contains(n));
        Ok(accepted)
    }

    /// Fields for a definition that newly implements `interfaces`: copies
    /// of the inherited fields it doesn't already declare, followed by
    /// fields of its own.
    pub(crate) fn fields_implementing(
        &mut self,
        owner: &Name,
        interfaces: &IndexSet<Name>,
        declared: &IndexMap<String, FieldDef>,
    ) -> ArbitraryResult<Vec<FieldDef>> {
        let mut inherited = parent_fields_from_defs(interfaces, &self.interface_type_defs);
        inherited.retain(|name, _| !declared.contains_key(name));
        let exclude_fields: IndexSet<Name> = declared
            .keys()
            .chain(inherited.keys())
            .map(|k| Name::new(k.clone()))
            .collect();
        let own = self.fields_definition_for(owner, &exclude_fields)?;
        Ok(inherited.into_values().chain(own).collect())
    }

    fn is_implemented(&self, name: &Name) -> bool {
        self.interface_type_defs
            .iter()
            .any(|itf| itf.interfaces.contains(name))
            || self
                .object_type_defs
                .iter()
                .any(|obj| obj.implements_interfaces.contains(name))
    }
}

/// Defs with a name and field list, so `InterfaceTypeDef` and
/// `ObjectTypeDef` can share the field helpers.
pub(crate) trait NamedDef {
    fn name(&self) -> &Name;
    fn fields(&self) -> &[FieldDef];
}

impl NamedDef for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

impl NamedDef for crate::ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

/// Try to accept `candidate` (and every interface it transitively
/// implements) into the implementer's `accepted` set.
///
/// Rejects when:
/// - The closure loops back at the implementer (would produce
///   `X implements ... implements X`).
/// - Any field in the closure clashes with a signature already
///   committed to in `accumulated_signatures`.
fn try_accept_candidate(
    candidate: &Name,
    graph: &crate::implements_graph::ImplementsGraph,
    fields_by_type_name: &HashMap<Name, IndexMap<String, FieldDef>>,
    accepted: &mut IndexSet<Name>,
    accumulated_signatures: &mut IndexMap<String, FieldDef>,
    self_name: Option<&Name>,
) {
    let closure = graph.closure(candidate);

    let would_cycle = self_name.is_some_and(|n| closure.contains(n));
    let would_conflict = closure.iter().any(|name| {
        fields_by_type_name
            .get(name)
            .into_iter()
            .flatten()
            .any(|(fname, fdef)| {
                accumulated_signatures.get(fname).is_some_and(|existing| {
                    existing.ty != fdef.ty
                        || existing.arguments_definition != fdef.arguments_definition
                })
            })
    });
    if would_cycle || would_conflict {
        return;
    }

    for name in closure {
        if !accepted.insert(name.clone()) {
            continue;
        }
        for (fname, fdef) in fields_by_type_name.get(&name).into_iter().flatten() {
            accumulated_signatures
                .entry(fname.clone())
                .or_insert_with(|| fdef.clone());
        }
    }
}

/// Union of every def's fields by type name, merging base + extensions.
/// First occurrence of each field name wins.
pub(crate) fn fields_from_all_definitions<T: NamedDef>(
    defs: &[T],
) -> HashMap<Name, IndexMap<String, FieldDef>> {
    unique_names(defs)
        .into_iter()
        .map(|n| {
            let fields = field_signatures_for(defs, &n);
            (n, fields)
        })
        .collect()
}

/// Distinct names across base + extensions, in first-occurrence order.
pub(crate) fn unique_names<T: NamedDef>(defs: &[T]) -> Vec<Name> {
    let mut seen: IndexSet<Name> = IndexSet::new();
    for d in defs {
        seen.insert(d.name().clone());
    }
    seen.into_iter().collect()
}

/// First-wins union of every field signature across `name`'s base +
/// prior extensions. An extension picking new implements consults this
/// so it doesn't accept a parent whose fields would overwrite anything
/// the type already declares.
pub(crate) fn field_signatures_for<T: NamedDef>(
    defs: &[T],
    name: &Name,
) -> IndexMap<String, FieldDef> {
    let mut out: IndexMap<String, FieldDef> = IndexMap::new();
    for def in defs {
        if def.name() == name {
            for f in def.fields() {
                out.entry(f.name.name.clone()).or_insert_with(|| f.clone());
            }
        }
    }
    out
}

/// First-wins union of the fields of every def named in `parents`,
/// including extensions.
pub(crate) fn parent_fields_from_defs<T: NamedDef>(
    parents: &IndexSet<Name>,
    defs: &[T],
) -> IndexMap<String, FieldDef> {
    let mut out: IndexMap<String, FieldDef> = IndexMap::new();
    for parent in parents {
        for def in defs {
            if def.name() == parent {
                for f in def.fields() {
                    out.entry(f.name.name.clone()).or_insert_with(|| f.clone());
                }
            }
        }
    }
    out
}

impl StackedEntity for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::generate;
    use crate::test_support::input;
    use crate::test_support::prop_assert_valid;
    use proptest::prelude::*;

    const SCHEMA: &str = "schema { query: Query }
        directive @d(x: Int, y: [String!]!) on INTERFACE | FIELD_DEFINITION | ARGUMENT_DEFINITION
        type Query { i: I }
        interface I { x: Int }
        interface J implements I { x: Int, y: String }
        type A implements I & J { x: Int, y: String }
        interface K @d(y: []) { z: Int }";

    proptest! {
        #[test]
        fn generates_valid_definitions(data in input()) {
            if let Some(doc) = generate(SCHEMA, &data, |b| b.interface_type_definition().map(Some)) {
                prop_assert_valid(&doc)?;
            }
        }
    }
}
