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
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#InterfaceTypeDefinition).
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
        let extend = !self.interface_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = if extend {
            let available_itfs: Vec<&Name> = self
                .interface_type_defs
                .iter()
                .filter_map(|itf| if itf.extend { None } else { Some(&itf.name) })
                .collect();
            (*self.u.choose(&available_itfs)?).clone()
        } else {
            self.type_name()?
        };
        // Extensions can declare additional `implements` clauses but must
        // not repeat names the base or earlier extensions already
        // declared, and must not introduce a cycle by picking an
        // interface whose closure points back at this type. Picks are
        // also rejected if they'd force-overwrite a field signature the
        // base def already declared (those overwrites would break
        // anything that already implements this type).
        let (already_declared, existing_signatures) = if extend {
            (
                existing_interfaces_of(&self.interface_type_defs, &name),
                signatures_of(&self.interface_type_defs, &name),
            )
        } else {
            (IndexSet::new(), IndexMap::new())
        };
        let interfaces: IndexSet<Name> = self
            .additional_implements(&already_declared, &existing_signatures, Some(&name))?
            .into_iter()
            .filter(|n| n != &name)
            .collect();
        let fields_def = self.fields_definition(&[])?;
        let directives = self.directives(DirectiveLocation::Interface)?;

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
    /// transitive closure. If we roll `X`, and `X implements Y`, the
    /// caller's `implements` clause is `X & Y` so the validator's
    /// "must also implement" rule is satisfied without a backfill step.
    ///
    /// A candidate is dropped (along with its closure) when its fields
    /// would conflict with parents already picked — generation can
    /// independently roll two interfaces that declare the same field
    /// with incompatible signatures, and a single implementer can never
    /// satisfy both.
    pub fn implements_interfaces(&mut self) -> ArbitraryResult<IndexSet<Name>> {
        self.additional_implements(&IndexSet::new(), &IndexMap::new(), None)
    }

    /// Pick interfaces to add to an existing `implements` clause.
    ///
    /// - `already_declared`: the union of the type's base + earlier
    ///   extension implements clauses, used both to skip duplicates
    ///   and to seed the conflict guard so new picks can't introduce a
    ///   field signature that contradicts what's already declared.
    /// - `existing_signatures`: field signatures the type already
    ///   commits to (across its base + prior extensions). New picks
    ///   that would force-overwrite any of these are dropped so we
    ///   don't change a parent's effective fields out from under
    ///   downstream implementers.
    /// - `forbidden`: the type's own name when an interface extension
    ///   is rolling new picks, so a candidate whose closure points back
    ///   at this type is dropped (no `X implements ... implements X`
    ///   cycles).
    ///
    /// Returns only the newly accepted interfaces (and their transitive
    /// parents).
    pub(crate) fn additional_implements(
        &mut self,
        already_declared: &IndexSet<Name>,
        existing_signatures: &IndexMap<String, FieldDef>,
        forbidden: Option<&Name>,
    ) -> ArbitraryResult<IndexSet<Name>> {
        if self.interface_type_defs.is_empty() {
            return Ok(IndexSet::new());
        }
        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let parent_fields = effective_fields_by_name(&self.interface_type_defs);
        let direct = interface_implements_union(&self.interface_type_defs);

        let mut accepted: IndexSet<Name> = already_declared.clone();
        let mut signatures: IndexMap<String, FieldDef> = existing_signatures.clone();
        for name in already_declared {
            if let Some(fields) = parent_fields.get(name) {
                for (fname, fdef) in fields {
                    signatures
                        .entry(fname.clone())
                        .or_insert_with(|| fdef.clone());
                }
            }
        }

        for _ in 0..num_itf {
            let candidate = self.u.choose(&self.interface_type_defs)?.name.clone();
            try_accept_with_closure(
                &candidate,
                &direct,
                &parent_fields,
                &mut accepted,
                &mut signatures,
                forbidden,
            );
        }
        Ok(accepted
            .into_iter()
            .filter(|n| !already_declared.contains(n))
            .collect())
    }

    /// After every interface (base + extension) has been generated,
    /// reconcile each implementer's `implements` clause and fields.
    /// For each type: expand the `implements` clause to its transitive
    /// closure (so a parent reached via an extension's late picks is
    /// recorded), then copy parent fields onto the implementer. The
    /// `existing_signatures` guard at pick time keeps every parent's
    /// effective field set stable through backfill, so iterating once
    /// against the pre-backfill snapshot is enough — no topological
    /// ordering needed.
    pub(crate) fn backfill_inherited_interface_fields(&mut self) {
        for name in unique_names(&self.interface_type_defs) {
            let Some(base_idx) = base_def_index(&self.interface_type_defs, &name) else {
                continue;
            };
            self.expand_interface_implements_closure(&name, base_idx);

            let parents = existing_interfaces_of(&self.interface_type_defs, &name);
            let mut inherited = parent_fields_now(&parents, &self.interface_type_defs);
            for i in def_indices_with_name(&self.interface_type_defs, &name) {
                for f in self.interface_type_defs[i].fields_def.iter_mut() {
                    if let Some(parent_fdef) = inherited.shift_remove(&f.name.name) {
                        *f = parent_fdef;
                    }
                }
            }
            self.interface_type_defs[base_idx]
                .fields_def
                .extend(inherited.into_values());
        }
    }

    /// Re-expand `name`'s direct implements set (taken across base +
    /// extensions) to its transitive closure and add any newly-required
    /// parents to the base def. Anything an extension already declares
    /// is held out so the same interface is never listed twice.
    fn expand_interface_implements_closure(&mut self, name: &Name, base_idx: usize) {
        let direct = interface_implements_union(&self.interface_type_defs);
        let start = existing_interfaces_of(&self.interface_type_defs, name);
        let closure = transitive_closure(start, &direct, Some(name));
        let extension_declared: IndexSet<Name> = self
            .interface_type_defs
            .iter()
            .filter(|i| i.extend && &i.name == name)
            .flat_map(|i| i.interfaces.iter().cloned())
            .collect();
        let base = &mut self.interface_type_defs[base_idx].interfaces;
        for parent in closure {
            if &parent != name && !extension_declared.contains(&parent) {
                base.insert(parent);
            }
        }
    }
}

