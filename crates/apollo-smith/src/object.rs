use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::field::FieldDef;
use crate::interface::base_def_index;
use crate::interface::def_indices_with_name;
use crate::interface::interface_implements_union;
use crate::interface::parent_fields_now;
use crate::interface::signatures_of;
use crate::interface::transitive_closure;
use crate::interface::unique_names;
use crate::name::Name;
use crate::DocumentBuilder;
use crate::StackedEntity;
use apollo_compiler::ast;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexMap;
use indexmap::IndexSet;

/// Object types represent concrete instantiations of sets of fields.
///
/// The introspection types (e.g. `__Type`, `__Field`, etc) are examples of
/// objects.
///
/// *ObjectTypeDefinition*:
///     Description? **type** Name ImplementsInterfaces? Directives? FieldsDefinition?
///
/// Detailed documentation can be found in [GraphQL spec](https://spec.graphql.org/October2021/#sec-Object).
#[derive(Debug, Clone)]
pub struct ObjectTypeDef {
    pub(crate) description: Option<Description>,
    pub(crate) name: Name,
    pub(crate) implements_interfaces: IndexSet<Name>,
    pub(crate) directives: IndexMap<Name, Directive>,
    pub(crate) fields_def: Vec<FieldDef>,
    pub(crate) extend: bool,
}

