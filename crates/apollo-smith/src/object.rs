use crate::description::Description;
use crate::directive::Directive;
use crate::directive::DirectiveLocation;
use crate::field::FieldDef;
use crate::interface::base_def_index;
use crate::interface::def_indices_with_name;
use crate::interface::field_signatures_for;
use crate::interface::parent_fields_from_defs;
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

        // Extensions can add new `implements` clauses but must not
        // duplicate prior picks or overwrite a field signature the
        // type already commits to. Objects can't appear in the
        // interface graph, so no cycle protection is needed.
        let existing_field_signatures = field_signatures_for(&self.object_type_defs, &name);
        let implements_interfaces = self.additional_implements(&existing_field_signatures, None)?;
        let existing_field_names: Vec<Name> = existing_field_signatures
            .keys()
            .map(|k| Name::new(k.clone()))
            .collect();
        let exclude_fields: Vec<&Name> = existing_field_names.iter().collect();
        let fields_def = self.fields_definition(&exclude_fields)?;
        let directives = self.directives(DirectiveLocation::Object)?;

        if extend
            && directives.is_empty()
            && fields_def.is_empty()
            && implements_interfaces.is_empty()
        {
            return Err(arbitrary::Error::IncorrectFormat);
        }

        Ok(ObjectTypeDef {
            description,
            directives,
            implements_interfaces,
            name,
            fields_def,
            extend,
        })
    }

    /// Reconcile each object's `implements` clause and fields once
    /// the interface backfill has finished. Each interface already
    /// holds its full inherited field set by that point, so the
    /// object only needs to copy from its direct parents.
    pub(crate) fn backfill_inherited_object_fields(&mut self) {
        for name in unique_names(&self.object_type_defs) {
            let Some(base_idx) = base_def_index(&self.object_type_defs, &name) else {
                continue;
            };
            self.expand_transitive_object_implementations(&name, base_idx);

            let parents = self.implements_graph.direct_parents(&name);
            let mut inherited_fields = parent_fields_from_defs(&parents, &self.interface_type_defs);
            // Rewrite object-declared fields to the parent interface's
            // signature so the object satisfies the interface contract
            // exactly (matching type + args).
            let def_indices = def_indices_with_name(&self.object_type_defs, &name);
            for &i in &def_indices {
                for f in self.object_type_defs[i].fields_def.iter_mut() {
                    if let Some(parent_fdef) = inherited_fields.shift_remove(&f.name.name) {
                        *f = parent_fdef;
                    }
                }
            }
            // Append the interface fields the object never declared.
            self.object_type_defs[base_idx]
                .fields_def
                .extend(inherited_fields.into_values());
        }
    }

    /// Write `name`'s transitive interface parents onto its base def, skipping any already declared by an extension.
    fn expand_transitive_object_implementations(&mut self, name: &Name, base_idx: usize) {
        let mut all_implemented_interfaces = self.implements_graph.closure(name);
        // Closure includes `name`, but an object never implements itself.
        all_implemented_interfaces.shift_remove(name);

        let interfaces_declared_by_extensions: IndexSet<Name> = self
            .object_type_defs
            .iter()
            .filter(|o| o.extend && &o.name == name)
            .flat_map(|o| o.implements_interfaces.iter().cloned())
            .collect();
        let interfaces_to_add = all_implemented_interfaces
            .into_iter()
            .filter(|p| !interfaces_declared_by_extensions.contains(p));
        self.object_type_defs[base_idx]
            .implements_interfaces
            .extend(interfaces_to_add);
    }
}

impl StackedEntity for ObjectTypeDef {
    fn name(&self) -> &Name {
        &self.name
    }

    fn fields_def(&self) -> &[FieldDef] {
        &self.fields_def
    }
}
