use crate::coordinate::TypeAttributeCoordinate;
use crate::executable::BuildError;
use crate::validation::diagnostics::ValidationError;
use crate::validation::operation::OperationValidationConfig;
use crate::validation::DiagnosticList;
use crate::validation::{FileId, ValidationDatabase};
use crate::{ast, executable, schema, Node};
use apollo_parser::LimitTracker;
use indexmap::IndexMap;
use std::cell::OnceCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::rc::Rc;

/// Represents a field selected against a parent type.
#[derive(Debug, Clone, Copy, Hash)]
pub(crate) struct FieldSelection<'a> {
    /// The type of the selection set this field selection is part of.
    pub parent_type: &'a ast::NamedType,
    pub field: &'a Node<executable::Field>,
}

impl FieldSelection<'_> {
    pub fn coordinate(&self) -> TypeAttributeCoordinate {
        TypeAttributeCoordinate {
            ty: self.parent_type.clone(),
            attribute: self.field.name.clone(),
        }
    }
}

/// Expand one or more selection sets to a list of all fields selected.
pub(crate) fn expand_selections<'doc>(
    fragments: &'doc IndexMap<ast::Name, Node<executable::Fragment>>,
    selection_sets: impl Iterator<Item = &'doc executable::SelectionSet>,
) -> Vec<FieldSelection<'doc>> {
    let mut selections = vec![];
    let mut queue: VecDeque<&executable::SelectionSet> = selection_sets.collect();
    let mut seen_fragments = HashSet::new();

    while let Some(next_set) = queue.pop_front() {
        for selection in &next_set.selections {
            match selection {
                executable::Selection::Field(field) => selections.push(FieldSelection {
                    parent_type: &next_set.ty,
                    field,
                }),
                executable::Selection::InlineFragment(spread) => {
                    queue.push_back(&spread.selection_set)
                }
                executable::Selection::FragmentSpread(spread)
                    if !seen_fragments.contains(&spread.fragment_name) =>
                {
                    seen_fragments.insert(&spread.fragment_name);
                    if let Some(fragment) = fragments.get(&spread.fragment_name) {
                        queue.push_back(&fragment.selection_set);
                    }
                }
                executable::Selection::FragmentSpread(_) => {
                    // Already seen
                }
            }
        }
    }

    selections
}

fn is_composite(ty: &schema::ExtendedType) -> bool {
    use schema::ExtendedType::*;
    matches!(ty, Object(_) | Interface(_) | Union(_))
}

/// A temporary index for frequent argument lookups by name, using a hash map if there are many
/// arguments.
enum ArgumentLookup<'a> {
    Map(HashMap<&'a ast::Name, &'a Node<ast::Argument>>),
    List(&'a [Node<ast::Argument>]),
}
impl<'a> ArgumentLookup<'a> {
    fn new(list: &'a [Node<ast::Argument>]) -> Self {
        if list.len() > 20 {
            Self::Map(list.iter().map(|arg| (&arg.name, arg)).collect())
        } else {
            Self::List(list)
        }
    }

    fn by_name(&self, name: &ast::Name) -> Option<&'a Node<ast::Argument>> {
        match self {
            Self::Map(map) => map.get(name).copied(),
            Self::List(list) => list.iter().find(|arg| arg.name == *name),
        }
    }
}

