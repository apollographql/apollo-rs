use std::collections::HashMap;

use apollo_compiler::executable::Selection;
use apollo_compiler::executable::SelectionSet;
use apollo_compiler::schema::ExtendedType;
use apollo_compiler::validation::Valid;
use apollo_compiler::ExecutableDocument;
use apollo_compiler::Name;
use apollo_compiler::Schema;
use arbitrary::Result;
use arbitrary::Unstructured;
use serde_json_bytes::json;
use serde_json_bytes::serde_json::Number;
use serde_json_bytes::Map;
use serde_json_bytes::Value;

const TYPENAME: &str = "__typename";

pub type Generator = Box<dyn Fn(&mut Unstructured) -> Result<Value>>;

pub struct ResponseBuilder<'a, 'doc, 'schema> {
    u: &'a mut Unstructured<'a>,
    doc: &'doc Valid<ExecutableDocument>,
    schema: &'schema Valid<Schema>,
    custom_scalar_generators: HashMap<Name, Generator>,
    min_list_size: usize,
    max_list_size: usize,
    null_ratio: Option<(u8, u8)>,
    operation_name: Option<&'doc str>,
}

impl<'a, 'doc, 'schema> ResponseBuilder<'a, 'doc, 'schema> {
    pub fn new(
        u: &'a mut Unstructured<'a>,
        doc: &'doc Valid<ExecutableDocument>,
        schema: &'schema Valid<Schema>,
    ) -> Self {
        Self {
            u,
            doc,
            schema,
            custom_scalar_generators: HashMap::new(),
            min_list_size: 0,
            max_list_size: 5,
            null_ratio: None,
            operation_name: None,
        }
    }

    /// Register a generator function for generating custom scalar values.
    pub fn with_custom_scalar(mut self, scalar_name: Name, generator: Generator) -> Self {
        self.custom_scalar_generators.insert(scalar_name, generator);
        self
    }

    /// Set the minimum number of items per list field. If unset, defaults to 0.
    pub fn with_min_list_size(mut self, min_size: usize) -> Self {
        self.min_list_size = min_size;
        self
    }

    /// Set the maximum number of items per list field. If unset, defaults to 5.
    pub fn with_max_list_size(mut self, max_size: usize) -> Self {
        self.max_list_size = max_size;
        self
    }

    /// Set the frequency of null values for nullable fields. If unset, fields will never be null.
    pub fn with_null_ratio(mut self, numerator: u8, denominator: u8) -> Self {
        self.null_ratio = Some((numerator, denominator));
        self
    }

    /// Set the operation name to generate a response for. If unset, uses the anonymous operation.
    /// If the operation does not exist, returns a response with `data: null`.
    pub fn with_operation_name(mut self, operation_name: Option<&'doc str>) -> Self {
        self.operation_name = operation_name;
        self
    }

    /// Builds a `Value` matching the shape of `self.doc`
    pub fn build(mut self) -> Result<Value> {
        if let Ok(operation) = self.doc.operations.get(self.operation_name) {
            Ok(json!({ "data": self.arbitrary_selection_set(&operation.selection_set)? }))
        } else {
            Ok(json!({ "data": null }))
        }
    }