/// Defs with a name and field list. Used by the backfill helpers so
/// both `InterfaceTypeDef` and `ObjectTypeDef` can share them.
pub(crate) trait NamedDef {
    fn name(&self) -> &Name;
    fn is_extend(&self) -> bool;
    fn fields(&self) -> &[FieldDef];
}

impl NamedDef for InterfaceTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn is_extend(&self) -> bool {
        self.extend
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

impl NamedDef for crate::ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }
    fn is_extend(&self) -> bool {
        self.extend
    }
    fn fields(&self) -> &[FieldDef] {
        &self.fields_def
    }
}

/// Accept a candidate interface along with its transitive parents, but
/// only if every field signature inside the closure is compatible with
/// the parents already accepted. A conflicting candidate drops out
/// entirely — its closure is never partially accepted — so the
/// implementer's `implements` clause and its inherited field set stay
/// internally consistent.
pub(crate) fn try_accept_with_closure(
    candidate: &Name,
    direct: &HashMap<Name, IndexSet<Name>>,
    parent_fields: &HashMap<Name, IndexMap<String, FieldDef>>,
    accepted: &mut IndexSet<Name>,
    signatures: &mut IndexMap<String, FieldDef>,
    forbidden: Option<&Name>,
) {
    // Reject a candidate whose closure reaches `forbidden` — that
    // would force `forbidden implements ... implements forbidden`.
    if forbidden == Some(candidate) {
        return;
    }
    let closure = transitive_closure(IndexSet::from_iter([candidate.clone()]), direct, None);
    if let Some(f) = forbidden {
        if closure.contains(f) {
            return;
        }
    }

    for name in &closure {
        if accepted.contains(name) {
            continue;
        }
        let Some(fields) = parent_fields.get(name) else {
            continue;
        };
        for (fname, fdef) in fields {
            if let Some(existing) = signatures.get(fname) {
                if existing.ty != fdef.ty
                    || existing.arguments_definition != fdef.arguments_definition
                {
                    return;
                }
            }
        }
    }

    for name in closure {
        if !accepted.insert(name.clone()) {
            continue;
        }
        if let Some(fields) = parent_fields.get(&name) {
            for (fname, fdef) in fields {
                signatures
                    .entry(fname.clone())
                    .or_insert_with(|| fdef.clone());
            }
        }
    }
}