/// Check if two field selections from the overlapping types are the same, so the fields can be merged.
fn same_name_and_arguments(
    field_a: FieldSelection<'_>,
    field_b: FieldSelection<'_>,
) -> Result<(), BuildError> {
    // 2bi. fieldA and fieldB must have identical field names.
    if field_a.field.name != field_b.field.name {
        return Err(BuildError::ConflictingFieldName {
            alias: field_a.field.response_key().clone(),
            original_location: field_a.field.location(),
            original_selection: field_a.coordinate(),
            conflicting_location: field_b.field.location(),
            conflicting_selection: field_b.coordinate(),
        });
    }

    // 2bii. fieldA and fieldB must have identical sets of arguments.
    let conflicting_field_argument =
        |original_arg: Option<&Node<ast::Argument>>,
         redefined_arg: Option<&Node<ast::Argument>>| {
            debug_assert!(
                    original_arg.is_some() || redefined_arg.is_some(),
                    "a conflicting field argument error can only exist when at least one field has the argument",
                );

            // We can take the name from either one of the arguments as they are necessarily the same.
            let arg = original_arg.or(redefined_arg).unwrap();

            BuildError::ConflictingFieldArgument {
                // field_a and field_b have the same name so we can use either one.
                alias: field_b.field.name.clone(),
                original_location: field_a.field.location(),
                original_coordinate: field_a.coordinate().with_argument(arg.name.clone()),
                original_value: original_arg.map(|arg| (*arg.value).clone()),
                conflicting_location: field_b.field.location(),
                conflicting_coordinate: field_b.coordinate().with_argument(arg.name.clone()),
                conflicting_value: redefined_arg.map(|arg| (*arg.value).clone()),
            }
        };

    // Check if fieldB provides the same argument names and values as fieldA (order-independent).
    let self_args = ArgumentLookup::new(&field_a.field.arguments);
    let other_args = ArgumentLookup::new(&field_b.field.arguments);
    for arg in &field_a.field.arguments {
        let Some(other_arg) = other_args.by_name(&arg.name) else {
            return Err(conflicting_field_argument(Some(arg), None));
        };

        if !same_value(&other_arg.value, &arg.value) {
            return Err(conflicting_field_argument(Some(arg), Some(other_arg)));
        }
    }
    // Check if fieldB provides any arguments that fieldA does not provide.
    for arg in &field_b.field.arguments {
        if self_args.by_name(&arg.name).is_none() {
            return Err(conflicting_field_argument(None, Some(arg)));
        };
    }

    Ok(())
}

/// Compare two input values, with two special cases for objects: assuming no duplicate keys,
/// and order-independence.
fn same_value(left: &ast::Value, right: &ast::Value) -> bool {
    match (left, right) {
        (ast::Value::Null, ast::Value::Null) => true,
        (ast::Value::Enum(left), ast::Value::Enum(right)) => left == right,
        (ast::Value::Variable(left), ast::Value::Variable(right)) => left == right,
        (ast::Value::String(left), ast::Value::String(right)) => left == right,
        (ast::Value::Float(left), ast::Value::Float(right)) => left == right,
        (ast::Value::Int(left), ast::Value::Int(right)) => left == right,
        (ast::Value::Boolean(left), ast::Value::Boolean(right)) => left == right,
        (ast::Value::List(left), ast::Value::List(right)) => left
            .iter()
            .zip(right.iter())
            .all(|(left, right)| same_value(left, right)),
        (ast::Value::Object(left), ast::Value::Object(right)) if left.len() == right.len() => {
            // This check could miss out on keys that exist in `right`, but not in `left`, if `left` contains duplicate keys.
            // We assume that that doesn't happen. GraphQL does not support duplicate keys and
            // that is checked elsewhere in validation.
            left.iter().all(|(key, value)| {
                right
                    .iter()
                    .find(|(other_key, _)| key == other_key)
                    .is_some_and(|(_, other_value)| same_value(value, other_value))
            })
        }
        _ => false,
    }
}