impl From<ObjectTypeDef> for ast::Definition {
    fn from(x: ObjectTypeDef) -> Self {
        if x.extend {
            ast::ObjectTypeExtension {
                name: x.name.into(),
                implements_interfaces: x
                    .implements_interfaces
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                directives: Directive::to_ast(x.directives),
                fields: x
                    .fields_def
                    .into_iter()
                    .map(|x| Node::new(x.into()))
                    .collect(),
            }
            .into()
        } else {
            ast::ObjectTypeDefinition {
                description: x.description.map(Into::into),
                name: x.name.into(),
                implements_interfaces: x
                    .implements_interfaces
                    .into_iter()
                    .map(Into::into)
                    .collect(),
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

impl TryFrom<apollo_parser::cst::ObjectTypeDefinition> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::cst::ObjectTypeDefinition) -> Result<Self, Self::Error> {
        Ok(Self {
            name: object_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: object_def.description().map(Description::from),
            directives: object_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            implements_interfaces: object_def
                .implements_interfaces()
                .map(|impl_int| {
                    impl_int
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
            extend: false,
            fields_def: object_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl TryFrom<apollo_parser::cst::ObjectTypeExtension> for ObjectTypeDef {
    type Error = crate::FromError;

    fn try_from(object_def: apollo_parser::cst::ObjectTypeExtension) -> Result<Self, Self::Error> {
        Ok(Self {
            name: object_def
                .name()
                .expect("object type definition must have a name")
                .into(),
            description: None,
            directives: object_def
                .directives()
                .map(Directive::convert_directives)
                .transpose()?
                .unwrap_or_default(),
            implements_interfaces: object_def
                .implements_interfaces()
                .map(|impl_int| {
                    impl_int
                        .named_types()
                        .map(|n| n.name().unwrap().into())
                        .collect()
                })
                .unwrap_or_default(),
            extend: true,
            fields_def: object_def
                .fields_definition()
                .expect("object type definition must have fields definition")
                .field_definitions()
                .map(FieldDef::try_from)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

impl DocumentBuilder<'_> {
    /// Create an arbitrary `ObjectTypeDef`
    pub fn object_type_definition(&mut self) -> ArbitraryResult<ObjectTypeDef> {
        let extend = !self.object_type_defs.is_empty() && self.u.arbitrary().unwrap_or(false);
        let description = self
            .u
            .arbitrary()
            .unwrap_or(false)
            .then(|| self.description())
            .transpose()?;
        let name = if extend {
            let available_objects: Vec<&Name> = self
                .object_type_defs
                .iter()
                .filter_map(|object| {
                    if object.extend {
                        None
                    } else {
                        Some(&object.name)
                    }
                })
                .collect();
            (*self.u.choose(&available_objects)?).clone()
        } else {
            self.type_name()?
        };

        // Extensions can declare additional `implements` clauses but
        // must not repeat names already declared by the base or earlier
        // extensions, and must not force-overwrite a field signature
        // the type already commits to. Objects can't appear in the
        // interface graph, so no cycle protection is needed.
        let (already_declared, existing_signatures) = if extend {
            (
                existing_implements_of(&self.object_type_defs, &name),
                signatures_of(&self.object_type_defs, &name),
            )
        } else {
            (IndexSet::new(), IndexMap::new())
        };
        let implements_interfaces =
            self.additional_implements(&already_declared, &existing_signatures, None)?;
        let fields_def = self.fields_definition(&[])?;
        let directives = self.directives(DirectiveLocation::Object)?;

        Ok(ObjectTypeDef {
            description,
            directives,
            implements_interfaces,
            name,
            fields_def,
            extend,
        })
    }

    /// After every interface and object has been generated, reconcile
    /// each object's `implements` clause and fields.
    ///
    /// Extensions can add `implements B` whose own parent set may have
    /// grown since the object was first generated, so the base's
    /// recorded `implements_interfaces` is stale. Each name's clause
    /// is re-expanded to the transitive closure (through the interface
    /// graph) and missing parents are added to the base def, held out
    /// from extensions to avoid double-declaration.
    ///
    /// Then a field whose name matches a parent's is overwritten with
    /// the parent's signature; parent fields not yet declared get
    /// appended to the base def.
    pub(crate) fn backfill_inherited_object_fields(&mut self) {
        for name in unique_names(&self.object_type_defs) {
            let Some(base_idx) = base_def_index(&self.object_type_defs, &name) else {
                continue;
            };
            self.expand_object_implements_closure(&name, base_idx);

            let parents = existing_implements_of(&self.object_type_defs, &name);
            let mut inherited = parent_fields_now(&parents, &self.interface_type_defs);
            let def_indices = def_indices_with_name(&self.object_type_defs, &name);
            for &i in &def_indices {
                for f in self.object_type_defs[i].fields_def.iter_mut() {
                    if let Some(parent_fdef) = inherited.shift_remove(&f.name.name) {
                        *f = parent_fdef;
                    }
                }
            }
            self.object_type_defs[base_idx]
                .fields_def
                .extend(inherited.into_values());
        }
    }

    /// Re-expand `name`'s direct implements set (taken across base +
    /// extensions) to its transitive closure via the interface graph,
    /// and add any newly-required parents to the base def. Anything an
    /// extension already declares is held out so the same interface is
    /// never listed twice.
    fn expand_object_implements_closure(&mut self, name: &Name, base_idx: usize) {
        let iface_direct = interface_implements_union(&self.interface_type_defs);
        let start = existing_implements_of(&self.object_type_defs, name);
        let closure = transitive_closure(start, &iface_direct, None);
        let extension_declared: IndexSet<Name> = self
            .object_type_defs
            .iter()
            .filter(|o| o.extend && &o.name == name)
            .flat_map(|o| o.implements_interfaces.iter().cloned())
            .collect();
        let base = &mut self.object_type_defs[base_idx].implements_interfaces;
        for parent in closure {
            if !extension_declared.contains(&parent) {
                base.insert(parent);
            }
        }
    }
}

/// Union of `implements_interfaces` across every object def sharing
/// `name` (base + extensions). Used both when an extension rolls
/// additional implements (so it knows what to avoid) and when the
/// backfill needs the full implements set for the type.
fn existing_implements_of(defs: &[ObjectTypeDef], name: &Name) -> IndexSet<Name> {
    let mut out: IndexSet<Name> = IndexSet::new();
    for def in defs {
        if &def.name == name {
            out.extend(def.implements_interfaces.iter().cloned());
        }
    }
    out
}

impl StackedEntity for ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}