/// Union of every def's fields by type name, merging base + extensions.
/// First occurrence of each field name wins.
pub(crate) fn effective_fields_by_name<T: NamedDef>(
    defs: &[T],
) -> HashMap<Name, IndexMap<String, FieldDef>> {
    let mut out: HashMap<Name, IndexMap<String, FieldDef>> = HashMap::new();
    for def in defs {
        let entry = out.entry(def.name().clone()).or_default();
        for f in def.fields() {
            entry
                .entry(f.name.name.clone())
                .or_insert_with(|| f.clone());
        }
    }
    out
}

/// Distinct names across base + extensions, in first-occurrence order.
pub(crate) fn unique_names<T: NamedDef>(defs: &[T]) -> Vec<Name> {
    let mut seen: IndexSet<Name> = IndexSet::new();
    for d in defs {
        seen.insert(d.name().clone());
    }
    seen.into_iter().collect()
}

/// Index of the base def for `name` (`extend: false`), falling back to
/// the first matching def when no base exists.
pub(crate) fn base_def_index<T: NamedDef>(defs: &[T], name: &Name) -> Option<usize> {
    defs.iter()
        .position(|d| !d.is_extend() && d.name() == name)
        .or_else(|| defs.iter().position(|d| d.name() == name))
}

/// Positions of every def (base + extensions) sharing the given name.
pub(crate) fn def_indices_with_name<T: NamedDef>(defs: &[T], name: &Name) -> Vec<usize> {
    defs.iter()
        .enumerate()
        .filter_map(|(i, d)| (d.name() == name).then_some(i))
        .collect()
}

/// Union of `interfaces` clauses across every interface def sharing
/// `name` (base + extensions). Used both when an extension picks
/// additional implements (so it knows what to avoid) and when the
/// backfill needs the full implements set for the type.
pub(crate) fn existing_interfaces_of(defs: &[InterfaceTypeDef], name: &Name) -> IndexSet<Name> {
    let mut out: IndexSet<Name> = IndexSet::new();
    for def in defs {
        if &def.name == name {
            out.extend(def.interfaces.iter().cloned());
        }
    }
    out
}

/// First-wins union of every field signature across `name`'s base +
/// prior extensions. An extension picking new implements consults this
/// so it doesn't accept a parent whose fields would overwrite anything
/// the type already declares.
pub(crate) fn signatures_of<T: NamedDef>(defs: &[T], name: &Name) -> IndexMap<String, FieldDef> {
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

/// Transitive closure over an implements graph, starting from `start`
/// and walking edges in `direct`. `skip` is omitted from the closure
/// even when it appears as a parent — used by the interface case to
/// avoid pulling a type into its own implements set on cycle.
pub(crate) fn transitive_closure(
    start: IndexSet<Name>,
    direct: &HashMap<Name, IndexSet<Name>>,
    skip: Option<&Name>,
) -> IndexSet<Name> {
    let mut closure = start.clone();
    let mut frontier: Vec<Name> = start.into_iter().collect();
    while let Some(next) = frontier.pop() {
        if let Some(parents) = direct.get(&next) {
            for parent in parents {
                if Some(parent) == skip {
                    continue;
                }
                if closure.insert(parent.clone()) {
                    frontier.push(parent.clone());
                }
            }
        }
    }
    closure
}

/// Direct implements graph: for every interface name, the union of
/// `interfaces` clauses across its base and every extension. A naive
/// `iter().map().collect()` would overwrite on duplicate keys and
/// lose extension-added implements; this combines them properly.
pub(crate) fn interface_implements_union(
    defs: &[InterfaceTypeDef],
) -> HashMap<Name, IndexSet<Name>> {
    let mut out: HashMap<Name, IndexSet<Name>> = HashMap::new();
    for def in defs {
        out.entry(def.name.clone())
            .or_default()
            .extend(def.interfaces.iter().cloned());
    }
    out
}

/// Current (first-wins) union of every parent's fields, read live from
/// `defs`. Used by the backfill so a child sees its parent's
/// post-backfill signature rather than a stale snapshot.
pub(crate) fn parent_fields_now<T: NamedDef>(
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