fn same_output_type_shape(
    schema: &schema::Schema,
    selection_a: FieldSelection<'_>,
    selection_b: FieldSelection<'_>,
) -> Result<(), BuildError> {
    let field_a = &selection_a.field.definition;
    let field_b = &selection_b.field.definition;

    let mut type_a = &field_a.ty;
    let mut type_b = &field_b.ty;

    let mismatching_type_diagnostic = || BuildError::ConflictingFieldType {
        alias: selection_a.field.response_key().clone(),
        original_location: selection_a.field.location(),
        original_coordinate: selection_a.coordinate(),
        original_type: field_a.ty.clone(),
        conflicting_location: selection_b.field.location(),
        conflicting_coordinate: selection_b.coordinate(),
        conflicting_type: field_b.ty.clone(),
    };

    // Steps 3 and 4 of the spec text unwrap both types simultaneously down to the named type.
    // The apollo-rs representation of NonNull and Lists makes it tricky to follow the steps
    // exactly.
    //
    // Instead we unwrap lists and non-null lists first, which leaves just a named type or a
    // non-null named type...
    while !type_a.is_named() || !type_b.is_named() {
        // 4. If typeA or typeB is List.
        // 4a. If typeA or typeB is not List, return false.
        // 4b. Let typeA be the item type of typeA
        // 4c. Let typeB be the item type of typeB
        (type_a, type_b) = match (type_a, type_b) {
            (ast::Type::List(type_a), ast::Type::List(type_b))
            | (ast::Type::NonNullList(type_a), ast::Type::NonNullList(type_b)) => {
                (type_a.as_ref(), type_b.as_ref())
            }
            (ast::Type::List(_), _)
            | (_, ast::Type::List(_))
            | (ast::Type::NonNullList(_), _)
            | (_, ast::Type::NonNullList(_)) => return Err(mismatching_type_diagnostic()),
            // Now it's a named type.
            (type_a, type_b) => (type_a, type_b),
        };
    }

    // Now we are down to two named types, we can check that they have the same nullability...
    let (type_a, type_b) = match (type_a, type_b) {
        (ast::Type::NonNullNamed(a), ast::Type::NonNullNamed(b)) => (a, b),
        (ast::Type::Named(a), ast::Type::Named(b)) => (a, b),
        _ => return Err(mismatching_type_diagnostic()),
    };

    let (Some(def_a), Some(def_b)) = (schema.types.get(type_a), schema.types.get(type_b)) else {
        return Ok(()); // Cannot do much if we don't know the type
    };

    match (def_a, def_b) {
        // 5. If typeA or typeB is Scalar or Enum.
        (
            def_a @ (schema::ExtendedType::Scalar(_) | schema::ExtendedType::Enum(_)),
            def_b @ (schema::ExtendedType::Scalar(_) | schema::ExtendedType::Enum(_)),
        ) => {
            // 5a. If typeA and typeB are the same type return true, otherwise return false.
            if def_a == def_b {
                Ok(())
            } else {
                Err(mismatching_type_diagnostic())
            }
        }
        // 6. If typeA or typeB is not a composite type, return false.
        (def_a, def_b) if is_composite(def_a) && is_composite(def_b) => Ok(()),
        _ => Err(mismatching_type_diagnostic()),
    }
}

/// A boolean that turns on after the first check.
struct OnceBool(std::cell::Cell<bool>);
impl OnceBool {
    fn new() -> Self {
        Self(false.into())
    }

    /// Returns `false` the first time it is called, then returns `true` forever.
    fn already_done(&self) -> bool {
        self.0.replace(true)
    }
}

/// Represents a merged field set that may or may not be valid.
struct MergedFieldSet<'doc> {
    selections: Vec<FieldSelection<'doc>>,
    grouped_by_output_names: OnceCell<IndexMap<ast::Name, Vec<FieldSelection<'doc>>>>,
    grouped_by_common_parents: OnceCell<Vec<Vec<FieldSelection<'doc>>>>,
    same_response_shape_guard: OnceBool,
    same_for_common_parents_guard: OnceBool,
}

impl<'doc> MergedFieldSet<'doc> {
    fn new(selections: Vec<FieldSelection<'doc>>) -> Self {
        Self {
            selections,
            grouped_by_output_names: Default::default(),
            grouped_by_common_parents: Default::default(),
            same_response_shape_guard: OnceBool::new(),
            same_for_common_parents_guard: OnceBool::new(),
        }
    }

    /// Given a set of fields, do all the fields that contribute to 1 output name have the same
    /// shape?
    ///
    /// This prevents leaf output fields from having an inconsistent type.
    fn same_response_shape_by_name(
        &self,
        validator: &mut FieldsInSetCanMerge<'_, 'doc>,
        diagnostics: &mut DiagnosticList,
    ) {
        // No need to do this if this field set has been checked before.
        if self.same_response_shape_guard.already_done() {
            return;
        }

        for fields_for_name in self.group_by_output_name().values() {
            let Some((field_a, rest)) = fields_for_name.split_first() else {
                continue;
            };
            for field_b in rest {
                // Covers steps 3-5 of the spec algorithm.
                if let Err(err) = same_output_type_shape(validator.schema, *field_a, *field_b) {
                    diagnostics.push(field_b.field.location(), err);
                    continue;
                }
            }

            let mut nested_selection_sets = fields_for_name
                .iter()
                .map(|selection| &selection.field.selection_set)
                .filter(|set| !set.selections.is_empty())
                .peekable();
            if nested_selection_sets.peek().is_some() {
                let merged_set =
                    expand_selections(&validator.document.fragments, nested_selection_sets);
                validator.same_response_shape_by_name(merged_set, diagnostics);
            }
        }
    }

