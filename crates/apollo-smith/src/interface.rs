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
        // Only the base def declares `implements`; extensions leave it
        // empty so we don't emit `implements X` twice for the same type.
        let interfaces: IndexSet<Name> = if extend {
            IndexSet::new()
        } else {
            self.implements_interfaces()?
                .into_iter()
                .filter(|n| n != &name)
                .collect()
        };
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
        if self.interface_type_defs.is_empty() {
            return Ok(IndexSet::new());
        }
        let num_itf = self
            .u
            .int_in_range(0..=(self.interface_type_defs.len() - 1))?;
        let parent_fields = effective_fields_by_name(&self.interface_type_defs);
        let direct: std::collections::HashMap<Name, IndexSet<Name>> = self
            .interface_type_defs
            .iter()
            .map(|i| (i.name.clone(), i.interfaces.clone()))
            .collect();

        let mut accepted: IndexSet<Name> = IndexSet::new();
        let mut signatures: IndexMap<String, FieldDef> = IndexMap::new();
        for _ in 0..num_itf {
            let candidate = self.u.choose(&self.interface_type_defs)?.name.clone();
            try_accept_with_closure(
                &candidate,
                &direct,
                &parent_fields,
                &mut accepted,
                &mut signatures,
            );
        }
        Ok(accepted)
    }

    /// After every interface (base + extension) has been generated,
    /// reconcile each implementer's fields against its declared parents.
    /// A field whose name matches a parent's is overwritten with the
    /// parent's signature so the return type and arguments match.
    /// Parent fields the implementer doesn't declare yet are appended
    /// to its base def.
    ///
    /// Interfaces are processed in `unique_names` (generation) order so
    /// each parent has already been reconciled by the time a child
    /// reads its fields — `parent_fields_now` always sees the
    /// post-backfill signature.
    pub(crate) fn backfill_inherited_interface_fields(&mut self) {
        for name in unique_names(&self.interface_type_defs) {
            let Some(base_idx) = base_def_index(&self.interface_type_defs, &name) else {
                continue;
            };
            let parents = self.interface_type_defs[base_idx].interfaces.clone();
            let mut inherited = parent_fields_now(&parents, &self.interface_type_defs);
            let def_indices = def_indices_with_name(&self.interface_type_defs, &name);
            for &i in &def_indices {
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
    direct: &std::collections::HashMap<Name, IndexSet<Name>>,
    parent_fields: &std::collections::HashMap<Name, IndexMap<String, FieldDef>>,
    accepted: &mut IndexSet<Name>,
    signatures: &mut IndexMap<String, FieldDef>,
) {
    // Walk transitive parents (skipping the candidate itself, so
    // self-cycles can't loop forever).
    let mut closure: IndexSet<Name> = IndexSet::from_iter([candidate.clone()]);
    let mut frontier: Vec<Name> = vec![candidate.clone()];
    while let Some(next) = frontier.pop() {
        if let Some(parents) = direct.get(&next) {
            for parent in parents {
                if parent != candidate && closure.insert(parent.clone()) {
                    frontier.push(parent.clone());
                }
            }
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
) -> std::collections::HashMap<Name, IndexMap<String, FieldDef>> {
    use std::collections::HashMap;
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
