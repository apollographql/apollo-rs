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
        // Extensions can add new `implements` clauses, but must not
        // duplicate prior ones, form a cycle, or overwrite a field
        // signature the type already commits to. `additional_implements`
        // enforces all three given the existing signatures.
        let existing_field_signatures = field_signatures_for(&self.interface_type_defs, &name);
        let interfaces = self.additional_implements(&existing_field_signatures, Some(&name))?;
        let exclude_fields: IndexSet<Name> = existing_field_signatures
            .keys()
            .map(|k| Name::new(k.clone()))
            .collect();
        let fields_def = self.fields_definition(&exclude_fields)?;
        let directives = self.directives(DirectiveLocation::Interface)?;

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

    /// Reconcile each interface's `implements` clause and fields once
    /// every interface is generated. Topological order means each
    /// child reads only its direct parents — those parents already
    /// hold their own inherited fields, so transitive fields flow
    /// down without re-walking the closure per child.
    pub(crate) fn backfill_inherited_interface_fields(&mut self) {
        let order = self.implements_graph.topo_order_parents_first();
        for name in order {
            let Some(base_idx) = base_def_index(&self.interface_type_defs, &name) else {
                continue;
            };
            self.expand_transitive_interface_implementations(&name, base_idx);

            let parents = self.implements_graph.direct_parents(&name);
            let mut inherited_fields = parent_fields_from_defs(&parents, &self.interface_type_defs);
            // Rewrite child-declared fields to the parent's signature so a
            // child's locally-picked type/args don't drift from the
            // interface contract (covariance + matching args).
            for i in def_indices_with_name(&self.interface_type_defs, &name) {
                for f in self.interface_type_defs[i].fields_def.iter_mut() {
                    if let Some(parent_fdef) = inherited_fields.shift_remove(&f.name.name) {
                        *f = parent_fdef;
                    }
                }
            }
            // Append the parent fields the child never declared.
            self.interface_type_defs[base_idx]
                .fields_def
                .extend(inherited_fields.into_values());
        }
    }

    /// Write `name`'s transitive implementations onto its base def, skipping any already declared by an extension.
    fn expand_transitive_interface_implementations(&mut self, name: &Name, base_idx: usize) {
        let mut all_implemented_interfaces = self.implements_graph.closure(name);
        // Closure includes `name`, but we don't want to write `interface X implements X`
        all_implemented_interfaces.shift_remove(name);

        let interfaces_declared_by_extensions: IndexSet<Name> = self
            .interface_type_defs
            .iter()
            .filter(|i| i.extend && &i.name == name)
            .flat_map(|i| i.interfaces.iter().cloned())
            .collect();
        let interfaces_to_add = all_implemented_interfaces
            .into_iter()
            .filter(|p| !interfaces_declared_by_extensions.contains(p));
        self.interface_type_defs[base_idx]
            .interfaces
            .extend(interfaces_to_add);
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

/// Current (first-wins) union of every parent's fields, read live from
/// `defs`. Used by the backfill so a child sees its parent's
/// post-backfill signature rather than a stale snapshot.
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