    fn arbitrary_selection_set(&mut self, selection_set: &SelectionSet) -> Result<Value> {
        let mut result = Map::new();

        for selection in &selection_set.selections {
            match selection {
                Selection::Field(field) => {
                    if field.name == TYPENAME {
                        result.insert(
                            field.name.to_string(),
                            Value::String(selection_set.ty.to_string().into()),
                        );
                    } else if !field.ty().is_non_null() && self.should_be_null()? {
                        result.insert(field.name.to_string(), Value::Null);
                    } else if field.selection_set.is_empty() && !field.ty().is_list() {
                        result.insert(
                            field.name.to_string(),
                            self.arbitrary_leaf_field(field.ty().inner_named_type())?,
                        );
                    } else if field.selection_set.is_empty() && field.ty().is_list() {
                        result.insert(
                            field.name.to_string(),
                            self.repeated_arbitrary_leaf_field(field.ty().inner_named_type())?,
                        );
                    } else if !field.selection_set.is_empty() && !field.ty().is_list() {
                        result.insert(
                            field.name.to_string(),
                            self.arbitrary_selection_set(&field.selection_set)?,
                        );
                    } else {
                        result.insert(
                            field.name.to_string(),
                            self.repeated_arbitrary_selection_set(&field.selection_set)?,
                        );
                    }
                }
                Selection::FragmentSpread(fragment) => {
                    if let Some(fragment_def) = self.doc.fragments.get(&fragment.fragment_name) {
                        let value = self.arbitrary_selection_set(&fragment_def.selection_set)?;
                        if let Some(value_obj) = value.as_object() {
                            result.extend(value_obj.clone());
                        }
                    }
                }
                Selection::InlineFragment(inline_fragment) => {
                    let value = self.arbitrary_selection_set(&inline_fragment.selection_set)?;
                    if let Some(value_obj) = value.as_object() {
                        result.extend(value_obj.clone());
                    }
                }
            }
        }

        Ok(Value::Object(result))
    }

    fn repeated_arbitrary_selection_set(&mut self, selection_set: &SelectionSet) -> Result<Value> {
        let num_values = self.arbitrary_len()?;
        let mut values = Vec::with_capacity(num_values);
        for _ in 0..num_values {
            values.push(self.arbitrary_selection_set(selection_set)?);
        }
        Ok(Value::Array(values))
    }

    fn arbitrary_leaf_field(&mut self, type_name: &Name) -> Result<Value> {
        let extended_ty = self.schema.types.get(type_name).unwrap();
        match extended_ty {
            ExtendedType::Enum(enum_ty) => {
                let enum_value = self.u.choose_iter(enum_ty.values.values())?;
                Ok(Value::String(enum_value.value.to_string().into()))
            }
            ExtendedType::Scalar(scalar) => {
                if scalar.name == "Boolean" {
                    let random_bool = self.u.arbitrary::<bool>()?;
                    Ok(Value::Bool(random_bool))
                } else if scalar.name == "Int" || scalar.name == "ID" {
                    let random_int = self.u.int_in_range(0..=100)?;
                    Ok(Value::Number(random_int.into()))
                } else if scalar.name == "Float" {
                    let random_float = self.u.arbitrary::<f64>()?;
                    Ok(Value::Number(Number::from_f64(random_float).unwrap()))
                } else if scalar.name == "String" {
                    let random_string = self.u.arbitrary::<String>()?;
                    Ok(Value::String(random_string.into()))
                } else if let Some(custom_generator) =
                    self.custom_scalar_generators.get(&scalar.name)
                {
                    let random_value = custom_generator(self.u)?;
                    Ok(random_value)
                } else {
                    // Likely a custom scalar which hasn't had a generator registered
                    let random_string = self.u.arbitrary::<String>()?;
                    Ok(Value::String(random_string.into()))
                }
            }
            _ => unreachable!(
                "We are in a field with an empty selection set, so it must be a scalar or enum type"
            ),
        }
    }

    fn repeated_arbitrary_leaf_field(&mut self, type_name: &Name) -> Result<Value> {
        let num_values = self.arbitrary_len()?;
        let mut values = Vec::with_capacity(num_values);
        for _ in 0..num_values {
            values.push(self.arbitrary_leaf_field(type_name)?);
        }
        Ok(Value::Array(values))
    }

    fn arbitrary_len(&mut self) -> Result<usize> {
        // Ideally, we would use `u.arbitrary_len()` to ensure we can generate enough values from
        // the remaining bytes, but it needs a type `T: Arbitrary` which `Value` does not implement.
        self.u.int_in_range(self.min_list_size..=self.max_list_size)
    }

    fn should_be_null(&mut self) -> Result<bool> {
        if let Some((numerator, denominator)) = self.null_ratio {
            self.u.ratio(numerator, denominator)
        } else {
            Ok(false)
        }
    }
}