    /// Given a set of fields, do all the fields selecting from potentially overlapping types
    /// select from the same thing?
    ///
    /// This prevents selecting two different fields from the same type into the same name. That
    /// would be a contradiction because there would be no way to know which field takes precedence.
    fn same_for_common_parents_by_name(
        &self,
        validator: &mut FieldsInSetCanMerge<'_, 'doc>,
        diagnostics: &mut DiagnosticList,
    ) {
        // No need to do this if this field set has been checked before.
        if self.same_for_common_parents_guard.already_done() {
            return;
        }

        for fields_for_name in self.group_by_output_name().values() {
            let selection_for_name = validator.lookup(fields_for_name.clone());
            for fields_for_parents in selection_for_name.group_by_common_parents(validator.schema) {
                // 2bi. fieldA and fieldB must have identical field names.
                // 2bii. fieldA and fieldB must have identical sets of arguments.
                // The same arguments check is reflexive so we don't need to check all
                // combinations.
                let Some((field_a, rest)) = fields_for_parents.split_first() else {
                    continue;
                };
                for field_b in rest {
                    if let Err(diagnostic) = same_name_and_arguments(*field_a, *field_b) {
                        diagnostics.push(field_b.field.location(), diagnostic);
                        continue;
                    }
                }

                let mut nested_selection_sets = fields_for_parents
                    .iter()
                    .map(|selection| &selection.field.selection_set)
                    .filter(|set| !set.selections.is_empty())
                    .peekable();
                if nested_selection_sets.peek().is_some() {
                    let merged_set =
                        expand_selections(&validator.document.fragments, nested_selection_sets);
                    validator.same_for_common_parents_by_name(merged_set, diagnostics);
                }
            }
        }
    }

    fn group_by_output_name(&self) -> &IndexMap<schema::Name, Vec<FieldSelection<'doc>>> {
        self.grouped_by_output_names.get_or_init(|| {
            let mut map = IndexMap::<_, Vec<_>>::new();
            for selection in &self.selections {
                map.entry(selection.field.response_key().clone())
                    .or_default()
                    .push(*selection);
            }
            map
        })
    }

    /// Returns potentially overlapping groups of fields. Fields overlap if they are selected from
    /// the same concrete type or if they are selected from an abstract type (future schema changes
    /// can make any abstract type overlap with any other type).
    fn group_by_common_parents(&self, schema: &schema::Schema) -> &Vec<Vec<FieldSelection<'doc>>> {
        self.grouped_by_common_parents.get_or_init(|| {
            let mut abstract_parents = vec![];
            let mut concrete_parents = IndexMap::<_, Vec<_>>::new();
            for selection in &self.selections {
                match schema.types.get(selection.parent_type) {
                    Some(schema::ExtendedType::Object(object)) => {
                        concrete_parents
                            .entry(object.name.clone())
                            .or_default()
                            .push(*selection);
                    }
                    Some(schema::ExtendedType::Interface(_) | schema::ExtendedType::Union(_)) => {
                        abstract_parents.push(*selection);
                    }
                    _ => {}
                }
            }

            if concrete_parents.is_empty() {
                vec![abstract_parents]
            } else {
                concrete_parents
                    .into_values()
                    .map(|mut group| {
                        group.extend(abstract_parents.iter().copied());
                        group
                    })
                    .collect()
            }
        })
    }
}

/// For use as a hash map key, avoiding a clone of a potentially large array into the key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct FieldSelectionsId(u64);

