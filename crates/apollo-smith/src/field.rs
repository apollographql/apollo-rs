use crate::definition::TypeDefinition;
use crate::selection_set::SelectedField;
use crate::DocumentBuilder;
use apollo_compiler::ast;
use apollo_compiler::ast::Argument;
use apollo_compiler::ast::DirectiveLocation;
use apollo_compiler::ast::Field;
use apollo_compiler::ast::FieldDefinition;
use apollo_compiler::ast::Type;
use apollo_compiler::coordinate::TypeAttributeCoordinate;
use apollo_compiler::Name;
use apollo_compiler::Node;
use arbitrary::Result as ArbitraryResult;
use indexmap::IndexSet;

impl DocumentBuilder<'_> {
    /// Create an arbitrary list of `FieldDefinition`
    pub fn fields_definition(
        &mut self,
        exclude: &IndexSet<Name>,
    ) -> ArbitraryResult<Vec<FieldDefinition>> {
        let available_types = self.list_existing_types();
        self.fields_of_types(exclude, &available_types)
    }

    /// Fields for the definition of `owner`, which may also reference
    /// `owner` itself.
    pub(crate) fn fields_definition_for(
        &mut self,
        owner: &Name,
        exclude: &IndexSet<Name>,
    ) -> ArbitraryResult<Vec<FieldDefinition>> {
        let available_types = self.output_field_types(owner);
        self.fields_of_types(exclude, &available_types)
    }

    /// Types a field of `owner` can have. `owner` isn't an existing type
    /// yet while its first definition is being generated, so it's added.
    pub(crate) fn output_field_types(&self, owner: &Name) -> Vec<Type> {
        let mut types = self.list_existing_types();
        let own = Type::Named(owner.clone());
        if !types.contains(&own) {
            types.push(own);
        }
        types
    }

    fn fields_of_types(
        &mut self,
        exclude: &IndexSet<Name>,
        available_types: &[Type],
    ) -> ArbitraryResult<Vec<FieldDefinition>> {
        // Types need at least one field, and most end up with a few.
        let mut fields_names = IndexSet::new();
        loop {
            let name = self.name_with_index(fields_names.len())?;
            if !exclude.contains(&name) {
                fields_names.insert(name);
            }
            if self.u.is_empty() || !self.u.ratio(3, 4)? {
                break;
            }
        }

        fields_names
            .into_iter()
            .map(|field_name| {
                Ok(FieldDefinition {
                    description: self.optional_description()?,
                    name: field_name,
                    arguments: self
                        .u
                        .arbitrary()
                        .unwrap_or(false)
                        .then(|| self.arguments_definition())
                        .transpose()?
                        .unwrap_or_default(),
                    ty: self.choose_ty(available_types)?,
                    directives: self.directives(DirectiveLocation::FieldDefinition)?,
                })
            })
            .collect()
    }

    /// The parent's field definitions that can be selected under their own
    /// name without conflicting with what's already selected here. This is a
    /// first pass before arguments are generated, so arguments are compared
    /// only when they're cached. `can_merge` makes the final check.
    fn mergeable_field_defs(&self, parent: &ast::Definition) -> Vec<FieldDefinition> {
        parent
            .fields()
            .iter()
            .filter(|def| {
                let coord = TypeAttributeCoordinate {
                    ty: parent.type_name().clone(),
                    attribute: def.name.clone(),
                };
                match self.cached_arguments(&coord) {
                    Some(args) => self.can_merge(
                        &def.name,
                        &SelectedField {
                            parent: parent.type_name().clone(),
                            field_name: def.name.clone(),
                            ty: def.ty.clone(),
                            args: args.clone(),
                        },
                    ),
                    None => self.can_merge_ignoring_args(
                        &def.name,
                        parent.type_name(),
                        &def.name,
                        &def.ty,
                    ),
                }
            })
            .map(|definition| (**definition).clone())
            .collect()
    }

    /// Sometimes alias a field, often to a response name already selected
    /// alongside it so that field merging gets exercised.
    fn alias(&mut self, index: usize) -> ArbitraryResult<Option<Name>> {
        if !self.u.ratio(1, 4)? {
            return Ok(None);
        }
        let existing = self.current_response_names();
        if !existing.is_empty() && self.u.arbitrary()? {
            return Ok(Some(self.u.choose(&existing)?.clone()));
        }
        Ok(Some(self.name_with_index(index)?))
    }

    /// Arguments for a selection of `def`, and whether any were already
    /// cached for it. Reusing cached arguments keeps same-named selections
    /// mergeable, but a field under a new alias has nothing to merge with,
    /// so it gets new arguments.
    fn arguments_for(
        &mut self,
        coord: &TypeAttributeCoordinate,
        def: &FieldDefinition,
        fresh_alias: bool,
    ) -> ArbitraryResult<(Vec<Argument>, bool)> {
        let cached_args = self.cached_arguments(coord).cloned();
        let cached = cached_args.is_some();
        let args = match cached_args {
            Some(args) if !fresh_alias => args,
            _ => self.field_arguments(&def.arguments)?,
        };
        Ok((args, cached))
    }

    /// Create an arbitrary `Field` on the composite type at the top of the stack
    pub fn field(&mut self, index: usize) -> ArbitraryResult<Field> {
        self.field_with_conditions(index, true)
    }

    /// Like `field`, without `@skip` or `@include` on the field itself when
    /// `conditional` is unset, for the root field of a subscription.
    pub(crate) fn field_with_conditions(
        &mut self,
        index: usize,
        conditional: bool,
    ) -> ArbitraryResult<Field> {
        let parent = self
            .stack
            .last()
            .expect("a composite type must be added on the stack");
        let parent_name = parent.type_name().clone();
        // Picking only fields that can merge with what's already selected
        // keeps conflicts from turning most selections into `__typename`.
        let candidates = self.mergeable_field_defs(parent);
        if candidates.is_empty() {
            return Ok(self.typename_field());
        }

        let chosen_field_def = self.u.choose(&candidates)?.clone();

        let name = chosen_field_def.name.clone();
        let coord = TypeAttributeCoordinate {
            ty: parent_name.clone(),
            attribute: name.clone(),
        };
        let alias = self.alias(index)?;
        let fresh_alias = alias
            .as_ref()
            .is_some_and(|alias| !self.has_response_name(alias));
        let variables = self.variable_checkpoint();
        let (args, cached) = self.arguments_for(&coord, &chosen_field_def, fresh_alias)?;
        let selected = SelectedField {
            parent: parent_name,
            field_name: name.clone(),
            ty: chosen_field_def.ty.clone(),
            args: args.clone(),
        };
        // An alias that can't merge with what's already selected under it
        // falls back to the field's own name.
        let alias = alias.filter(|alias| self.can_merge(alias, &selected));
        let response_name = alias.clone().unwrap_or_else(|| name.clone());
        if !self.can_merge(&response_name, &selected) {
            // Variables first used by these arguments would be left unused
            self.rollback_variables(variables);
            return Ok(self.typename_field());
        }
        if !cached {
            self.cache_arguments(coord, args.clone());
        }
        self.record_selection(response_name.clone(), selected);
        let directives = self.selection_directives(DirectiveLocation::Field, conditional)?;

        let selection_set = if !crate::ty::is_builtin(&chosen_field_def.ty) {
            // Put current ty on the stack
            if self.stack_ty(&chosen_field_def.ty) {
                let res = self.in_merged_subselection(&response_name, |b| b.selection_set());
                self.stack.pop();
                Some(res?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Field {
            alias,
            name,
            arguments: args.into_iter().map(Node::new).collect(),
            directives,
            selection_set,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection_set::SelectedField;
    use crate::test_support::generated_against_any;
    use crate::test_support::prop_assert_valid;
    use crate::DocumentBuilder;
    use apollo_compiler::name;
    use apollo_compiler::ty;
    use arbitrary::Unstructured;
    use proptest::prelude::*;

    // Fields that aliases can collide with: the same names across types,
    // arguments, and a union whose members share field names.
    const ALIASES: &str = "schema { query: Query }
        type Query { a: Int, b: Int, c(x: Int): String, d: String, o: O, u: U }
        type O { a: Int, b: String, o: O }
        type P { a: String, b: String }
        union U = O | P";

    // One field with an argument, so it's often selected with different
    // arguments under different aliases.
    const ARGUMENTS: &str = "schema { query: Query }
        type Query { c(x: Int): String, d: String }";

    proptest! {
        #[test]
        fn aliased_selections_stay_mergeable(
            case in generated_against_any(&[ALIASES, ARGUMENTS], |b| b.operation_definition()),
        ) {
            if let Some(doc) = &case.document {
                prop_assert_valid(doc)?;
            }
        }
    }

    // Once `B.x` is selected as a `String`, `A.x` can't share its response
    // name, so only `y` is offered.
    #[test]
    fn offers_only_fields_that_merge_with_earlier_selections() {
        let schema = "schema { query: Query }
            type Query { u: U }
            union U = A | B
            type A { x: Int, y: Int }
            type B { x: String, y: Int }";
        let doc = apollo_compiler::ast::Document::parse(schema, "schema.graphql")
            .expect("parse compiler document");
        let mut u = Unstructured::new(&[]);
        let mut builder = DocumentBuilder::with_document(&mut u, doc).expect("builder from schema");
        let a = builder
            .object_type_defs
            .iter()
            .find(|o| o.type_name() == "A")
            .cloned()
            .expect("schema defines A");
        let x = name!(x);
        let (offered, _) = builder.tracking_merges(|b| {
            b.record_selection(
                x.clone(),
                SelectedField {
                    parent: name!(B),
                    field_name: x.clone(),
                    ty: ty!(String),
                    args: Vec::new(),
                },
            );
            b.mergeable_field_defs(&a)
        });
        let offered: Vec<&str> = offered.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(offered, ["y"]);
    }

    fn builder_for<'a>(u: &'a mut Unstructured<'a>, schema: &str) -> DocumentBuilder<'a> {
        let doc = apollo_compiler::ast::Document::parse(schema, "schema.graphql")
            .expect("parse compiler document");
        DocumentBuilder::with_document(u, doc).expect("builder from schema")
    }

    #[test]
    fn offers_the_type_being_defined_as_a_field_type() {
        let mut u = Unstructured::new(&[]);
        let builder = builder_for(&mut u, "schema { query: Query } type Query { a: Int }");
        let user = ty!(User);
        assert!(builder
            .output_field_types(user.inner_named_type())
            .contains(&user));
    }

    // An extension of `A` generated after `B` can reference it, while `B`
    // already references `A`.
    #[test]
    fn extensions_offer_types_defined_after_the_extended_type() {
        let mut u = Unstructured::new(&[]);
        let builder = builder_for(
            &mut u,
            "schema { query: Query } type Query { a: A } type A { x: Int } type B { a: A }",
        );
        let a = name!(A);
        let b = ty!(B);
        assert!(builder.output_field_types(&a).contains(&b));
    }

    // Each field after the first costs input, so a type built from
    // exhausted input stops at one.
    #[test]
    fn exhausted_input_generates_one_field() {
        let mut u = Unstructured::new(&[]);
        let mut builder = builder_for(&mut u, "schema { query: Query } type Query { a: Int }");
        let fields = builder
            .fields_definition(&IndexSet::new())
            .expect("fields from exhausted input");
        assert_eq!(fields.len(), 1);
    }
}