impl FieldSelectionsId {
    fn new(selections: &[FieldSelection<'_>]) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hash;
        use std::hash::Hasher;

        // We can use the unseeded default hasher because the output will be
        // hashed again with a randomly seeded hasher and still lead to unpredictable output.
        let mut hasher = DefaultHasher::new();
        selections.hash(&mut hasher);
        Self(hasher.finish())
    }
}

// The field depth in the field merging validation matches the nesting level in the resulting
// data. It makes sense to use the same limit as serde_json.
const FIELD_DEPTH_LIMIT: usize = 128;

/// Implements the `FieldsInSetCanMerge()` validation.
/// https://spec.graphql.org/draft/#sec-Field-Selection-Merging
///
/// This uses the [validation algorithm described by XING][0] ([archived][1]), which
/// scales much better with larger selection sets that may have many overlapping fields,
/// and with widespread use of fragments.
///
/// [0]: https://tech.new-work.se/graphql-overlapping-fields-can-be-merged-fast-ea6e92e0a01
/// [1]: https://web.archive.org/web/20240208084612/https://tech.new-work.se/graphql-overlapping-fields-can-be-merged-fast-ea6e92e0a01
pub(crate) struct FieldsInSetCanMerge<'s, 'doc> {
    schema: &'s schema::Schema,
    document: &'doc executable::ExecutableDocument,
    /// Stores merged field sets.
    ///
    /// The value is an Rc because it needs to have an independent lifetime from `self`,
    /// so the cache can be updated while a field set is borrowed.
    cache: HashMap<FieldSelectionsId, Rc<MergedFieldSet<'doc>>>,
    // The recursion limit is used for two separate recursions, but they are not interleaved,
    // so the effective limit does apply to field nesting levels in both cases.
    recursion_limit: LimitTracker,
}

impl<'s, 'doc> FieldsInSetCanMerge<'s, 'doc> {
    pub(crate) fn new(
        schema: &'s schema::Schema,
        document: &'doc executable::ExecutableDocument,
    ) -> Self {
        Self {
            schema,
            document,
            cache: Default::default(),
            recursion_limit: LimitTracker::new(FIELD_DEPTH_LIMIT),
        }
    }

    pub(crate) fn validate_operation(
        &mut self,
        operation: &'doc Node<executable::Operation>,
        diagnostics: &mut DiagnosticList,
    ) {
        let fields = expand_selections(
            &self.document.fragments,
            std::iter::once(&operation.selection_set),
        );
        let set = self.lookup(fields);
        set.same_response_shape_by_name(self, diagnostics);
        set.same_for_common_parents_by_name(self, diagnostics);

        if self.recursion_limit.high > self.recursion_limit.limit {
            diagnostics.push(operation.location(), super::Details::RecursionLimitError);
        }
    }

    fn lookup(&mut self, selections: Vec<FieldSelection<'doc>>) -> Rc<MergedFieldSet<'doc>> {
        let id = FieldSelectionsId::new(&selections);
        self.cache
            .entry(id)
            .or_insert_with(|| Rc::new(MergedFieldSet::new(selections)))
            .clone()
    }

    fn same_for_common_parents_by_name(
        &mut self,
        selections: Vec<FieldSelection<'doc>>,
        diagnostics: &mut DiagnosticList,
    ) {
        if self.recursion_limit.check_and_increment() {
            return;
        }
        let field_set = self.lookup(selections);
        field_set.same_for_common_parents_by_name(self, diagnostics);
        self.recursion_limit.decrement();
    }

    fn same_response_shape_by_name(
        &mut self,
        selections: Vec<FieldSelection<'doc>>,
        diagnostics: &mut DiagnosticList,
    ) {
        if self.recursion_limit.check_and_increment() {
            return;
        }
        let field_set = self.lookup(selections);
        field_set.same_response_shape_by_name(self, diagnostics);
        self.recursion_limit.decrement();
    }
}

pub(crate) fn validate_selection_set(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    selection_set: &[ast::Selection],
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    diagnostics.extend(validate_selections(
        db,
        file_id,
        against_type,
        selection_set,
        context,
    ));

    diagnostics
}

pub(crate) fn validate_selections(
    db: &dyn ValidationDatabase,
    file_id: FileId,
    against_type: Option<&ast::NamedType>,
    selection_set: &[ast::Selection],
    context: OperationValidationConfig<'_>,
) -> Vec<ValidationError> {
    let mut diagnostics = vec![];

    for selection in selection_set {
        match selection {
            ast::Selection::Field(field) => diagnostics.extend(super::field::validate_field(
                db,
                file_id,
                against_type,
                field,
                context.clone(),
            )),
            ast::Selection::FragmentSpread(fragment) => {
                diagnostics.extend(super::fragment::validate_fragment_spread(
                    db,
                    file_id,
                    against_type,
                    fragment,
                    context.clone(),
                ))
            }
            ast::Selection::InlineFragment(inline) => {
                diagnostics.extend(super::fragment::validate_inline_fragment(
                    db,
                    file_id,
                    against_type,
                    inline,
                    context.clone(),
                ))
            }
        }
    }

    diagnostics
}
